/* WebGL2 engine: compiles the uber shader, owns the render loop,
   exposes deterministic renderAt(phase) for exporters. */

var Engine = (function () {
  var canvas, gl, program, uniforms = {};
  var playing = true;
  var suspensionReasons = {};
  var suspensionCount = 0;
  var initialized = false;
  var contextLost = false;
  var frameHandle = 0;
  var frameTimer = 0;
  var targetFrameIntervalMs = 0;
  var lastDrawAt = 0;
  var dirty = true;
  var loopT = 0;            // seconds into current loop
  var lastTick = 0;
  var fps = 60, fpsAcc = 0, fpsN = 0, fpsCb = null;
  var contextCb = null;
  var getParams = null;     // injected: () => P

  var CONTEXT_OPTIONS = {
    antialias: false,
    preserveDrawingBuffer: false,
    powerPreference: "low-power",
    failIfMajorPerformanceCaveat: true
  };

  var UNIFORM_NAMES = [
    "u_res", "u_phase", "u_seed", "u_mode",
    "u_c1", "u_c2", "u_c3", "u_c4", "u_bg",
    "u_hue", "u_sat", "u_exposure", "u_contrast",
    "u_scale", "u_complex", "u_warp", "u_flow", "u_stretch",
    "u_light", "u_gloss", "u_lightAngle", "u_irid", "u_glow",
    "u_grain", "u_cell", "u_lines", "u_ca", "u_vig", "u_soft",
    "u_travel",
    "u_synth", "u_modeB", "u_mixOp", "u_blend",
    "u_genome", "u_g1", "u_g2", "u_g3"
  ];

  function compile(type, src) {
    var sh = gl.createShader(type);
    gl.shaderSource(sh, src);
    gl.compileShader(sh);
    if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
      throw new Error("Shader compile error:\n" + gl.getShaderInfoLog(sh));
    }
    return sh;
  }

  function createContext() {
    gl = canvas.getContext("webgl2", CONTEXT_OPTIONS);
    if (!gl) throw new Error("WebGL2 not available");
  }

  function buildPipeline() {
    uniforms = {};
    program = gl.createProgram();
    gl.attachShader(program, compile(gl.VERTEX_SHADER, VERT_SRC));
    gl.attachShader(program, compile(gl.FRAGMENT_SHADER, FRAG_SRC));
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      throw new Error("Program link error:\n" + gl.getProgramInfoLog(program));
    }
    gl.useProgram(program);

    var buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

    UNIFORM_NAMES.forEach(function (n) {
      uniforms[n] = gl.getUniformLocation(program, n);
    });
  }

  function notifyContext(state, error) {
    if (contextCb) contextCb(state, error || null);
  }

  function handleContextLost(event) {
    if (event && event.preventDefault) event.preventDefault();
    contextLost = true;
    suspendFor("context");
    notifyContext("lost");
  }

  function handleContextRestored() {
    try {
      createContext();
      buildPipeline();
      contextLost = false;
      initialized = true;
      lastTick = performance.now();
      resumeFor("context");
      markDirty();
      notifyContext("restored");
    } catch (e) {
      contextLost = true;
      notifyContext("error", e);
    }
  }

  function installContextHandlers() {
    canvas.addEventListener("webglcontextlost", handleContextLost, false);
    canvas.addEventListener("webglcontextrestored", handleContextRestored, false);
  }

  function isContextLost() {
    return contextLost || !gl || (typeof gl.isContextLost === "function" && gl.isContextLost());
  }

  function init(canvasEl, paramsGetter) {
    canvas = canvasEl;
    getParams = paramsGetter;
    contextLost = false;
    createContext();
    installContextHandlers();
    buildPipeline();
    initialized = true;
    lastTick = performance.now();
    if (document.hidden) suspendFor("visibility");
    markDirty();
  }

  function setSize(w, h) {
    if (!initialized || isContextLost()) return;
    if (canvas.width === w && canvas.height === h) return;
    canvas.width = w;
    canvas.height = h;
    gl.viewport(0, 0, w, h);
    markDirty();
  }

  function pushUniforms(P, phase) {
    gl.uniform2f(uniforms.u_res, canvas.width, canvas.height);
    gl.uniform1f(uniforms.u_phase, phase);
    gl.uniform1f(uniforms.u_seed, P.seed);
    gl.uniform1i(uniforms.u_mode, P.mode);

    gl.uniform3fv(uniforms.u_c1, hexToRgb01(P.c1));
    gl.uniform3fv(uniforms.u_c2, hexToRgb01(P.c2));
    gl.uniform3fv(uniforms.u_c3, hexToRgb01(P.c3));
    gl.uniform3fv(uniforms.u_c4, hexToRgb01(P.c4));
    gl.uniform3fv(uniforms.u_bg, hexToRgb01(P.bg));

    gl.uniform1f(uniforms.u_hue, P.hue);
    gl.uniform1f(uniforms.u_sat, P.sat);
    gl.uniform1f(uniforms.u_exposure, P.exposure);
    gl.uniform1f(uniforms.u_contrast, P.contrast);

    gl.uniform1f(uniforms.u_scale, P.scale);
    gl.uniform1f(uniforms.u_complex, P.complex);
    gl.uniform1f(uniforms.u_warp, P.warp);
    gl.uniform1f(uniforms.u_flow, P.flow);
    gl.uniform1f(uniforms.u_stretch, P.stretch);

    gl.uniform1f(uniforms.u_light, P.light);
    gl.uniform1f(uniforms.u_gloss, P.gloss);
    gl.uniform1f(uniforms.u_lightAngle, P.lightAngle);
    gl.uniform1f(uniforms.u_irid, P.irid);
    gl.uniform1f(uniforms.u_glow, P.glow);

    gl.uniform1f(uniforms.u_grain, P.grain);
    gl.uniform1f(uniforms.u_cell, P.cell);
    gl.uniform1f(uniforms.u_lines, P.lines);
    gl.uniform1f(uniforms.u_ca, P.ca);
    gl.uniform1f(uniforms.u_vig, P.vig);
    gl.uniform1f(uniforms.u_soft, P.soft);

    gl.uniform1f(uniforms.u_travel, P.travel);

    gl.uniform1i(uniforms.u_synth, P.synthOn ? 1 : 0);
    gl.uniform1i(uniforms.u_modeB, P.modeB | 0);
    gl.uniform1i(uniforms.u_mixOp, P.mixOp | 0);
    gl.uniform1f(uniforms.u_blend, P.blend);

    var g = P.genes || [0,0,0,0, 0,0,0,0, 0,0,0,0];
    gl.uniform1i(uniforms.u_genome, P.genomeOn ? 1 : 0);
    gl.uniform4f(uniforms.u_g1, g[0], g[1], g[2], g[3]);
    gl.uniform4f(uniforms.u_g2, g[4], g[5], g[6], g[7]);
    gl.uniform4f(uniforms.u_g3, g[8], g[9], g[10], g[11]);
  }

  function renderAt(phase) {
    if (!initialized || isContextLost()) return;
    var P = getParams();
    pushUniforms(P, phase);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    dirty = false;
  }

  function currentPhase() {
    var P = getParams();
    return (loopT / P.loop) % 1;
  }

  function tick(now) {
    frameHandle = 0;
    if (isSuspended()) return;
    if (!playing && !dirty) return;

    var dt = Math.min((now - lastTick) / 1000, 0.1);
    lastTick = now;

    if (playing) {
      var P = getParams();
      loopT = (loopT + dt) % P.loop;
      dirty = true;
      if (playing && targetFrameIntervalMs && lastDrawAt && now - lastDrawAt < targetFrameIntervalMs) {
        reportFrameTime(dt, false);
        scheduleFrame(targetFrameIntervalMs - (now - lastDrawAt));
        return;
      }
    }
    if (dirty) {
      try {
        renderAt(currentPhase());
        lastDrawAt = now;
      } catch (e) {
        if (isContextLost()) {
          handleContextLost();
          return;
        }
        throw e;
      }
    }

    reportFrameTime(dt, true);
    scheduleFrame();
  }

  function reportFrameTime(dt, rendered) {
    fpsAcc += dt;
    if (rendered) fpsN++;
    if (fpsAcc >= 0.5) {
      fps = Math.round(fpsN / fpsAcc);
      fpsAcc = 0; fpsN = 0;
      if (fpsCb) fpsCb(fps);
    }
  }

  function cancelScheduledFrame() {
    if (frameHandle) {
      cancelAnimationFrame(frameHandle);
      frameHandle = 0;
    }
    if (frameTimer) {
      clearTimeout(frameTimer);
      frameTimer = 0;
    }
  }

  function scheduleFrame(delayMs) {
    if (!initialized) return;
    if (isContextLost()) return;
    if (frameHandle || frameTimer || isSuspended()) return;
    if (!playing && !dirty) return;
    if (delayMs && delayMs > 1) {
      frameTimer = setTimeout(function () {
        frameTimer = 0;
        scheduleFrame();
      }, Math.min(delayMs, targetFrameIntervalMs || delayMs));
      return;
    }
    frameHandle = requestAnimationFrame(tick);
  }

  function markDirty() {
    if (!initialized || isContextLost()) return;
    dirty = true;
    if (frameTimer) {
      clearTimeout(frameTimer);
      frameTimer = 0;
    }
    scheduleFrame();
  }

  function isSuspended() {
    return suspensionCount > 0;
  }

  function resetFps() {
    fpsAcc = 0;
    fpsN = 0;
  }

  function setTargetFps(fps) {
    var safeFps = Math.max(0, Number(fps) || 0);
    targetFrameIntervalMs = safeFps > 0 ? 1000 / safeFps : 0;
    lastDrawAt = 0;
    resetFps();
    cancelScheduledFrame();
    scheduleFrame();
  }

  function suspendFor(reason) {
    reason = reason || "manual";
    if (!suspensionReasons[reason]) {
      suspensionReasons[reason] = true;
      suspensionCount++;
    }
    resetFps();
    cancelScheduledFrame();
  }

  function resumeFor(reason) {
    reason = reason || "manual";
    if (suspensionReasons[reason]) {
      delete suspensionReasons[reason];
      suspensionCount = Math.max(0, suspensionCount - 1);
    }
    if (isSuspended()) return;
    resetFps();
    lastTick = performance.now();
    markDirty();
  }

  function readPixels() {
    if (!initialized || isContextLost()) throw new Error("WebGL context is unavailable");
    var w = canvas.width, h = canvas.height;
    var buf = new Uint8Array(w * h * 4);
    gl.readPixels(0, 0, w, h, gl.RGBA, gl.UNSIGNED_BYTE, buf);
    /* flip vertically: GL origin is bottom-left */
    var flipped = new Uint8Array(w * h * 4);
    var row = w * 4;
    for (var y = 0; y < h; y++) {
      flipped.set(buf.subarray(y * row, (y + 1) * row), (h - 1 - y) * row);
    }
    return flipped;
  }

  return {
    init: init,
    setSize: setSize,
    renderAt: renderAt,
    readPixels: readPixels,
    currentPhase: currentPhase,
    resetTime: function () { loopT = 0; markDirty(); },
    setLoopTime: function (t) { loopT = t; markDirty(); },
    suspendFor: suspendFor,
    resumeFor: resumeFor,
    suspend: function () { suspendFor("manual"); },
    resume: function () { resumeFor("manual"); },
    setPlaying: function (v) {
      playing = !!v;
      lastTick = performance.now();
      markDirty();
    },
    isPlaying: function () { return playing; },
    markDirty: markDirty,
    setTargetFps: setTargetFps,
    onFps: function (cb) { fpsCb = cb; },
    onContextChange: function (cb) { contextCb = cb; },
    isContextLost: isContextLost,
    canvas: function () { return canvas; },
    size: function () { return [canvas.width, canvas.height]; }
  };
})();

document.addEventListener("visibilitychange", function () {
  if (document.hidden) {
    Engine.suspendFor("visibility");
  } else {
    Engine.resumeFor("visibility");
  }
});
