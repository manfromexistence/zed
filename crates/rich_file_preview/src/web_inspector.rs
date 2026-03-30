use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DomRectSnapshot {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CapturedElement {
    pub selector: String,
    pub dom_path: String,
    pub tag_name: String,
    pub text: String,
    pub html: String,
    pub css: String,
    pub inline_css: String,
    pub url: String,
    #[serde(default)]
    pub auto_send_to_ai: bool,
    #[serde(default)]
    pub rect: DomRectSnapshot,
}

impl CapturedElement {
    pub fn summary(&self) -> String {
        format!("{} @ {}", self.selector, self.url)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WebIpcMessage {
    Capability {
        webgpu: bool,
        url: String,
        title: Option<String>,
    },
    Navigation {
        url: String,
        title: Option<String>,
    },
    Hover(CapturedElement),
    Selection(CapturedElement),
    Log {
        message: String,
    },
}

pub const INIT_SCRIPT: &str = r#"
(() => {
  if (window.__zedPreview) {
    return;
  }

  const send = payload => {
    try {
      window.ipc?.postMessage(JSON.stringify(payload));
    } catch (_) {}
  };

  const limitText = (value, limit) => {
    if (!value) return '';
    const text = String(value);
    return text.length > limit ? text.slice(0, limit) : text;
  };

  const selectorFor = element => {
    if (!(element instanceof Element)) return 'unknown';
    const parts = [];
    let current = element;
    while (current && current.nodeType === Node.ELEMENT_NODE && parts.length < 5) {
      let part = current.tagName.toLowerCase();
      if (current.id) {
        part += `#${current.id}`;
        parts.unshift(part);
        break;
      }
      const classes = Array.from(current.classList || []).slice(0, 2);
      if (classes.length) {
        part += `.${classes.join('.')}`;
      }
      const parent = current.parentElement;
      if (parent) {
        const siblings = Array.from(parent.children).filter(child => child.tagName === current.tagName);
        if (siblings.length > 1) {
          part += `:nth-of-type(${siblings.indexOf(current) + 1})`;
        }
      }
      parts.unshift(part);
      current = current.parentElement;
    }
    return parts.join(' > ') || element.tagName.toLowerCase();
  };

  const domPathFor = element => {
    const nodes = [];
    let current = element;
    while (current && current.nodeType === Node.ELEMENT_NODE && nodes.length < 10) {
      nodes.unshift(current.tagName.toLowerCase());
      current = current.parentElement;
    }
    return nodes.join('/');
  };

  const cssText = element => {
    const style = getComputedStyle(element);
    const names = Array.from(style).slice(0, 64);
    return names
      .map(name => `${name}: ${style.getPropertyValue(name)};`)
      .join('\n');
  };

  const rectFor = element => {
    const rect = element.getBoundingClientRect();
    return {
      x: rect.x,
      y: rect.y,
      width: rect.width,
      height: rect.height,
    };
  };

  const serializeElement = (element, autoSendToAi) => ({
    selector: selectorFor(element),
    dom_path: domPathFor(element),
    tag_name: element.tagName.toLowerCase(),
    text: limitText((element.innerText || element.textContent || '').trim(), 2000),
    html: limitText(element.outerHTML || '', 8000),
    css: cssText(element),
    inline_css: element.getAttribute('style') || '',
    url: window.location.href,
    auto_send_to_ai: !!autoSendToAi,
    rect: rectFor(element),
  });

  const state = {
    armed: false,
    autoSendToAi: false,
    hovered: null,
    selected: null,
    selectedId: null,
    overlay: null,
    styleNode: null,
    lastHoverAt: 0,
  };

  const ensureOverlay = () => {
    if (state.overlay) return state.overlay;
    const overlay = document.createElement('div');
    overlay.id = 'zed-preview-inspector-overlay';
    overlay.style.position = 'fixed';
    overlay.style.pointerEvents = 'none';
    overlay.style.zIndex = '2147483646';
    overlay.style.border = '1.5px solid rgba(73, 145, 255, 0.95)';
    overlay.style.background = 'rgba(73, 145, 255, 0.08)';
    overlay.style.boxShadow = '0 0 0 1px rgba(73,145,255,0.2), 0 12px 32px rgba(8,16,32,0.28)';
    overlay.style.borderRadius = '8px';
    overlay.style.display = 'none';
    document.documentElement.appendChild(overlay);
    state.overlay = overlay;
    return overlay;
  };

  const ensureStyleNode = () => {
    if (state.styleNode) return state.styleNode;
    const styleNode = document.createElement('style');
    styleNode.id = 'zed-preview-style-overrides';
    document.documentElement.appendChild(styleNode);
    state.styleNode = styleNode;
    return styleNode;
  };

  const updateOverlay = element => {
    const overlay = ensureOverlay();
    if (!element) {
      overlay.style.display = 'none';
      return;
    }
    const rect = element.getBoundingClientRect();
    overlay.style.display = 'block';
    overlay.style.left = `${rect.left}px`;
    overlay.style.top = `${rect.top}px`;
    overlay.style.width = `${Math.max(rect.width, 1)}px`;
    overlay.style.height = `${Math.max(rect.height, 1)}px`;
  };

  const setSelected = element => {
    if (state.selected && state.selected !== element) {
      state.selected.removeAttribute('data-zed-preview-target');
    }
    state.selected = element;
    state.selectedId = `zed-preview-${Date.now()}`;
    element.setAttribute('data-zed-preview-target', state.selectedId);
  };

  const disarm = () => {
    state.armed = false;
    state.autoSendToAi = false;
    document.removeEventListener('mousemove', onMouseMove, true);
    document.removeEventListener('click', onClick, true);
    document.removeEventListener('keydown', onKeyDown, true);
    updateOverlay(null);
  };

  const maybeSendHover = element => {
    const now = Date.now();
    if (now - state.lastHoverAt < 60) {
      return;
    }
    state.lastHoverAt = now;
    send({ kind: 'hover', ...serializeElement(element, false) });
  };

  const onMouseMove = event => {
    if (!state.armed) return;
    const element = event.target instanceof Element ? event.target.closest('*') : null;
    if (!element || element === state.overlay) return;
    state.hovered = element;
    updateOverlay(element);
    maybeSendHover(element);
  };

  const onClick = event => {
    if (!state.armed) return;
    const element = event.target instanceof Element ? event.target.closest('*') : null;
    if (!element || element === state.overlay) return;
    event.preventDefault();
    event.stopPropagation();
    setSelected(element);
    send({ kind: 'selection', ...serializeElement(element, state.autoSendToAi) });
    disarm();
  };

  const onKeyDown = event => {
    if (event.key === 'Escape') {
      disarm();
    }
  };

  const reportNavigation = () => {
    send({ kind: 'navigation', url: window.location.href, title: document.title || '' });
  };

  const patchHistory = () => {
    const originalPushState = history.pushState;
    const originalReplaceState = history.replaceState;
    history.pushState = function(...args) {
      const result = originalPushState.apply(this, args);
      queueMicrotask(reportNavigation);
      return result;
    };
    history.replaceState = function(...args) {
      const result = originalReplaceState.apply(this, args);
      queueMicrotask(reportNavigation);
      return result;
    };
    window.addEventListener('hashchange', reportNavigation);
    window.addEventListener('popstate', reportNavigation);
  };

  patchHistory();

  window.__zedPreview = {
    armInspect(autoSendToAi) {
      disarm();
      state.armed = true;
      state.autoSendToAi = !!autoSendToAi;
      document.addEventListener('mousemove', onMouseMove, true);
      document.addEventListener('click', onClick, true);
      document.addEventListener('keydown', onKeyDown, true);
    },
    cancelInspect() {
      disarm();
    },
    applySelectedCss(css) {
      if (!state.selected) return false;
      setSelected(state.selected);
      ensureStyleNode().textContent = `[data-zed-preview-target="${state.selectedId}"] { ${css} }`;
      return true;
    },
    clearSelectedCss() {
      if (state.styleNode) {
        state.styleNode.textContent = '';
      }
      if (state.selected) {
        state.selected.removeAttribute('data-zed-preview-target');
      }
      state.selected = null;
      state.selectedId = null;
      return true;
    },
  };

  const emitCapability = () => {
    send({
      kind: 'capability',
      webgpu: !!navigator.gpu,
      url: window.location.href,
      title: document.title || '',
    });
    reportNavigation();
  };

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', emitCapability, { once: true });
  } else {
    emitCapability();
  }
})();
"#;

pub fn arm_inspector_script(auto_send_to_ai: bool) -> String {
    format!(
        "window.__zedPreview?.armInspect({});",
        if auto_send_to_ai { "true" } else { "false" }
    )
}

pub fn cancel_inspector_script() -> &'static str {
    "window.__zedPreview?.cancelInspect();"
}

pub fn apply_css_script(css: &str) -> String {
    let escaped = serde_json::to_string(css).unwrap_or_else(|_| "\"\"".to_string());
    format!("window.__zedPreview?.applySelectedCss({escaped});")
}

pub fn clear_css_script() -> &'static str {
    "window.__zedPreview?.clearSelectedCss();"
}
