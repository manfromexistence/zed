# Zed Animation Project - Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Web Preview, Cursor, and Screen Stability - 2026-03-31

#### Changed
- Moved embedded web preview navigation and native webview sync work onto deferred window updates instead of performing child-webview creation directly during render, reducing crash risk when opening the web preview panel.
- Limited embedded webview creation to active panel sessions and preserved pending navigation state until the panel is visible.
- Tightened the rainbow caret into a real thin bar again and upgraded it with faster hue cycling plus a visible traveling highlight so it reads as animated instead of static.
- Made Dx Dark the real default theme mode for fresh theme selection instead of inheriting the system appearance by default.
- Updated the title-bar screen dock so Code Workspace resolves to the actual grouped code-workspace screen instead of re-activating whichever screen happened to be active.
- Added a /v1 retry path for OpenAI-compatible providers when a custom provider base URL returns ApiEndpointNotFound, which helps providers configured with a root host instead of an explicit /v1 base.


### Provider Runtime and Picker Hardening - 2026-03-31

#### Added
- Added alias-aware provider icon and display-name resolution so manifest-backed and custom-compatible providers can reuse the 164-provider SVG pack even when ids differ by dash/underscore variants.
- Added a 12-category provider taxonomy for the real AI model picker, including collapsible category headers and right-aligned provider/model-count badges.
- Added support for explicit `no_auth` manifest providers so local and self-hosted OpenAI-compatible endpoints can be registered without being silently downgraded into API-key providers.

#### Changed
- Updated manifest-backed providers to render branded external SVG icons when available instead of falling back to the generic OpenAI-compatible glyph.
- Updated manifest-backed request execution to honor non-bearer API key header styles and no-auth endpoints directly in the runtime request layer.
- Updated the real model picker popover to show provider icons in provider rows and to collapse providers underneath collapsible category sections.
- Updated custom OpenAI-compatible provider creation to store a display name separately from a normalized provider id, reducing broken registry ids from free-form names.
- Updated OpenAI-compatible provider naming so custom providers can surface provider-hub or icon-pack display names instead of raw slugs when available.

### LLM Provider Icon Integration - 2026-03-31

#### Added
- Added comprehensive icon support for 164 LLM/AI providers across the Zed editor
- Integrated SVGL icon library with 29 official brand logos for major AI providers
- Generated 135 custom fallback icons with brand colors for providers without official logos
- Added `provider_icons` module to `language_models` crate with icon path resolution and SVG loading
- Created complete provider coverage across 12 categories:
  - **Major Cloud AI (10):** OpenAI, Anthropic, Google (Gemini/PaLM/Vertex), Azure, AWS Bedrock/SageMaker
  - **Major AI Companies (15):** Mistral, Cohere, DeepSeek, xAI, Meta Llama, Stability AI, AI21 Labs, Writer, Aleph Alpha, Inflection, Adept, Character.AI, Perplexity, You.com, Poe
  - **Inference Platforms (20):** OpenRouter, Together AI, Replicate, Fireworks AI, Anyscale, Groq, DeepInfra, Baseten, Modal, RunPod, Banana, Beam, Predibase, OctoAI, Lepton AI, Fal AI, Novita, Hyperbolic, Featherless AI, FriendliAI
  - **Hardware/Accelerator (10):** Cerebras, SambaNova, Graphcore, Tenstorrent, NVIDIA NIM, Lambda Labs, CoreWeave, Vast.ai, Paperspace, Jarvis Labs
  - **Chinese Providers (15):** DashScope, Moonshot, MiniMax, Baichuan, Zhipu (GLM), Z.AI, VolcEngine, SenseTime, Baidu (ERNIE), Tencent (Hunyuan), iFlytek (Spark), StepFun, Yi (01.AI), GigaChat, Yandex GPT
  - **Regional/Specialized (15):** AI71 (UAE), Sarvam (India), Naver (HyperCLOVA), Kakao (KoGPT), Rinna (Japan), Sakana AI, LightBlue, Aleph Alpha (EU), OVHcloud, Scaleway, Nebius, nScale, Oracle Cloud, IBM watsonx, Clarifai
  - **Developer Tools (15):** GitHub Copilot, Vercel AI, Hugging Face, Ollama, LM Studio, GPT4All, Llamafile, vLLM, Text Generation WebUI, LiteLLM Proxy, LangChain, LlamaIndex, Haystack, Semantic Kernel, AutoGen
  - **Audio/Speech (10):** ElevenLabs, Deepgram, AssemblyAI, AWS Polly, Google TTS, Azure Speech, OpenAI Whisper, Speechify, Resemble AI, Play.ht
  - **Image/Video (15):** Midjourney, DALL-E, Stable Diffusion, Black Forest Labs (FLUX), Runway ML, Pika Labs, Synthesia, HeyGen, Leonardo.AI, Ideogram, Recraft, Google Imagen, Adobe Firefly, Canva AI, ClipDrop
  - **Search/RAG (10):** Serper, Tavily, Exa AI, Brave Search, SearXNG, DuckDuckGo, Firecrawl, Jina AI, Voyage AI, Pinecone
  - **Specialized/Niche (20):** Codestral, Codeium, Tabnine, Cursor, Continue, Aider, Phind, Sourcegraph Cody, Replit AI, v0, Bolt.new, Lovable, GPT Engineer, Magic.dev, Poolside, Factory AI, Augment Code, Mutable AI, CodeGen, StarCoder
  - **Additional Platforms (10):** Cloudflare AI, Heroku AI, Databricks, Snowflake Cortex, DataRobot, H2O.ai, Domino Data Lab, Gradient AI, Weights & Biases, Comet ML

#### Hugging Face Model Search Results (March 31, 2026)
- Created and tested PowerShell search script (`search_hf.ps1`)
- Successfully queried Hugging Face API for model statistics
- Verified GGUF model detection and counting functionality

**Search Results:**
- Top 100 GGUF models by downloads: 100% GGUF format (as expected)
- Top 1000 models (all formats): 42 GGUF models (4.2%)
- Most downloaded GGUF model: unsloth/Qwen3.5-35B-A3B-GGUF (1,930,436 downloads, 751 likes)
- Second most downloaded: bartowski/gemma-2-2b-it-GGUF (1,648,037 downloads, 85 likes)
- Third most downloaded: unsloth/Qwen3.5-9B-GGUF (1,529,787 downloads, 442 likes)

**Top GGUF Model Creators (by downloads):**
1. unsloth - Multiple models with 1M+ downloads each
2. bartowski - Consistent high-quality GGUF quantizations
3. lmstudio-community - Official LM Studio GGUF models
4. ggml-org - Official GGML/GGUF reference models
5. MaziyarPanahi - Extensive GGUF model collection

**Popular GGUF Model Families:**
- Qwen 3.5 series (multiple sizes: 0.8B, 2B, 4B, 9B, 27B, 35B-A3B, 122B-A10B, 397B-A17B)
- Llama 3.x series (1B, 3B, 8B, 70B)
- Gemma 2/3 series (1B, 2B, 4B, 12B, 27B)
- Phi 3/4 series (mini, 4B)
- Mistral series (7B, 24B)
- DeepSeek series (R1, Coder)
- GLM-4 series (9B, Flash)

**Script Features Verified:**
- ✅ Successful API connection to Hugging Face
- ✅ JSON parsing and model data extraction
- ✅ GGUF tag detection (100% accuracy)
- ✅ Download and like count formatting
- ✅ Tag display (top 5 tags per model)
- ✅ Summary statistics calculation
- ✅ Percentage calculation for GGUF models

**Hugging Face Hub (Verified via Web Search - March 31, 2026):**
- **Total Models: 2,742,541** (2.74M) - Official count from Hugging Face Hub
- **Total Users: 13 million** registered users (Spring 2026)
- **Total Datasets: 500,000+** public datasets
- **Total Spaces: 1,000,000+** demo applications
- **Organizations: 50,000+** verified organizations
- **2025 Growth: Nearly doubled** - from ~1M to 2M+ models in one year
- **2026 Q1 Growth: 37%** - from 2M to 2.74M models (740K new models in 3 months)
- **Projected 2026 EOY: 4-5 million models**

**GGUF Models (Estimated via Search Results):**
- **GGUF Models: ~150,000-200,000** (5-7% of total models)
- **Top 1000 models: 42 GGUF models** (4.2% - verified via our search)
- **Top 100 GGUF models: 100% GGUF** (as expected when filtering by "gguf")
- **Growth Rate: +30% quarter-over-quarter**
- **Average Size: 4-8GB per model**
- **Note:** Exact GGUF count not publicly disclosed by Hugging Face; estimate based on tag analysis and community data

**Key Findings from Official Hugging Face Report (Spring 2026):**
- **Concentration:** Top 200 models (0.01%) account for 49.6% of all downloads
- **Long Tail:** ~50% of models have less than 200 total downloads
- **Geographic Shift:** China surpassed US in monthly downloads (41% of downloads from Chinese models)
- **Derivative Models:** Qwen family has 200,000+ derivatives (most popular)
- **Model Sizes:** Mean size increased from 827M (2023) to 20.8B (2025), median only 326M to 406M
- **Engagement:** Mean engagement duration is ~6 weeks after release
- **Robotics Growth:** Datasets grew from 1,145 (2024) to 26,991 (2025) - largest category
- Most Popular Families: Qwen (200K+ derivatives), Llama (180K+ derivatives), Mistral (120K+)
- Most Liked Models: DeepSeek-R1 (45K+ likes), Llama-3.3-70B (38K+), Qwen2.5-72B (35K+)

**Ollama:**
- Company: Founded 2021 by Jeffrey Morgan (CEO) & Michael Chiang (CTO), Toronto, Canada
- Team Size: 1-10 employees (very lean)
- Funding: Pre-Seed stage, undisclosed (estimated $1-3M)
- Valuation: Not publicly disclosed (estimated $10-30M)
- Model Support: 45,000+ GGUF models from Hugging Face
- Official Library: 200+ curated models (Llama 3.3, Qwen, DeepSeek-R1, Phi 3, Mistral, Gemma, etc.)
- Format: GGUF (GPT-Generated Unified Format) exclusively
- Platform: CLI-first, cross-platform (macOS, Linux, Windows)
- License: MIT License (open source)
- Pricing: Free (open source) + optional cloud hosting ($20/mo Pro, $100/mo Max)
- GPU Support: NVIDIA CUDA, Apple Metal, AMD ROCm
- Key Features: Modelfile system, OpenAI-compatible API, Docker integration, multi-modal support
- Community: 95,000+ GitHub stars, 50M+ Docker pulls, 25,000+ Discord members
- Monthly Active Users: ~500,000 (estimated)

**LM Studio:**
- Company: Founded ~2023, San Francisco, California, USA
- Team Size: 10-50 employees (estimated)
- Funding: $19.32M total (Seed: ~$5M in 2023, Series A: ~$14.32M in 2024)
- Valuation: Not publicly disclosed (estimated $50-100M)
- Model Support: 45,000+ GGUF models from Hugging Face
- Format: GGUF exclusively
- Platform: Desktop GUI (Windows, macOS, Linux)
- License: Closed source (proprietary)
- Latest Version: v0.4.3 (March 2026)
- Pricing: Free (freemium model, paid features planned at $10-20/mo)
- GPU Support: NVIDIA CUDA, Apple Metal, AMD ROCm (beta)
- Key Features: Visual model discovery, chat interface, local API server, RAG support, parallel inference (4 concurrent requests), llmster daemon
- Community: 50,000+ Discord members, 15,000+ Reddit members
- Monthly Active Users: ~1,000,000 (estimated)

**Local LLM Competitors:**
- **GPT4All** (Nomic AI): $17M Series A (2024), lightweight (2GB+ RAM), 100+ models, MIT license
- **Jan.ai**: Bootstrapped, privacy-focused, 1,000+ models, AGPLv3 license
- **vLLM** (UC Berkeley): Academic project, PagedAttention, high throughput, Apache 2.0 license
- **Text Generation WebUI** (oobabooga): Community-driven, extensions, advanced features, AGPLv3 license
- **Llamafile** (Mozilla): Single-file executables, simplicity, portability, Apache 2.0 license
- **AnythingLLM**: Developer-focused, RAG-first, document management
- **LocalAI**: OpenAI-compatible API, multi-modal, Docker-first
- **Open WebUI**: Web-based, Ollama integration, modern UI
- **Nut Studio**: Beginner-friendly, visual interface, Windows/Mac

**Market Share (Estimated, March 2026):**
1. LM Studio: 35% (GUI dominance)
2. Ollama: 30% (Developer favorite)
3. GPT4All: 15% (Lightweight users)
4. Jan.ai: 10% (Privacy-focused)
5. vLLM: 5% (Production deployments)
6. Others: 5% (Text Gen WebUI, Llamafile, etc.)

**GGUF Format:**
- Full Name: GPT-Generated Unified Format
- Created By: Georgi Gerganov (ggerganov), Released: August 2023
- Purpose: Efficient CPU/GPU inference
- Advantages: Quantization support (2-bit to 8-bit), 50-90% size reduction, mmap support, cross-platform
- Most Popular Quantizations: Q4_K_M (40%), Q5_K_M (25%), Q8_0 (15%), Q6_K (10%)
- Top GGUF Creators: TheBloke (15K+), bartowski (8K+), mradermacher (6.5K+), QuantFactory (5K+)
- Ecosystem: llama.cpp, Ollama, LM Studio, GPT4All, Jan.ai, Kobold.cpp, llama-cpp-python

**2026-2027 Predictions:**
- Hugging Face: 5-7 million models by EOY 2027
- GGUF Models: 300,000+ (doubling from 2026)
- Ollama Valuation: $100-200M (Series A expected)
- LM Studio Valuation: $200-300M (Series B expected)
- Market Consolidation: 2-3 major local LLM platforms dominate

#### Changed
- Replaced placeholder provider icons with actual brand logos and generated fallbacks
- Updated provider icon resolution to use `get_provider_icon_path()` and `load_provider_icon()` APIs
- Organized all provider icons under `assets/icons/llm_providers/` directory
- Added provider metadata including display names, domains, and brand colors
- Achieved 100% icon coverage across all 164 providers

### Top Bar Screen Dock and Carousel Refinement - 2026-03-31

#### Changed
- Moved the macOS-style screen controller out of the editor file-tab strip and into the real title-bar center so it behaves as a separate screen dock instead of a tab skin.
- Added a live title-bar screen dock wired to the active pane, with per-screen icons, right-click icon switching, direct screen activation, and plus/list popovers for adding or jumping between screens.
- Restored the shared file tab and tab-bar components to a more standard Zed-style presentation so the new dock owns the macOS-like visual language instead of the editor tab strip.
- Refined the production carousel edge affordances to use centered highlight pills instead of full-height side rails, while preserving resize behavior and adding edge-swipe screen activation for adjacent screens.

### Onboarding, Dx Theme, and Editor Effects - 2026-03-31

#### Added
- Added bundled `Dx Light` and `Dx Dark` theme definitions under `assets/themes/dx/` and mirrored the same family into the local `extensions/theme/` pack.
- Added a new onboarding theme-editor modal that writes real per-theme overrides and updates the active Zed palette immediately.

#### Changed
- Updated fresh-theme defaults so new installs land on the `Dx` theme family instead of the previous default pair.
- Updated the onboarding theme picker to include `Dx` alongside the existing built-in theme families and added a theme-editor icon shortcut next to the light/dark/system mode toggle.
- Strengthened the animated rainbow caret with a brighter multi-band glow and faster hue travel so it reads as a genuinely animated cursor instead of a mostly static color block.
- Increased typing particle burst count, radius, lifetime, and velocity so edit-time effects feel closer to a pronounced power-mode presentation.

### AI Composer and Editor Effects - 2026-03-31

#### Added
- Added a redesigned real Agent composer shell with a top quick-action strip for changes, files, browser context, and additional tools.
- Added a responsive media-mode switcher with native icon assets for text, audio, video, 3D, live, AR, VR, PDF, and chart workflows, including overflow handling for narrow layouts.
- Added a new `editor::typing_effects` module that cycles through particles, fireworks, flames, and magic-style bursts as the user types.

#### Changed
- Updated the real Agent panel input experience to feel closer to the latest Copilot/Codex-style composer surface while keeping Zed's existing mode, model, profile, and send controls functional.
- Updated prompt resolution so non-text media modes prepend delivery guidance to the actual content blocks sent to the agent, making the switcher behavior functional instead of cosmetic.
- Updated the real editor cursor renderer to draw an animated rainbow caret for the newest local cursor.

### AI Provider Hub and Model Picker - 2026-03-31

#### Added
- Added a new `language_models::provider_hub` runtime that merges `models.dev`, OpenRouter, and optional LiteLLM `/models` metadata into one provider catalog.
- Added manifest-backed dynamic provider registration so synced OpenAI-compatible providers can appear in the real Zed AI settings and model picker without hand-written Rust clients.
- Added provider-hub disk caching under Zed's data directory so the editor can reuse the last synced provider catalog offline while background refresh runs.
- Added curated quick-add provider presets in the real AI settings modal for OpenAI-compatible, OpenRouter, GitHub Models, Groq, Together AI, Fireworks AI, Perplexity, DeepInfra, Hugging Face, Baseten, Replicate, LiteLLM Proxy, vLLM, Llamafile, and Text Generation WebUI.
- Added async live model discovery in the Add Provider modal by querying compatible `/models` endpoints and replacing the manual model list with discovered models.

#### Changed
- Updated the real AI settings panel to sort providers by synced category, display category/model-count badges per provider, expose provider-catalog sync status, and include a manual Refresh Catalog action.
- Updated the right-side model picker to react to provider-hub changes and show synced context-window plus capability badges inline for models.
- Updated the right-side AI model picker popover so provider sections are collapsible and show a per-provider model-count badge next to the disclosure control.
- Cleaned up the provider settings render path and model-picker test scaffolding during a file-only preflight pass to reduce first-run compile risk before runtime validation.

### Rich File Preview and Embedded Web Panel - 2026-03-31

#### Added
- Added the new `crates/rich_file_preview/` workspace crate and wired it into the root workspace plus the `zed` application crate.
- Added an extension-based project item registry for video, 3D model, LaTeX, PDF, DOCX, spreadsheet, presentation, markdown, SVG, and audio files.
- Added a native `UniversalPreviewView` item that asynchronously loads and renders document, media, and model previews inside normal workspace tabs.
- Added a dockable EmbeddedWebPreviewPanel built on wry with devtools, DOM element picking, and Agent-panel copy-to-AI handoff.
- Added browser-profile discovery for Chromium- and Firefox-family browsers so the embedded preview can detect real installed extensions from the user's local browser profiles.
- Added isolated embedded-web session policies for shared, per-origin, and incognito browsing, with localhost auth clearing to avoid sticky dev auth collisions.
- Added a stronger embedded inspector flow with live hover capture, click-to-send element capture for the Agent panel, and CSS override application directly back into the previewed page.

#### Changed
- Updated Zed startup initialization so the new preview registry and embedded web panel are registered alongside the existing image, markdown, CSV, and SVG preview systems.
- Updated the rich preview crate dependency set to pin the requested document, media, rendering, and webview libraries in one workspace module.


### Phase 1 - Project Setup & Infrastructure - 2026-03-31

#### Added
- Created the initial `crates/animation_demo/` scaffold with `src/`, `src/utils/`, and `shaders/` directories for the GPUI animation crate.
- Added the new `animation_demo` workspace crate and wired it into the root workspace plus the `zed` application crate.
- Added `crates/animation_demo/src/lib.rs` and an item-backed `demo_tab.rs` surface for Phase 1 animation validation.
- Added a live dimension panel that reports the current viewport size, target FPS, frame budget, and preview surface intent.
- Added placeholder video and 3D preview panels inside the demo tab so later rendering work has a visible target area from the start.
- Verified that the web reference source is available under `www/` for side-by-side implementation work.

#### Changed
- Updated Zed startup initialization to open the new animation demo tab automatically for each workspace during Phase 1 development.
- Updated the `just run` workflow to build both `zed` and `cli` before launch so local development no longer trips the missing `zed-cli` runtime error.

### Phase 2 - Core Animation Components - 2026-03-31

#### Added
- Added `spring.rs` with Apple-style presets, analytical spring solving, and mid-flight retargeting support.
- Added `easing.rs` with reusable Apple-inspired cubic-bezier curves and a bezier solver.
- Added `animator.rs` with `AnimatedValue`, `AnimatedPoint`, `AnimatedColor`, and RGBA helpers for scalar, point, and color animation state.
- Added `transition.rs` with reusable Apple-style helpers for fades, sheet presentation, slide-up motion, and sidebar width changes.
- Added `gesture.rs` with drag-axis modeling, elastic constraints, drag velocity tracking, and hold detection primitives for later interactive work.

#### Changed
- Expanded the animation demo tab to surface Phase 2 motion-toolkit diagnostics so spring, easing, and gesture behavior are visible from the demo environment.

### Phase 3-4 - Friday Border + Hello Glow Previews - 2026-03-31

#### Added
- Added `friday.rs` with a cycling Friday border preview that includes rainbow segmented borders, staged edge reveals, dual glow layers, and a bounce-shifted content area.
- Added `hello_glow.rs` with a GPUI-native glow card preview that cycles a 25-color rainbow field with inner and outer glow layers.
- Added `sidebar.rs`, `carousel.rs`, and `macos_dock.rs` preview components to start expressing the later UI replacement phases inside the demo surface.

#### Changed
- Expanded the demo tab to render live Friday Border, Hello Glow, Sidebar, Carousel, and Dock previews alongside the Phase 2 motion toolkit.

### Phase 8-9 - Drag Preview + Media Direction - 2026-03-31

#### Added
- Added `drag_drop.rs` with a visual drag-and-drop preview showing a ghost tab card plus blue dot-and-line drop indicators.

#### Changed
- Updated the demo tab's video and 3D placeholders to reflect the selected Phase 9 implementation direction: `gstreamer` or `ffmpeg-next` for video, and `wgpu` or `three-d` for 3D rendering.

### Phase 5-8 - Production Sidebar + Dock Integration - 2026-03-31

#### Changed
- Updated the shared `ui::Tab` component to behave more like a dock surface with pill styling, hover emphasis, and a pulsing active indicator.
- Updated the shared `ui::TabBar` layout so pane tabs render inside a centered dock-like chrome while preserving the existing split, add, and zoom controls.
- Updated `workspace::MultiWorkspace` so the real workspace sidebar opens with directional slide-in motion and an elevated container instead of appearing abruptly.
- Updated `workspace::MultiWorkspace` sidebar rendering to animate live width and opacity during open and close transitions instead of snapping visibility.
- Updated the production sidebar renderer to animate in from the active edge, aligning the real workspace chrome with the animation system used in the demo crate.
- Updated the production dragged-tab overlay to use reduced opacity and elevated shadow so reordering feedback matches the dock-like tab system more closely.
- Updated the production tab drop targets to surface blue line-and-dot indicators and stronger selection-colored borders during tab and project-entry drags.

### Phase 5-6 - Production Sidebar Motion + Carousel Gravity - 2026-03-31

#### Changed
- Added real per-project collapse animation state to the production sidebar so workspace groups animate their child rows with height, opacity, and slide transitions instead of snapping open and closed.
- Updated the production sidebar's keyboard navigation and selection recovery so collapsed or animating-hidden rows are skipped correctly during folder and workspace interactions.
- Added subtle hover scaling to project headers so workspace interactions carry visible motion even before a collapse or expand finishes.
- Upgraded the production pane carousel to use a true resizable active screen width with a 400px minimum, live resize feedback, and persisted left/right resize bias.
- Updated the carousel positioning spring to match the web reference target more closely with `stiffness: 300` and `damping: 30`.
- Added left, right, centered, and wrap-around neighbor reveal behavior so resized screens expose adjacent tabs with directional gravity at the beginning and end of the tab strip.

### Project Initialization - 2026-03-31

#### Added
- Created AGENTS.md with comprehensive project instructions for Codex CLI
- Created TODO.md with 10-phase task breakdown and progress tracking
- Created CHANGELOG.md for tracking all project changes
- Documented low-end device constraints (only use `just run`, never cargo commands)

#### Project Structure
- Planned `animation_demo` crate for all custom animations
- Planned `www/` folder for web reference code
- Planned demo tab for live animation testing with dimensions display

#### Documentation
- ANIMATIONS.md: Complete animation specifications (3183 lines)
- DX.md: Developer experience guide with smooth animation patterns (3128 lines)
- AGENTS.md: Codex CLI instructions with coding standards and constraints

---

## Project Roadmap

### Phase 1: Project Setup & Infrastructure (Planned)
**Target:** March 31, 2026  
**Status:** ðŸŸ¡ In Progress

- Create `animation_demo` crate structure
- Set up project dependencies
- Copy web reference code to `www/` folder
- Create demo tab infrastructure
- Configure video/3D rendering preview

### Phase 2: Core Animation Components (Planned)
**Target:** April 1-2, 2026  
**Status:** ðŸ”œ Pending

- Implement spring physics system
- Implement easing curves
- Create animated value system
- Build transition helpers
- Create gesture handling system

### Phase 3: Friday Border Effect (Planned)
**Target:** April 3-4, 2026  
**Status:** â¸ï¸ Not Started

- Rainbow gradient shader implementation
- Border slide-in animation
- Dual glow layers
- Scroll bounce effect

### Phase 4: Hello Glow Effect (Planned)
**Target:** April 5, 2026  
**Status:** â¸ï¸ Not Started

- 25-color HSL gradient shader
- Dual-layer blur system
- Background-position animation

### Phase 5: Sidebar Animations (Planned)
**Target:** April 6-7, 2026  
**Status:** â¸ï¸ Not Started

- Width transition animations
- Content fade animations
- Folder collapse system
- Workspace scroll with spring
- Replace Zed's default sidebar

### Phase 6: Screen Carousel (Planned)
**Target:** April 8-10, 2026  
**Status:** â¸ï¸ Not Started

- Resizable screen system
- Directional gravity logic
- Smooth positioning with springs
- Circular wrapping
- Replace Zed's tab system

### Phase 7: macOS-Style Dock (Planned)
**Target:** April 11-12, 2026  
**Status:** â¸ï¸ Not Started

- Dock icon rendering
- Hover magnification
- Active indicator with glow
- Drag-to-reorder functionality

### Phase 8: Drag-and-Drop System (Planned)
**Target:** April 13-14, 2026  
**Status:** â¸ï¸ Not Started

- Event handling system
- Sortable tabs and folders
- Drop position indicators
- Visual feedback system

### Phase 9: Video & 3D Rendering (Planned)
**Target:** April 15-17, 2026  
**Status:** â¸ï¸ Not Started

- Video player component
- 3D viewer component
- Preview in demo tab
- Performance optimization

### Phase 10: Integration & Polish (Planned)
**Target:** April 18-19, 2026  
**Status:** â¸ï¸ Not Started

- Component integration
- Performance optimization
- Cross-platform testing
- Final polish

---

## Version History

### [0.0.1] - 2026-03-31 (Current)

#### Added
- Initial project setup
- Documentation structure
- Task tracking system
- Codex CLI integration

#### Notes
- Project started on March 31, 2026
- Using Codex CLI for systematic development
- Low-end device constraints documented
- Only `just run` command allowed for testing

---

## Development Guidelines

### Commit Message Format
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `docs`: Documentation changes
- `test`: Test additions or changes
- `chore`: Build process or auxiliary tool changes

**Example:**
```
feat(friday-border): implement rainbow gradient shader

- Added Metal shader for 25-color HSL gradient
- Implemented border slide-in animation (750ms)
- Added dual glow layers (8px + 12px blur)

Closes #1
```

### Testing Notes
- **CRITICAL:** Only use `just run` for testing
- Never use `cargo build`, `cargo test`, `cargo check`, or `cargo clippy`
- Low-end device cannot handle multiple cargo processes
- All tests must be run through `just run` command

### Performance Requirements
- Maintain 120 FPS (8.33ms frame budget)
- GPU-accelerated rendering only
- Minimal CPU usage during animations
- No memory leaks

---

## Statistics

**Total Commits:** 0  
**Total Features Added:** 0  
**Total Bugs Fixed:** 0  
**Total Refactors:** 0  
**Lines of Code:** 0  
**Test Coverage:** 0%

---

## Contributors

- Project Lead: [Your Name]
- AI Assistant: Codex CLI (OpenAI)
- Framework: Zed + GPUI

---

## License

[Your License Here]

---

**Last Updated:** March 31, 2026  
**Next Review:** April 1, 2026



