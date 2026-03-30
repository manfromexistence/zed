# AI Coding Agents Guide - GPT-5.4 & Codex CLI

**Date:** March 30, 2026  
**Model:** GPT-5.4 via Codex CLI
**Focus:** Recreating Web Animations in Zed Code Editor using Rust + GPUI

---

## ⚠️ CRITICAL WARNINGS - READ FIRST ⚠️

### Low-End System Requirements

**THIS IS A LOW-END SYSTEM WITH LIMITED RESOURCES**

Zed is a massive project that functions like a low-level operating system. You MUST follow these rules:

#### ✅ ALLOWED COMMANDS:
```bash
just run          # ONLY command for testing Zed
just --list       # See available just commands
```

#### ❌ FORBIDDEN COMMANDS (WILL CRASH SYSTEM):
```bash
cargo build       # ❌ NEVER USE - Requires too much RAM
cargo build --release  # ❌ NEVER USE - Will crash
cargo test        # ❌ NEVER USE - Too resource intensive
cargo check       # ❌ NEVER USE - Will hang system
cargo clippy      # ❌ NEVER USE - Too much memory
cargo run         # ❌ NEVER USE - Use 'just run' instead
```

**Why:** These commands require significant RAM (8GB+) and disk space (10GB+) that this system doesn't have. Using them will cause:
- Out of memory errors
- System freezes
- Disk space exhaustion
- Build failures

**ONLY use `just run` to test your changes. The justfile is configured for low-end systems.**

---

### Web Code Location

**The website code is located in `zed/www/` folder**

This is where you'll find all the reference implementations:
```
zed/www/
├── components/
│   ├── friday.tsx              # Friday border animation
│   ├── hello-glow.tsx          # Hello glow effect
│   ├── browser/                # Sidebar components
│   │   ├── sidebar-expanded.tsx
│   │   ├── sidebar-collapsed.tsx
│   │   └── draggable-tab.tsx
│   └── screens/                # Carousel and dock
│       ├── screen-carousel.tsx
│       └── macos-dock.tsx
├── app/
├── package.json
└── README.md
```

**Purpose:** This allows Codex CLI (GPT-5.4) to easily study how animations work in the web version before implementing them in GPUI. Always reference these files when implementing animations.

---

## Executive Summary

This document provides comprehensive guidance for AI coding agents (GPT-5.4 via Codex CLI) working on the Zed animation recreation project. It covers model capabilities, project structure, coding conventions, and agent-specific workflows.

### Project Context

**Goal:** Recreate web animations from Friday application in Zed code editor using Rust + GPUI

**Key Technologies:**
- Rust (Edition 2024)
- GPUI (Zed's GPU-accelerated UI framework)
- Metal/Vulkan shaders
- Framer Motion patterns → GPUI animations

**Web Reference:** All website code is in `zed/www/` folder for easy reference

**Testing:** ONLY use `just run` command (never use cargo commands)

---

## ⚠️ MANDATORY READING - SYSTEM CONSTRAINTS ⚠️

### This is a Low-End System

You are working on a resource-constrained system. Zed is an enormous project (like a mini operating system) that requires careful handling.

#### Testing Commands

| Command | Status | Reason |
|---------|--------|--------|
| `just run` | ✅ REQUIRED | Optimized for low-end systems |
| `cargo build` | ❌ FORBIDDEN | Requires 8GB+ RAM, will crash |
| `cargo test` | ❌ FORBIDDEN | Too resource intensive |
| `cargo check` | ❌ FORBIDDEN | Will hang system |
| `cargo clippy` | ❌ FORBIDDEN | Too much memory usage |
| `cargo run` | ❌ FORBIDDEN | Use `just run` instead |

#### What Happens If You Ignore This Warning

```bash
# ❌ If you do this:
cargo build

# 💥 This will happen:
# - System runs out of memory
# - Build process crashes
# - Disk fills up
# - System becomes unresponsive
# - You lose all progress
```

#### The Correct Way

```bash
# ✅ Always do this:
just run

# This command:
# - Uses optimized build settings
# - Manages memory efficiently
# - Works on low-end systems
# - Tests your changes properly
```

### Web Code Reference Location

**IMPORTANT:** The website code is NOT in `reference/` folder. It's in `www/` folder.

```bash
# ✅ CORRECT - Web code is here
zed/www/components/friday.tsx
zed/www/components/hello-glow.tsx
zed/www/components/browser/sidebar-expanded.tsx

# ❌ WRONG - This doesn't exist
reference/friday-web/components/friday.tsx
```

**Why:** The `www/` folder contains the complete website codebase so Codex CLI (GPT-5.4) can easily study how animations work before implementing them in GPUI.

### Your Workflow with Codex CLI

1. **Study** - Read web code in `www/` folder using `/mention` or file reading
2. **Plan** - Design GPUI implementation
3. **Implement** - Write Rust code in `crates/animation_demo/`
4. **Test** - Run `just run` (ONLY this command)
5. **Verify** - Check Animation Demo tab in Zed

**Never skip step 4's warning: ONLY use `just run`**

---

## Part 1: GPT-5.4 Model Capabilities

### Overview

**Released:** March 5, 2026  
**Model ID:** `gpt-5.4`  
**Context Window:** Up to 1 million tokens  
**Specialty:** Frontier model for professional work, agentic workflows, and computer use
**Access:** Via Codex CLI

### Core Capabilities

1. **Native Computer Use**
   - First general-purpose model with native computer-use capabilities
   - Can write Playwright code, read screenshots, issue keyboard/mouse actions
   - Operates desktop and web apps like a human
   - 75.0% success rate on OSWorld-Verified benchmark

2. **Extended Context (1M tokens)**
   - Plan, execute, and verify tasks across long horizons
   - Maintain coherence across complex projects
   - Process vast amounts of information
   - Perfect for large codebases like Zed

3. **Agentic Workflows**
   - Multi-step reasoning and planning
   - Tool search and selection
   - Autonomous task execution
   - Verification and iteration

4. **Coding Excellence**
   - Combines GPT-5.3-Codex coding pipeline with general reasoning
   - 83% win rate against industry professionals
   - Significantly fewer tokens than predecessors
   - Optimized for long-horizon coding tasks

5. **Professional Work**
   - Document analysis
   - Report preparation
   - Spreadsheet work
   - Presentation creation

### Model Comparison

| Model | Context | Specialty | Best For |
|-------|---------|-----------|----------|
| **gpt-5.4** | 1M tokens | Unified frontier model | Complex professional work, agentic tasks |
| **gpt-5.3-codex** | 128K tokens | Agentic coding | Interactive coding with steering |
| **gpt-5.2-codex** | 128K tokens | Coding | Long-horizon coding tasks |
| **gpt-5.1-codex-max** | 128K tokens | Previous gen | Legacy projects |

### When to Use GPT-5.4

✅ **Use GPT-5.4 for:**
- Complex multi-file refactoring
- Architecture design and planning
- Long-horizon agentic tasks
- Computer use and automation
- Large codebase analysis (Zed fork)
- Multi-step implementation plans

❌ **Don't use GPT-5.4 for:**
- Simple one-off code snippets (use gpt-5.3-codex-spark)
- Quick questions (use standard models)
- Cost-sensitive tasks (use smaller models)

---

## Part 2: Codex CLI Overview

### What is Codex CLI?

**Codex CLI** is OpenAI's terminal-based coding agent built in Rust. It's a powerful tool for autonomous code editing that runs locally, reads your repo, makes code changes, and runs shell commands.

**Key Features:**
- Built in Rust (fast performance)
- Runs locally with cloud sandbox option
- Reads entire codebase
- Makes file changes with approval system
- Executes shell commands
- MCP (Model Context Protocol) integration
- AGENTS.md configuration system

### Installation

```bash
# Install Codex CLI
npm install -g @openai/codex-cli

# Or via Homebrew (macOS)
brew install openai/tap/codex

# Authenticate
codex
# Opens browser for ChatGPT OAuth

# Or use API key
export OPENAI_API_KEY="your-api-key"
codex
```

### Basic Usage

**Interactive TUI:**
```bash
# Launch interactive mode with GPT-5.4
codex -m gpt-5.4

# With initial prompt
codex -m gpt-5.4 "Study www/components/friday.tsx and implement in GPUI"

# With specific model
codex -m gpt-5.4 "implement screen carousel"
```

**Non-Interactive (Scripting):**
```bash
# Execute and exit
codex -m gpt-5.4 exec "analyze the Friday border animation"

# Short form
codex -m gpt-5.4 e "implement sidebar animation"

# ⚠️ CRITICAL: Never use cargo commands in exec
# ❌ WRONG: codex exec "cargo build && cargo test"
# ✅ CORRECT: codex exec "implement feature, then I'll run 'just run'"
```

**Resume Sessions:**
```bash
# Show session picker
codex resume

# Resume most recent
codex resume --last
```

### Configuration

**Main config:** `~/.codex/config.toml`

```toml
model = "gpt-5.4"
model_reasoning_effort = "high"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
web_search = "cached"

[features]
multi_agent = true
shell_tool = true

[profiles.zed-dev]
approval_policy = "on-request"
sandbox_mode = "workspace-write"
# Never allow cargo commands
forbidden_commands = ["cargo build", "cargo test", "cargo check", "cargo clippy", "cargo run"]

[profiles.paranoid]
approval_policy = "untrusted"
sandbox_mode = "read-only"
```

**Load profile:**
```bash
codex --profile zed-dev
```

### Approval Modes

| Mode | Behavior |
|------|----------|
| `untrusted` | Prompt for every action |
| `on-request` | Prompt for risky operations only (RECOMMENDED) |
| `never` | Fully autonomous (use with caution) |
| `reject` | Auto-reject specific categories |

### Sandbox Modes

| Mode | Access |
|------|--------|
| `read-only` | Can read files; no writes or shell exec |
| `workspace-write` | Read/write within working directory (RECOMMENDED) |
| `danger-full-access` | Unrestricted (use in isolated envs only) |

### Slash Commands (Interactive Mode)

**Session Management:**
- `/new` - Start fresh conversation
- `/resume` - Reload previous conversation
- `/fork` - Clone current thread
- `/compact` - Summarize to free tokens
- `/status` - Show model, policy, token usage
- `/quit` - Exit

**Configuration:**
- `/model` - Switch model
- `/permissions` - Adjust approval mode
- `/personality` - Change communication style
- `/plan` - Enter plan mode (review before execution)
- `/experimental` - Toggle features

**Dev Tools:**
- `/diff` - Show Git changes
- `/review` - Request working-tree analysis
- `/mention` - Attach specific files (e.g., `/mention zed/www/components/friday.tsx`)
- `/init` - Generate AGENTS.md scaffold
- `/mcp` - List configured MCP tools

### ⚠️ CRITICAL: Codex CLI and Low-End Systems

**Important:** Codex CLI can execute shell commands. You MUST configure it to prevent cargo commands:

```toml
# Add to ~/.codex/config.toml
[profiles.zed-dev]
forbidden_commands = [
    "cargo build",
    "cargo test", 
    "cargo check",
    "cargo clippy",
    "cargo run"
]
```

**Always remind Codex:**
```bash
codex -m gpt-5.4 "Implement sidebar animation. CRITICAL: Only use 'just run' to test, never cargo commands."
```

---

## Part 3: AGENTS.md System

### What is AGENTS.md?

AGENTS.md is Codex's project instruction file - a set of guidelines loaded before any task. It provides context, conventions, and rules for AI agents.

### File Priority

Files are loaded in this order (later overrides earlier):

1. `~/.codex/AGENTS.override.md` (global override)
2. `~/.codex/AGENTS.md` (global)
3. Git root → current directory:
   - `AGENTS.override.md`
   - `AGENTS.md`
   - Fallback filenames (e.g., `TEAM_GUIDE.md`)

**Size Limit:** 32 KiB (configurable via `project_doc_max_bytes`)

### Generate Scaffold

```bash
codex /init
```

### Example Global AGENTS.md

**Location:** `~/.codex/AGENTS.md`

```markdown
## Working Agreements

- Always run tests after modifying source files
- Follow language-specific conventions (snake_case for Rust, camelCase for TypeScript)
- Use descriptive commit messages
- Request code review for significant changes

## Code Quality

- Write clear, self-documenting code
- Add comments for complex logic
- Follow DRY (Don't Repeat Yourself)
- Prefer composition over inheritance

## Testing

- Write unit tests for new functions
- Update tests when modifying existing code
- Aim for >80% code coverage
- Run full test suite before committing
```

### Example Project AGENTS.md

**Location:** `<project-root>/AGENTS.md`

This file (the one you're reading) serves as the project AGENTS.md for the Zed animation project.

### Subsystem Override

**Location:** `crates/animation_demo/AGENTS.override.md`

```markdown
## Animation Demo Crate - Strict Rules

### Performance Requirements

- MUST maintain 120 FPS
- MUST profile every animation
- MUST test on all platforms
- MUST check memory leaks

### Code Review

- All animations require visual review
- Performance metrics must be documented
- Shader code requires extra scrutiny

### Testing

- NEVER use cargo test on low-end system
- Only write test code
- Tests will run in CI/CD
```

---

## Part 4: Project-Specific Agent Instructions

### For This Project (Zed Animation Recreation)

**Primary Goal:** Recreate Friday web animations in Zed using Rust + GPUI

**Key Files to Study:**
- `animations.md` - Complete animation guide
- `zed/www/components/` - Web reference implementations
- `crates/workspace/src/workspace.rs` - Workspace structure
- `crates/workspace/src/sidebar.rs` - Current sidebar
- `crates/workspace/src/pane.rs` - Tab system
- `crates/gpui/src/` - GPUI framework

**Development Workflow:**

1. **Study Phase**
   - Read `animations.md` thoroughly
   - Study web code in `www/` folder
   - Understand GPUI patterns from existing Zed code
   - Review Framer Motion → GPUI translation patterns

2. **Planning Phase**
   - Break down animation into steps
   - Identify GPUI primitives needed
   - Plan shader requirements (if any)
   - Estimate performance impact

3. **Implementation Phase**
   - Create/modify files in `animation_demo` crate
   - Implement animation module
   - Build components (sidebar, carousel, dock)
   - Write shaders (gradient, blur) if needed

4. **Testing Phase**
   - Run `just run` (ONLY this command)
   - Test in Animation Demo tab
   - Verify 120 FPS in FPS counter
   - Check memory usage doesn't grow
   - Compare with web version in www/

5. **Integration Phase**
   - Replace Zed components
   - Test with real editors
   - Verify all features work
   - Performance optimization if needed

### Animation Implementation Checklist

For each animation:

- [ ] Study web implementation
- [ ] Identify key properties (position, size, opacity, etc.)
- [ ] Determine easing/spring physics
- [ ] Create `AnimatedValue` for each property
- [ ] Implement `Render` trait
- [ ] Add to demo tab
- [ ] Profile performance
- [ ] Optimize if needed
- [ ] Document in code
- [ ] Add tests

### Code Quality Standards

**Rust Code:**
```rust
// ✅ GOOD: Clear, documented, idiomatic
/// Animated sidebar that expands/collapses with spring physics.
pub struct AnimatedSidebar {
    /// Current width (animated)
    width: AnimatedValue<f32>,
    /// Whether sidebar is expanded
    expanded: bool,
}

impl AnimatedSidebar {
    /// Creates a new sidebar in collapsed state (56px)
    pub fn new() -> Self {
        Self {
            width: AnimatedValue::new(56.0, 56.0, SpringConfig {
                stiffness: 400.0,
                damping: 30.0,
                mass: 1.0,
            }),
            expanded: false,
        }
    }
    
    /// Toggles sidebar between expanded (360px) and collapsed (56px)
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
        self.width.target = if self.expanded { 360.0 } else { 56.0 };
    }
}
```

```rust
// ❌ BAD: No docs, unclear naming, magic numbers
pub struct Sidebar {
    w: AnimatedValue<f32>,
    e: bool,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            w: AnimatedValue::new(56.0, 56.0, SpringConfig {
                stiffness: 400.0,
                damping: 30.0,
                mass: 1.0,
            }),
            e: false,
        }
    }
}
```

### Performance Guidelines

**Frame Budget (120 FPS = 8.33ms):**
- Layout: 2ms
- Animation updates: 1ms
- Painting: 2ms
- GPU rendering: 3ms
- Buffer: 0.33ms

**Optimization Checklist:**
- [ ] Pre-allocate buffers
- [ ] Batch GPU operations
- [ ] Cull off-screen elements
- [ ] Cache expensive calculations
- [ ] Use GPU for heavy work (blur, gradients)
- [ ] Only request frame when animating
- [ ] Profile with `cargo flamegraph`

### Git Commit Messages

```bash
# ✅ GOOD: Clear, descriptive
git commit -m "feat(sidebar): Add animated expand/collapse with spring physics

- Implement AnimatedValue for width property
- Add toggle() method with 56px ↔ 360px transition
- Use spring config: stiffness=400, damping=30
- Maintain 120 FPS performance
- Add unit tests for animation settling"

# ❌ BAD: Vague, no context
git commit -m "update sidebar"
```

---

## Part 5: Agent Workflows

### Workflow 1: Implementing New Animation

**Agent:** GPT-5.4 (via Codex CLI)

```bash
# Step 1: Study web implementation in www/ folder
codex -m gpt-5.4

# In session:
> /mention zed/www/components/friday.tsx
> Analyze this Friday border animation and explain how it works in detail

# Step 2: Plan GPUI implementation
> Create an implementation plan for Friday border in GPUI, including shader requirements and performance considerations

# Step 3: Implement
> Implement Friday border animation in crates/animation_demo/src/friday.rs following the plan

# Step 4: Test (CRITICAL - User will run this manually)
> I've implemented the code. The user should now run 'just run' to test it.
> NEVER suggest cargo build, cargo test, or cargo check.

# Step 5: Verify
> Once tested, integrate Friday border into demo tab with dimensions display
```

**CRITICAL:** Never execute or suggest cargo commands. Always remind the user to run `just run`.

### Workflow 2: Debugging Performance Issue

**Agent:** GPT-5.4 (via Codex CLI)

```bash
# Launch Codex with GPT-5.4
codex -m gpt-5.4

# In session:
> /mention crates/animation_demo/src/sidebar.rs
> The sidebar animation is dropping to 60 FPS. Analyze the code and identify performance bottlenecks.

# Agent will:
# 1. Analyze code
# 2. Identify bottlenecks (allocations, expensive calculations, etc.)
# 3. Propose fixes
# 4. Implement optimizations
# 5. Remind user to test with 'just run'

# NEVER suggest profiling commands like 'cargo flamegraph' on this system
```

**CRITICAL:** Use the built-in FPS counter in the demo tab for performance monitoring, not external profiling tools.

### Workflow 3: Refactoring for Code Quality

**Agent:** GPT-5.4 (via Codex CLI)

```bash
codex -m gpt-5.4 "Refactor animation module to extract common patterns into reusable utilities. Maintain all existing functionality. Remind me to test with 'just run' when done."
```

### Workflow 4: Adding Tests

**Agent:** GPT-5.4 (via Codex CLI)

**IMPORTANT:** Do NOT run tests on this low-end system. Only write test code.

```bash
codex -m gpt-5.4 "Add unit tests for AnimatedValue spring physics calculations. Write the test code but DO NOT run cargo test - this system can't handle it."
```

Example test code:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spring_animation() {
        let mut value = AnimatedValue::new(0.0, 100.0, SpringConfig::default());
        
        // Simulate 1 second
        for _ in 0..60 {
            value.update(1.0 / 60.0);
        }
        
        assert!((value.current - 100.0).abs() < 1.0);
    }
}
```

**DO NOT RUN:** `cargo test` - This will crash the system.

**Instead:** Tests will be run in CI/CD or on a more powerful machine.

### Workflow 5: Documentation

**Agent:** GPT-5.4 (via Codex CLI)

```bash
codex -m gpt-5.4 "Generate comprehensive API documentation for the animation module with examples. Add doc comments to all public APIs."
```

---

## Part 6: Performance Monitoring

### Using the Demo Tab

The Animation Demo tab provides real-time performance metrics:

**FPS Counter:**
- Shows current frames per second
- Target: 120 FPS
- Warning if drops below 100 FPS

**Frame Time:**
- Shows milliseconds per frame
- Target: <8.33ms (for 120 FPS)
- Helps identify performance bottlenecks

**Animation Progress:**
- Shows completion percentage (0-100%)
- Useful for debugging animation timing
- Helps verify animations complete properly

**Dimensions:**
- Live width/height tracking
- Useful for responsive animations
- Helps match web implementation

### Performance Optimization

If FPS drops below 120:

1. **Check Render Loop**
   - Remove unnecessary calculations
   - Cache expensive operations
   - Use dirty flags

2. **Optimize Allocations**
   - Pre-allocate buffers
   - Reuse data structures
   - Avoid allocations in hot paths

3. **GPU Optimization**
   - Batch similar operations
   - Minimize draw calls
   - Use efficient shaders

4. **Profile Code**
   - Add timing logs
   - Identify slow sections
   - Focus optimization efforts

---

## Part 7: Git Workflow

### Commit Messages

```bash
# ✅ GOOD: Clear, descriptive
git commit -m "feat(sidebar): Add animated expand/collapse

- Implement AnimatedValue for width property
- Add toggle() method with 56px ↔ 360px transition
- Use spring config: stiffness=400, damping=30
- Maintain 120 FPS performance
- Add unit tests for animation settling"

# ❌ BAD: Vague, no context
git commit -m "update sidebar"
```

### Branch Strategy

```bash
# Create feature branch
git checkout -b feature/sidebar-animation

# Make changes and commit
git add crates/animation_demo/src/sidebar.rs
git commit -m "feat(sidebar): Implement animated sidebar"

# Test before pushing
just run

# Push to remote
git push origin feature/sidebar-animation
```

---

## Part 8: Best Practices

### DO ✅

1. **Read animations.md and study www/ folder first**
2. **Study existing Zed code before modifying**
3. **Test animations in demo tab before integration**
4. **ONLY use `just run` to test - NEVER use cargo commands**
5. **Write clear commit messages**
6. **Add documentation for public APIs**
7. **Reference www/ folder for web implementation details**
8. **Keep animations at 120 FPS**
9. **Use the demo tab's FPS counter for performance monitoring**
10. **Study web code in www/ before implementing in GPUI**

### DON'T ❌

1. **Don't use cargo build, cargo test, cargo check, or cargo clippy**
2. **Don't run any cargo commands directly - system will crash**
3. **Don't skip performance testing in demo tab**
4. **Don't use magic numbers (document constants)**
5. **Don't modify Zed core without understanding it**
6. **Don't ignore the FPS counter in demo tab**
7. **Don't write code without documentation**
8. **Don't optimize prematurely**
9. **Don't break existing Zed functionality**
10. **Don't exceed frame budget (8.33ms)**

### CRITICAL REMINDERS ⚠️

**Testing:**
```bash
# ✅ CORRECT
just run

# ❌ WRONG - WILL CRASH SYSTEM
cargo build
cargo test
cargo check
cargo run
```

**Web Reference:**
```bash
# ✅ CORRECT - Study web code here
zed/www/components/friday.tsx
zed/www/components/hello-glow.tsx
zed/www/components/browser/sidebar-expanded.tsx

# ❌ WRONG - Old location (doesn't exist)
reference/friday-web/components/friday.tsx
```

---

## Part 9: Troubleshooting

### Common Issues

**Issue:** Animation drops below 120 FPS

**Solution:**
1. Check the FPS counter in demo tab
2. Review code for expensive operations in render loop
3. Look for unnecessary allocations
4. Optimize shader code if needed
5. Test with `just run`

---

**Issue:** Can't find web reference code

**Solution:**
```bash
# Web code is in www/ folder, not reference/
cd zed/www/
ls components/
```

---

**Issue:** System crashes when building

**Solution:**
```bash
# STOP using cargo commands!
# Only use:
just run
```

---

**Issue:** Changes not showing up

**Solution:**
```bash
# Make sure you're testing correctly
just run

# Then open Zed and check the Animation Demo tab
```

---

**Issue:** Out of memory error

**Solution:**
- You probably used a cargo command
- Restart system
- Only use `just run` from now on
- Close other applications to free RAM

---

**Issue:** Shader compilation errors

**Solution:**
1. Check Metal shader syntax
2. Review www/ folder for web implementation
3. Test with `just run`
4. Check error messages in demo tab

---

## Part 10: Quick Reference

### Essential Commands

```bash
# ✅ ONLY ALLOWED COMMAND
just run          # Test Zed with your changes

# ❌ FORBIDDEN COMMANDS (WILL CRASH)
cargo build       # DON'T USE
cargo test        # DON'T USE
cargo check       # DON'T USE
cargo clippy      # DON'T USE
cargo run         # DON'T USE
```

### Web Reference Locations

```bash
# Animation implementations (study these first)
zed/www/components/friday.tsx
zed/www/components/hello-glow.tsx
zed/www/components/browser/sidebar-expanded.tsx
zed/www/components/browser/sidebar-collapsed.tsx
zed/www/components/screens/screen-carousel.tsx
zed/www/components/screens/macos-dock.tsx

# GPUI implementations (write these)
zed/crates/animation_demo/src/friday.rs
zed/crates/animation_demo/src/hello_glow.rs
zed/crates/animation_demo/src/sidebar.rs
zed/crates/animation_demo/src/carousel.rs
zed/crates/animation_demo/src/macos_dock.rs
```

### Model Information

| Aspect | Details |
|--------|---------|
| Model | GPT-5.4 via Codex CLI |
| Context | 1M tokens |
| Best for | Complex multi-file work, architecture design, agentic workflows |
| Access | Terminal: `codex -m gpt-5.4` |

### Performance Targets

- **Frame Rate:** 120 FPS (8.33ms per frame)
- **Memory:** No leaks, stable usage
- **CPU:** <50% on modern hardware
- **GPU:** Efficient shader usage

### File Locations

```
zed/
├── www/                      # ⭐ Web reference code (study this)
│   ├── components/
│   ├── app/
│   └── package.json
├── crates/
│   ├── animation_demo/       # Your implementations
│   │   └── src/
│   ├── workspace/            # Zed's workspace (modify)
│   └── gpui/                 # GPUI framework (study)
└── justfile                  # Build configuration (use 'just run')
```

### Critical Reminders

1. ⚠️ **ONLY use `just run` to test**
2. ⚠️ **Web code is in `www/` folder**
3. ⚠️ **Never use cargo commands directly**
4. ⚠️ **This is a low-end system - be careful**
5. ⚠️ **Study www/ before implementing**

---

**Document Version:** 2.0  
**Last Updated:** March 30, 2026  
**For:** GPT-5.4 (via Kiro IDE) working on Zed Animation Project
