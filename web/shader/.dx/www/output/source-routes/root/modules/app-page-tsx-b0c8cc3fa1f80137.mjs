
export const dxSourceText = "export const metadata = {\n  title: \"DX Shader, the generative shader studio\",\n  description:\n    \"Create looping abstract shader art with WebGL2, curated controls, share codes, and deterministic exports.\",\n} as const;\n\nexport default function HomePage() {\n  return (\n    <main className=\"dx-shader-root\" data-dx-route=\"/\" data-dx-template=\"dx-shader\">\n      <header className=\"topbar\">\n        <div className=\"brand\">\n          <svg className=\"brand-mark\" viewBox=\"0 0 20 20\" aria-hidden=\"true\">\n            <path\n              d=\"M10 1 L19 10 L10 19 L1 10 Z\"\n              fill=\"none\"\n              stroke=\"currentColor\"\n              strokeWidth=\"1.4\"\n            />\n            <path\n              d=\"M10 5.5 L14.5 10 L10 14.5 L5.5 10 Z\"\n              fill=\"currentColor\"\n            />\n          </svg>\n          <span className=\"brand-name\">DX Shader</span>\n          <span className=\"brand-sub\">generative shader studio</span>\n        </div>\n\n        <div className=\"topbar-center\">\n          <div\n            className=\"segmented\"\n            id=\"dx-shader-aspect-control\"\n            role=\"group\"\n            aria-label=\"Canvas aspect ratio\"\n          />\n        </div>\n\n        <div className=\"topbar-actions\">\n          <button\n            className=\"btn btn-primary\"\n            id=\"btn-random\"\n            title=\"Randomize everything (R)\"\n            type=\"button\"\n          >\n            <svg viewBox=\"0 0 16 16\" aria-hidden=\"true\">\n              <rect\n                x=\"1.5\"\n                y=\"1.5\"\n                width=\"13\"\n                height=\"13\"\n                rx=\"3\"\n                fill=\"none\"\n                stroke=\"currentColor\"\n                strokeWidth=\"1.5\"\n              />\n              <circle cx=\"5.4\" cy=\"5.4\" r=\"1.25\" fill=\"currentColor\" />\n              <circle cx=\"10.6\" cy=\"10.6\" r=\"1.25\" fill=\"currentColor\" />\n              <circle cx=\"10.6\" cy=\"5.4\" r=\"1.25\" fill=\"currentColor\" />\n              <circle cx=\"5.4\" cy=\"10.6\" r=\"1.25\" fill=\"currentColor\" />\n            </svg>\n            Randomize\n          </button>\n          <div className=\"topbar-divider\" />\n          <button\n            className=\"btn\"\n            id=\"btn-export-png\"\n            title=\"Save still image (S)\"\n            type=\"button\"\n          >\n            <svg viewBox=\"0 0 16 16\" aria-hidden=\"true\">\n              <rect\n                x=\"1.5\"\n                y=\"1.5\"\n                width=\"13\"\n                height=\"13\"\n                rx=\"2\"\n                fill=\"none\"\n                stroke=\"currentColor\"\n                strokeWidth=\"1.5\"\n              />\n              <circle cx=\"5.5\" cy=\"5.5\" r=\"1.5\" fill=\"currentColor\" />\n              <path\n                d=\"M2 12 L6 8 L9 11 L11.5 8.5 L14 11\"\n                fill=\"none\"\n                stroke=\"currentColor\"\n                strokeWidth=\"1.5\"\n              />\n            </svg>\n            Image\n          </button>\n          <button\n            className=\"btn\"\n            id=\"btn-export-video\"\n            title=\"Record looping video\"\n            type=\"button\"\n          >\n            <svg viewBox=\"0 0 16 16\" aria-hidden=\"true\">\n              <rect\n                x=\"1.5\"\n                y=\"3.5\"\n                width=\"9\"\n                height=\"9\"\n                rx=\"2\"\n                fill=\"none\"\n                stroke=\"currentColor\"\n                strokeWidth=\"1.5\"\n              />\n              <path\n                d=\"M10.5 7 L14.5 4.5 V11.5 L10.5 9\"\n                fill=\"currentColor\"\n              />\n            </svg>\n            Video\n          </button>\n          <button\n            className=\"btn\"\n            id=\"btn-export-gif\"\n            title=\"Render seamless looping GIF\"\n            type=\"button\"\n          >\n            <svg viewBox=\"0 0 16 16\" aria-hidden=\"true\">\n              <path\n                d=\"M13.5 8 a5.5 5.5 0 1 1 -1.6 -3.9\"\n                fill=\"none\"\n                stroke=\"currentColor\"\n                strokeWidth=\"1.5\"\n              />\n              <path\n                d=\"M13.8 1.6 V4.4 H11\"\n                fill=\"none\"\n                stroke=\"currentColor\"\n                strokeWidth=\"1.5\"\n              />\n            </svg>\n            GIF\n          </button>\n          <button\n            className=\"btn\"\n            id=\"btn-set\"\n            title=\"Generate a consistent set of variations\"\n            type=\"button\"\n          >\n            <svg viewBox=\"0 0 16 16\" aria-hidden=\"true\">\n              <rect\n                x=\"1.5\"\n                y=\"1.5\"\n                width=\"5.5\"\n                height=\"5.5\"\n                rx=\"1.5\"\n                fill=\"none\"\n                stroke=\"currentColor\"\n                strokeWidth=\"1.4\"\n              />\n              <rect\n                x=\"9\"\n                y=\"1.5\"\n                width=\"5.5\"\n                height=\"5.5\"\n                rx=\"1.5\"\n                fill=\"none\"\n                stroke=\"currentColor\"\n                strokeWidth=\"1.4\"\n              />\n              <rect\n                x=\"1.5\"\n                y=\"9\"\n                width=\"5.5\"\n                height=\"5.5\"\n                rx=\"1.5\"\n                fill=\"none\"\n                stroke=\"currentColor\"\n                strokeWidth=\"1.4\"\n              />\n              <rect\n                x=\"9\"\n                y=\"9\"\n                width=\"5.5\"\n                height=\"5.5\"\n                rx=\"1.5\"\n                fill=\"none\"\n                stroke=\"currentColor\"\n                strokeWidth=\"1.4\"\n              />\n            </svg>\n            Set\n          </button>\n          <div className=\"topbar-divider\" />\n          <a className=\"btn btn-ghost\" href=\"/dx-shader/docs.html\" title=\"Documentation\">\n            <svg viewBox=\"0 0 16 16\" aria-hidden=\"true\">\n              <path\n                d=\"M3 2 H10 L13 5 V14 H3 Z\"\n                fill=\"none\"\n                stroke=\"currentColor\"\n                strokeWidth=\"1.4\"\n              />\n              <path\n                d=\"M5.5 7.5 H10.5 M5.5 10 H10.5\"\n                stroke=\"currentColor\"\n                strokeWidth=\"1.2\"\n              />\n            </svg>\n            Docs\n          </a>\n        </div>\n      </header>\n\n      <div className=\"app\">\n        <section className=\"stage\" id=\"stage\">\n          <div\n            className=\"shader-loader\"\n            data-loader-mode=\"auto\"\n            data-loader-state=\"preparing\"\n            id=\"shader-boot-loader\"\n            role=\"status\"\n            aria-live=\"polite\"\n          >\n            <div className=\"shader-loader-panel\">\n              <div\n                className=\"shader-loader-mark\"\n                data-dx-icon-search=\"pack:dx lucide:loader-circle\"\n              >\n                <dx-icon\n                  className=\"shader-loader-dx\"\n                  name=\"pack:dx\"\n                  aria-hidden=\"true\"\n                />\n                <dx-icon\n                  className=\"shader-loader-spinner\"\n                  data-lucide-icon=\"loader-circle\"\n                  name=\"lucide:loader-circle\"\n                  aria-hidden=\"true\"\n                />\n              </div>\n              <div className=\"shader-loader-copy\">\n                <span className=\"shader-loader-title\">Preparing shader engine</span>\n                <span className=\"shader-loader-detail\" id=\"shader-boot-status\">\n                  Checking GPU readiness\n                </span>\n              </div>\n              <button\n                className=\"shader-loader-action\"\n                hidden=\"hidden\"\n                id=\"shader-boot-start\"\n                type=\"button\"\n              >\n                Start shader\n              </button>\n              <div className=\"shader-loader-shimmer\" aria-hidden=\"true\" />\n            </div>\n          </div>\n          <div className=\"canvas-frame\" id=\"canvas-frame\">\n            <div className=\"canvas-shell\" id=\"canvas-shell\">\n              <div\n                className=\"shader-static-fallback\"\n                id=\"shader-static-fallback\"\n                aria-hidden=\"true\"\n              >\n                <span className=\"shader-static-title\">DX Shader</span>\n                <span className=\"shader-static-detail\">Safe shader preview</span>\n              </div>\n              <noscript>\n                <div className=\"shader-static-fallback shader-static-fallback-noscript\">\n                  <span className=\"shader-static-title\">DX Shader</span>\n                  <span className=\"shader-static-detail\">\n                    JavaScript is required to start the live shader.\n                  </span>\n                </div>\n              </noscript>\n              <canvas id=\"view\"></canvas>\n            </div>\n          </div>\n          <div className=\"stage-meta\">\n            <div className=\"meta-left\">\n              <button\n                className=\"icon-btn\"\n                id=\"btn-play\"\n                title=\"Play / pause (Space)\"\n                type=\"button\"\n              >\n                <svg id=\"icon-pause\" viewBox=\"0 0 14 14\">\n                  <rect\n                    x=\"2.5\"\n                    y=\"2\"\n                    width=\"3\"\n                    height=\"10\"\n                    rx=\"1\"\n                    fill=\"currentColor\"\n                  />\n                  <rect\n                    x=\"8.5\"\n                    y=\"2\"\n                    width=\"3\"\n                    height=\"10\"\n                    rx=\"1\"\n                    fill=\"currentColor\"\n                  />\n                </svg>\n                <svg id=\"icon-play\" viewBox=\"0 0 14 14\">\n                  <path d=\"M3.5 2 L12 7 L3.5 12 Z\" fill=\"currentColor\" />\n                </svg>\n              </button>\n              <span className=\"meta-item mono\" id=\"meta-mode\">\n                ...\n              </span>\n              <span className=\"meta-sep\" />\n              <span className=\"meta-item mono dim\" id=\"meta-seed\">\n                seed 0000\n              </span>\n            </div>\n            <div className=\"meta-right\">\n              <span className=\"meta-item mono dim\" id=\"meta-loop\">\n                4.0s loop\n              </span>\n              <span className=\"meta-sep\" />\n              <span className=\"meta-item mono dim\" id=\"meta-res\">\n                0x0\n              </span>\n              <span className=\"meta-sep\" />\n              <span className=\"meta-item mono dim\" id=\"meta-fps\">\n                60 fps\n              </span>\n            </div>\n          </div>\n        </section>\n\n        <aside className=\"rail\" id=\"rail\" />\n      </div>\n\n      <div className=\"overlay is-hidden\" id=\"overlay\" hidden=\"hidden\">\n        <div className=\"overlay-card\">\n          <div className=\"overlay-title\" id=\"overlay-title\">\n            Rendering\n          </div>\n          <div className=\"overlay-detail mono\" id=\"overlay-detail\">\n            ...\n          </div>\n          <div className=\"progress\">\n            <div className=\"progress-fill\" id=\"overlay-bar\" />\n          </div>\n          <button className=\"btn overlay-cancel\" id=\"overlay-cancel\" type=\"button\">\n            Cancel\n          </button>\n        </div>\n      </div>\n\n      <div className=\"toast mono is-hidden\" id=\"toast\" hidden=\"hidden\"></div>\n\n      <script defer src=\"/dx-shader/js/palettes.js\"></script>\n      <script defer src=\"/dx-shader/js/shaders.js\"></script>\n      <script defer src=\"/dx-shader/js/engine.js\"></script>\n      <script defer src=\"/dx-shader/js/fx.js\"></script>\n      <script defer src=\"/dx-shader/js/exporter.js\"></script>\n      <script defer src=\"/dx-shader/js/modals.js\"></script>\n      <script defer src=\"/dx-shader/js/ui.js\"></script>\n      <script defer src=\"/dx-shader/js/main.js\"></script>\n    </main>\n  );\n}\n";
export const dxSourceModule = Object.freeze({
  "source_path": "app/page.tsx",
  "chunk_output": ".dx/www/output/source-routes/root/modules/app-page-tsx-b0c8cc3fa1f80137.mjs",
  "kind": "tsx",
  "hash": "b0c8cc3fa1f80137",
  "dependencies": [],
  "browser_executable": true,
  "source_transformed": false,
  "transform_kind": "metadata-only",
  "runtime_exports": [],
  "ecmascript_analysis": {
    "schema": "dx.ecmascript.analysis",
    "schema_revision": 1,
    "source_path": "app/page.tsx",
    "source_kind": "tsx",
    "parser_backend": "oxc-parser",
    "diagnostics": 0,
    "compatibility_reference": {
      "upstream_crates": [
        "turbopack-ecmascript"
      ],
      "reference_only": true,
      "runtime_build_adoption": false,
      "public_runtime_dependency": false,
      "vendor_root": "vendor/next-rust",
      "vendor_commit": "f3f56ecec2f3f8cefa0f0a1323ea406740251d5c",
      "next_transform_references": [
        "next-custom-transforms::track_dynamic_imports",
        "next-custom-transforms::react_server_components"
      ],
      "copied_code": false
    },
    "output_model": {
      "contract": "dx.www.moduleGraph",
      "compiler_owns_output": true,
      "public_architecture": "DX-owned source graph analysis"
    },
    "runtime_boundaries": {
      "next_runtime_required": false,
      "react_runtime_required": false,
      "rsc_required": false,
      "node_modules_required": false
    },
    "directives": [],
    "static_imports": [],
    "dynamic_imports": [],
    "unresolved_dynamic_imports": [],
    "unsupported_dynamic_imports": [],
    "dynamic_import_analysis": {
      "status": "none-observed",
      "static_count": 0,
      "unresolved_count": 0,
      "unsupported_count": 0,
      "boundary": "source-owned dynamic import analysis; static specifiers become evidence, expressions remain unresolved, and unsupported call forms stay as adapter-boundary receipts"
    },
    "export_names": [
      "HomePage",
      "metadata"
    ],
    "jsx": true,
    "top_level_await": false,
    "full_nextjs_parity": false,
    "analysis_boundary": "Uses vendored Turbopack ECMAScript and selected Next transform behavior as compatibility references while emitting DX-owned source graph receipts."
  },
  "node_modules_required": false
});
export const dxRuntimeModule = Object.freeze({
  transformed: false,
  transformKind: "metadata-only",
  exportNames: []
});
export const dxRuntimeExports = Object.freeze({});
export const dxLinkedDependencies = Object.freeze([]);
export default dxSourceModule;
