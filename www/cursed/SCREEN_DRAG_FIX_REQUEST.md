## Goal

I need **perfect, intuitive drag behavior** for a horizontal screen carousel in a Next.js/React app. The current implementation mostly works but has edge‑case bugs and feels inconsistent, especially when:

- The **left sidebar expands/collapses**, changing the available width.
- The user **drags quickly vs. slowly** (velocity vs. distance thresholds).
- The active screen has been **resized** and no longer matches the container width.

Please **redesign and refactor the drag + layout logic** so that:

- The active screen is always laid out and animated in a **visually clean, predictable way**.
- Dragging is **smooth, weighted, and forgiving** (no “sausage” artefacts, no jitter).
- It behaves correctly when the **sidebar is toggled** or the window is resized.
- The code remains **strongly typed, easy to reason about, and well‑structured**.

You can assume a modern browser environment and that `ResizeObserver`, `motion/react`, and `re-resizable` are available.

---

## High‑level requirements

- **1. Layout & sizing**
  - The carousel lives in a container that fills the main content area of the browser (to the right of the sidebar).
  - By default, **each screen should occupy the full logical width/height of the available content area**, with a small, configurable gap between screens.
  - When the sidebar is **collapsed or expanded**, the carousel should:
    - Recompute the container width/height.
    - Update each screen’s width/height accordingly.
    - Keep the **currently active screen aligned correctly** without visual jumps.
  - When the window is resized, behavior should be the same as for sidebar toggling: no stale widths and no partial off‑screen artifacts.

- **2. Drag behavior**
  - Dragging should feel **weighted**:
    - Use both **drag distance** and **velocity** to decide whether to snap to the next/previous screen or back to the current one.
    - Avoid accidental screen changes on tiny/slow drags.
  - It should be possible to **drag slightly past the edges** for a rubber‑band feel, but:
    - It should **never drift permanently off‑screen**.
    - The active screen should always settle into a clean resting position.
  - When screens have varying widths (due to resizing), the centering logic should remain correct and predictable.

- **3. Resize behavior**
  - The active screen is resizable (via `re-resizable`), but:
    - Resizing should **not break** the drag or centering behavior.
    - After resize, the carousel should recompute the correct `x` offset for the active index.
  - The **non‑active screens** can use default full‑width values; they don’t have to retain individual resized widths unless you think it significantly improves UX.

- **4. Implementation quality**
  - Keep the logic **modular**:
    - Consider extracting reusable utility functions or hooks for:
      - Computing target `x` offsets for a given active index and screen sizes.
      - Deciding the next index based on drag offset + velocity.
  - Maintain or improve **TypeScript types**.
  - Prefer **clear, well‑named variables** over comments; comments should only explain non‑obvious intent or trade‑offs.

You are free to **rewrite the carousel internals** as long as the public props & overall behavior remain compatible (or are easily adaptable).

---

## Files to consider and refactor

These are all the current TypeScript/TSX files that participate in the “screen” experience and drag behavior.

> Please read all of them first to understand the current architecture, then propose and implement a cohesive refactor focused on the carousel / drag / layout logic.

### `components/browser/browser-content.tsx`

```tsx
```tsx
import { useState } from "react";
import { MacOSDock } from "@/components/screens/macos-dock";
import { ScreenCarousel } from "@/components/screens/screen-carousel";
import type { Screen, ScreenType } from "@/components/screens/types";

interface BrowserContentProps {
  sidebarExpanded: boolean;
}

export function BrowserContent({ sidebarExpanded }: BrowserContentProps) {
  const [activeScreenType, setActiveScreenType] =
    useState<ScreenType>("welcome");
  const [screens, setScreens] = useState<Screen[]>([
    {
      id: "welcome",
      type: "welcome",
      title: "Welcome",
      width: 0, // Will be set to full width
      height: 0, // Will be set to full height
    },
    {
      id: "terminal",
      type: "terminal",
      title: "Terminal",
      width: 0,
      height: 0,
    },
    {
      id: "code",
      type: "code",
      title: "Code Editor",
      width: 0,
      height: 0,
    },
    {
      id: "browser",
      type: "browser",
      title: "Browser",
      width: 0,
      height: 0,
    },
  ]);

  const handleScreenResize = (id: string, width: number, height: number) => {
    setScreens((prev) =>
      prev.map((screen) =>
        screen.id === id ? { ...screen, width, height } : screen,
      ),
    );
  };

  const handleAddScreen = () => {
    const newScreen: Screen = {
      id: `screen-${Date.now()}`,
      type: "welcome",
      title: `New Screen ${screens.length + 1}`,
      // Use 0 so the carousel's initialization logic
      // will promote this to full container width/height.
      width: 0,
      height: 0,
    };
    setScreens([...screens, newScreen]);
    setActiveScreenType(newScreen.type);
  };

  const handleToggleViewMode = () => {
    // TODO: Implement grid view mode
    console.log("Toggle view mode");
  };

  return (
    <div className="relative flex h-full w-full items-center justify-center overflow-hidden">
      <MacOSDock
        activeScreen={activeScreenType}
        onScreenChange={setActiveScreenType}
        onAddScreen={handleAddScreen}
        onToggleViewMode={handleToggleViewMode}
        sidebarExpanded={sidebarExpanded}
      />
      <ScreenCarousel
        activeScreenType={activeScreenType}
        screens={screens}
        onScreenChange={setActiveScreenType}
        onScreenResize={handleScreenResize}
        onScreensUpdate={setScreens}
        sidebarExpanded={sidebarExpanded}
      />
    </div>
  );
}
```

### `components/screens/types.ts`

```tsx
export type ScreenType = "browser" | "terminal" | "code" | "welcome";

export interface Screen {
  id: string;
  type: ScreenType;
  title: string;
  width: number;
  height: number;
}

export interface ScreenDimensions {
  width: number;
  height: number;
}

export interface ScreenCarouselDragConfig {
  /**
   * Gap in pixels between individual screens in the carousel.
   */
  gap: number;
  /**
   * Minimum horizontal swipe velocity before we treat an interaction
   * as an intentional "fling" to the next / previous screen.
   */
  minSwipeVelocity: number;
  /**
   * Minimum fraction of the container width that must be dragged
   * (in the absence of strong velocity) before we switch screens.
   */
  minSwipeOffsetRatio: number;
  /**
   * How "stretchy" the drag is at the edges of the carousel.
   * Lower values feel heavier / more locked-in.
   */
  dragElastic: number;
}

export const defaultScreenCarouselDragConfig: ScreenCarouselDragConfig = {
  gap: 8,
  minSwipeVelocity: 900,
  minSwipeOffsetRatio: 0.35,
  dragElastic: 0.08,
};
```

### `components/screens/screen-carousel.tsx`

```tsx
"use client";

import { animate, motion, useMotionValue } from "motion/react";
import { Resizable } from "re-resizable";
import { useEffect, useRef, useState } from "react";
import { cn } from "@/lib/utils";
import { BrowserScreen } from "./browser-screen";
import { CodeScreen } from "./code-screen";
import { TerminalScreen } from "./terminal-screen";
import type {
  Screen,
  ScreenCarouselDragConfig,
  ScreenType,
} from "./types";
import { defaultScreenCarouselDragConfig } from "./types";
import { WelcomeScreen } from "./welcome-screen";

interface ScreenCarouselProps {
  activeScreenType: ScreenType;
  screens: Screen[];
  onScreenChange: (type: ScreenType) => void;
  onScreenResize: (id: string, width: number, height: number) => void;
  onScreensUpdate: (screens: Screen[]) => void;
  sidebarExpanded: boolean;
}

export function ScreenCarousel({
  activeScreenType,
  screens,
  onScreenChange,
  onScreenResize,
  onScreensUpdate,
  sidebarExpanded,
}: ScreenCarouselProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [containerSize, setContainerSize] = useState({ width: 0, height: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [isResizing, setIsResizing] = useState(false);
  const initializedRef = useRef(false);
  const previousContainerWidthRef = useRef(0);
  const resizeTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const x = useMotionValue(0);

  const dragConfig: ScreenCarouselDragConfig = defaultScreenCarouselDragConfig;

  const activeIndex = screens.findIndex((s) => s.type === activeScreenType);

  // Initialize container size and keep it in sync with layout changes
  // (including sidebar expand/collapse) using ResizeObserver.
  useEffect(() => {
    if (!containerRef.current) return;

    const element = containerRef.current;

    const updateSize = () => {
      const rect = element.getBoundingClientRect();
      const newSize = {
        width: rect.width,
        height: rect.height,
      };
      setContainerSize((prev) =>
        prev.width === newSize.width && prev.height === newSize.height
          ? prev
          : newSize,
      );
    };

    updateSize();

    const resizeObserver = new ResizeObserver(() => {
      updateSize();
    });

    resizeObserver.observe(element);

    return () => {
      resizeObserver.disconnect();
    };
  }, []);

  // Initialize screens when container size is available
  useEffect(() => {
    if (
      !initializedRef.current &&
      containerSize.width > 0 &&
      containerSize.height > 0
    ) {
      const logicalWidth = Math.max(containerSize.width - 32, 400);
      const logicalHeight = Math.max(containerSize.height - 32, 300);
      // On the very first layout pass, force every screen to
      // occupy (almost) the full container, leaving a small
      // margin so the screens don't feel "overflowed" or too
      // heavy against the edges.
      initializedRef.current = true;
      previousContainerWidthRef.current = containerSize.width;
      onScreensUpdate(
        screens.map((screen) => ({
          ...screen,
          width: logicalWidth,
          height: logicalHeight,
        })),
      );
    } else if (
      initializedRef.current &&
      containerSize.width > 0 &&
      Math.abs(containerSize.width - previousContainerWidthRef.current) > 10
    ) {
      // Recalculate when container width changes significantly (e.g., sidebar toggle)
      const logicalWidth = Math.max(containerSize.width - 32, 400);
      const logicalHeight = Math.max(containerSize.height - 32, 300);
      previousContainerWidthRef.current = containerSize.width;
      onScreensUpdate(
        screens.map((screen) => ({
          ...screen,
          width: logicalWidth,
          height: logicalHeight,
        })),
      );
    }
  }, [containerSize, screens, onScreensUpdate]);

  // We still receive sidebarExpanded so the carousel can react to it
  // in the future if needed, but width/height are now driven purely
  // by the actual container size via the ResizeObserver above.

  useEffect(() => {
    if (!isDragging && !isResizing && containerSize.width > 0) {
      const gap = dragConfig.gap; // gap-2 = 8px
      const activeScreen = screens[activeIndex];
      const activeScreenWidth = activeScreen
        ? activeScreen.width
        : containerSize.width;

      // Keep active screen centered when it's smaller than container
      let targetX = -activeIndex * (containerSize.width + gap);

      if (activeScreenWidth < containerSize.width) {
        const offset = (containerSize.width - activeScreenWidth) / 2;
        targetX += offset;
      }

      animate(x, targetX, {
        type: "spring",
        stiffness: 260,
        damping: 32,
      });
    }
  }, [activeIndex, containerSize.width, isDragging, isResizing, screens, x]);

  const handleDragEnd = (_event: any, info: any) => {
    setIsDragging(false);
    const offset = info.offset.x;
    const velocity = info.velocity.x;

    let newIndex = activeIndex;

    const direction = offset < 0 ? 1 : -1;
    const swipePower =
      Math.abs(offset) * 0.5 + Math.abs(velocity) * 0.8;

    if (swipePower > dragConfig.minSwipeVelocity) {
      // Strong, fast swipe changes screen regardless of exact distance,
      // but still respects direction and bounds.
      newIndex = activeIndex + direction;
    } else if (
      Math.abs(offset) >
      containerSize.width * dragConfig.minSwipeOffsetRatio
    ) {
      // Slower drags must pull a significant portion of the width
      // before we change the active screen.
      newIndex = offset > 0 ? activeIndex - 1 : activeIndex + 1;
    }

    newIndex = Math.max(0, Math.min(screens.length - 1, newIndex));

    if (newIndex !== activeIndex) {
      onScreenChange(screens[newIndex].type);
    }
  };

  const renderScreen = (screen: Screen) => {
    switch (screen.type) {
      case "terminal":
        return <TerminalScreen />;
      case "code":
        return <CodeScreen />;
      case "browser":
        return <BrowserScreen />;
      case "welcome":
        return <WelcomeScreen />;
      default:
        return <WelcomeScreen />;
    }
  };

  return (
    <div ref={containerRef} className="relative h-full w-full overflow-hidden">
      <motion.div
        className="flex h-full gap-2"
        style={{ x }}
        drag={!isResizing ? "x" : false}
        dragConstraints={{
          left: -(screens.length - 1) * (containerSize.width + dragConfig.gap),
          right: 0,
        }}
        dragElastic={dragConfig.dragElastic}
        onDragStart={() => setIsDragging(true)}
        onDragEnd={handleDragEnd}
      >
        {screens.map((screen, index) => {
          const isActive = index === activeIndex;
          const screenWidth = isActive ? screen.width : Math.max(containerSize.width - 32, 400);
          const screenHeight = isActive ? screen.height : Math.max(containerSize.height - 32, 300);

          return (
            <div
              key={screen.id}
              className="flex items-center justify-center shrink-0"
              style={{
                width: screenWidth,
                height: containerSize.height,
              }}
            >
              {isActive ? (
                <Resizable
                  size={{ width: screenWidth, height: screenHeight }}
                  onResizeStart={() => setIsResizing(true)}
                  onResize={(_e, _direction, _ref, d) => {
                    // Throttle updates during resize to avoid jitter
                    if (resizeTimeoutRef.current) {
                      clearTimeout(resizeTimeoutRef.current);
                    }

                    const newWidth = screenWidth + d.width;
                    const newHeight = screenHeight + d.height;

                    // Update position immediately for smooth feedback
                    const gap = dragConfig.gap;
                    let targetX = -activeIndex * (containerSize.width + gap);

                    if (newWidth < containerSize.width) {
                      const offset = (containerSize.width - newWidth) / 2;
                      targetX += offset;
                    }

                    x.set(targetX);

                    // Throttle state updates
                    resizeTimeoutRef.current = setTimeout(() => {
                      onScreenResize(screen.id, newWidth, newHeight);
                    }, 16); // ~60fps
                  }}
                  onResizeStop={(_e, _direction, _ref, d) => {
                    if (resizeTimeoutRef.current) {
                      clearTimeout(resizeTimeoutRef.current);
                    }
                    setIsResizing(false);
                    // Final update
                    onScreenResize(
                      screen.id,
                      screenWidth + d.width,
                      screenHeight + d.height,
                    );
                  }}
                  minWidth={400}
                  minHeight={300}
                  maxWidth={Math.max(containerSize.width - 32, 400)}
                  maxHeight={Math.max(containerSize.height - 32, 300)}
                  className="relative"
                  style={{ width: screenWidth, height: screenHeight }}
                  enable={{
                    top: true,
                    right: true,
                    bottom: true,
                    left: true,
                    topRight: true,
                    bottomRight: true,
                    bottomLeft: true,
                    topLeft: true,
                  }}
                >
                  <motion.div
                    initial={{ scale: 0.95, opacity: 0 }}
                    animate={{ scale: 1, opacity: 1 }}
                    className="h-full w-full"
                  >
                    {renderScreen(screen)}
                  </motion.div>
                </Resizable>
              ) : (
                <div
                  className="h-full w-full"
                  style={{ width: screenWidth, height: screenHeight }}
                >
                  {renderScreen(screen)}
                </div>
              )}
            </div>
          );
        })}
      </motion.div>
      {/* Screen Indicators */}
      <div className="absolute bottom-4 left-1/2 -translate-x-1/2 flex gap-2">
        {screens.map((screen, index) => (
          <button
            key={screen.id}
            onClick={() => onScreenChange(screen.type)}
            className={cn(
              "h-2 rounded-full transition-all",
              index === activeIndex
                ? "w-8 bg-primary"
                : "w-2 bg-muted-foreground/30 hover:bg-muted-foreground/50",
            )}
          />
        ))}
      </div>
    </div>
  );
}
```

### `components/screens/macos-dock.tsx`

```tsx
"use client";

import { Code, Globe, LayoutGrid, Plus, Terminal } from "lucide-react";
import { Dock, DockIcon } from "@/components/ui/dock";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";
import type { ScreenType } from "./types";

interface MacOSDockProps {
  activeScreen: ScreenType;
  onScreenChange: (type: ScreenType) => void;
  onAddScreen: () => void;
  onToggleViewMode: () => void;
  sidebarExpanded: boolean;
}

export function MacOSDock({
  activeScreen,
  onScreenChange,
  onAddScreen,
  onToggleViewMode,
  sidebarExpanded,
}: MacOSDockProps) {
  const sidebarWidth = sidebarExpanded ? 360 : 56;

  return (
    <div
      className="pointer-events-none fixed top-4 z-50 flex justify-center transition-all duration-200"
      style={{
        left: `${sidebarWidth}px`,
        right: 0,
      }}
    >
      <div className="pointer-events-auto">
        <Dock iconMagnification={60} iconDistance={100}>
          <DockIcon
            className={cn(
              "transition-all",
              activeScreen === "terminal"
                ? "bg-primary text-primary-foreground"
                : "bg-black/10 dark:bg:white/10 hover:bg-black/20 dark:hover:bg-white/20",
            )}
            onClick={() => onScreenChange("terminal")}
          >
            <Terminal className="h-6 w-6" />
          </DockIcon>
          <DockIcon
            className={cn(
              "transition-all",
              activeScreen === "code"
                ? "bg-primary text-primary-foreground"
                : "bg-black/10 dark:bg-white/10 hover:bg-black/20 dark:hover:bg-white/20",
            )}
            onClick={() => onScreenChange("code")}
          >
            <Code className="h-6 w-6" />
          </DockIcon>
          <DockIcon
            className={cn(
              "transition-all",
              activeScreen === "browser"
                ? "bg-primary text-primary-foreground"
                : "bg-black/10 dark:bg-white/10 hover:bg-black/20 dark:hover:bg:white/20",
            )}
            onClick={() => onScreenChange("browser")}
          >
            <Globe className="h-6 w-6" />
          </DockIcon>

          <Separator orientation="vertical" className="h-full py-2" />

          <DockIcon
            className="bg-black/10 dark:bg-white/10 hover:bg-black/20 dark:hover:bg:white/20"
            onClick={onAddScreen}
          >
            <Plus className="h-6 w-6" />
          </DockIcon>
          <DockIcon
            className="bg-black/10 dark:bg:white/10 hover:bg-black/20 dark:hover:bg:white/20"
            onClick={onToggleViewMode}
          >
            <LayoutGrid className="h-6 w-6" />
          </DockIcon>
        </Dock>
      </div>
    </div>
  );
}
```

### `components/screens/welcome-screen.tsx`

```tsx
"use client";

import { motion } from "motion/react";
import { DockDemo } from "@/components/dock";
import { ThemeSwitcher } from "@/components/theme-switcher";

export function WelcomeScreen() {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      className="flex h-full w-full items-center justify-center rounded-[20px] border border-border bg-background p-8 shadow-2xl overflow-hidden"
    >
      <div className="text-center space-y-6">
        <div>
          <h1 className="text-accent-foreground mb-2 text-4xl font-bold">
            Zen Browser
          </h1>
          <p className="text-muted-foreground">
            Experience the web with vertical tabs and workspaces
          </p>
        </div>
        <ThemeSwitcher />
        <DockDemo />
      </div>
    </motion.div>
  );
}
```

### `components/screens/terminal-screen.tsx`

```tsx
"use client";

import { motion } from "motion/react";

export function TerminalScreen() {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className="flex h-full w-full flex-col rounded-[20px] border border-border bg-black/95 p-4 font-mono text-sm shadow-2xl overflow-hidden"
    >
      <div className="mb-2 flex items-center gap-2">
        <div className="h-3 w-3 rounded-full bg-red-500" />
        <div className="h-3 w-3 rounded-full bg-yellow-500" />
        <div className="h-3 w-3 rounded-full bg-green-500" />
        <span className="ml-2 text-xs text-muted-foreground">Terminal</span>
      </div>
      <div className="flex-1 overflow-auto">
        <div className="space-y-1">
          <div className="text-green-400">
            <span className="text-blue-400">user@zen</span>
            <span className="text-white">:</span>
            <span className="text-cyan-400">~</span>
            <span className="text-white">$ </span>
            <span className="text-white">Welcome to Zen Browser Terminal</span>
          </div>
          <div className="text-muted-foreground">Type commands here...</div>
          <div className="mt-4 text-green-400">
            <span className="text-blue-400">user@zen</span>
            <span className="text-white">:</span>
            <span className="text-cyan-400">~</span>
            <span className="text-white">$ </span>
            <span className="animate-pulse">_</span>
          </div>
        </div>
      </div>
    </motion.div>
  );
}
```

### `components/screens/browser-screen.tsx`

```tsx
"use client";

import { Globe } from "lucide-react";
import { motion } from "motion/react";

export function BrowserScreen() {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className="flex h-full w-full flex-col rounded-[20px] border border-border bg-card shadow-2xl overflow-hidden"
    >
      <div className="flex items-center gap-2 border-b border-border p-3">
        <div className="h-3 w-3 rounded-full bg-red-500" />
        <div className="h-3 w-3 rounded-full bg-yellow-500" />
        <div className="h-3 w-3 rounded-full bg-green-500" />
        <div className="ml-2 flex flex-1 items-center gap-2 rounded-md bg-accent px-3 py-1.5">
          <Globe className="h-4 w-4 text-muted-foreground" />
          <span className="text-sm text-muted-foreground">
            https://zen-browser.app
          </span>
        </div>
      </div>
      <div className="flex flex-1 items-center justify-center p-8">
        <div className="text-center space-y-4">
          <div className="mx-auto h-16 w-16 rounded-full bg-primary/10 flex items-center justify-center">
            <Globe className="h-8 w-8 text-primary" />
          </div>
          <h2 className="text-2xl font-bold text-foreground">Browser View</h2>
          <p className="text-muted-foreground">Your web browsing experience</p>
        </div>
      </div>
    </motion.div>
  );
}
```

### `components/screens/code-screen.tsx`

```tsx
"use client";

import { motion } from "motion/react";

export function CodeScreen() {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className="flex h-full w-full flex-col rounded-[20px] border border-border bg-[#1e1e1e] p-4 font-mono text-sm shadow-2xl overflow-hidden"
    >
      <div className="mb-2 flex items-center gap-2">
        <div className="h-3 w-3 rounded-full bg-red-500" />
        <div className="h-3 w-3 rounded-full bg-yellow-500" />
        <div className="h-3 w-3 rounded-full bg-green-500" />
        <span className="ml-2 text-xs text-muted-foreground">Code Editor</span>
      </div>
      <div className="flex-1 overflow-auto">
        <div className="space-y-1 text-sm">
          <div>
            <span className="text-purple-400">import</span>
            <span className="text-white"> {"{"} </span>
            <span className="text-blue-300">useState</span>
            <span className="text-white"> {"}"} </span>
            <span className="text-purple-400">from</span>
            <span className="text-orange-300"> "react"</span>
            <span className="text-white">;</span>
          </div>
          <div className="mt-4">
            <span className="text-purple-400">export</span>
            <span className="text-purple-400"> function</span>
            <span className="text-yellow-300"> App</span>
            <span className="text-white">() {"{"}</span>
          </div>
          <div className="ml-4">
            <span className="text-purple-400">const</span>
            <span className="text-white"> [</span>
            <span className="text-blue-300">count</span>
            <span className="text-white">, </span>
            <span className="text-blue-300">setCount</span>
            <span className="text-white">] = </span>
            <span className="text-yellow-300">useState</span>
            <span className="text-white">(</span>
            <span className="text-orange-300">0</span>
            <span className="text-white">);</span>
          </div>
          <div className="mt-4 ml-4">
            <span className="text-purple-400">return</span>
            <span className="text-white"> (</span>
          </div>
          <div className="ml-8">
            <span className="text-gray-500">&lt;</span>
            <span className="text-green-400">div</span>
            <span className="text-gray-500">&gt;</span>
          </div>
          <div className="ml-12">
            <span className="text-gray-500">&lt;</span>
            <span className="text-green-400">h1</span>
            <span className="text-gray-500">&gt;</span>
            <span className="text-white">Zen Browser Code Editor</span>
            <span className="text-gray-500">&lt;/</span>
            <span className="text-green-400">h1</span>
            <span className="text-gray-500">&gt;</span>
          </div>
          <div className="ml-8">
            <span className="text-gray-500">&lt;/</span>
            <span className="text-green-400">div</span>
            <span className="text-gray-500">&gt;</span>
          </div>
          <div className="ml-4">
            <span className="text-white">);</span>
          </div>
          <div>
            <span className="text-white">{"}"}</span>
          </div>
        </div>
      </div>
    </motion.div>
  );
}
```

### `components/screens/index.ts`

```tsx
export * from "./browser-screen";
export * from "./code-screen";
export * from "./macos-dock";
export * from "./screen-carousel";
export * from "./terminal-screen";
export * from "./types";
export * from "./welcome-screen";
```

---

## What I want from you (the AI reading this file)

1. **Carefully analyze** the existing drag, layout, and resize logic in `screen-carousel.tsx` and how it interacts with:
   - `BrowserContent` (state source of truth for screens and active screen type).
   - The different `*Screen` components.
   - `MacOSDock` and the sidebar’s expanded/collapsed state.
2. **Design a cleaner, more robust architecture** for:
   - Calculating container size.
   - Deriving each screen’s size.
   - Computing the carousel `x` offset for the active index.
   - Determining the next index from drag offset + velocity.
3. **Refactor or rewrite `ScreenCarousel`** (and related types/utilities) to implement that design, improving:
   - Correctness (no weird off‑screen behavior).
   - Feel (weighted, natural drag).
   - Maintainability (clear functions, minimal duplication, strong types).
4. Return the **full updated code** for all files you change, plus any **new helper hooks/utilities** you introduce, so I can apply the patch directly to my codebase.
