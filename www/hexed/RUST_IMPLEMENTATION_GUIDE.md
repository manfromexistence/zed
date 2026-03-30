# Rust Implementation Guide: Browser Screen System

## Table of Contents
1. [Overview](#overview)
2. [Core Data Structures](#core-data-structures)
3. [Application Architecture](#application-architecture)
4. [Screen Carousel System](#screen-carousel-system)
5. [Advanced Resizing with Directional Gravity](#advanced-resizing-with-directional-gravity)
6. [Circular Wrapping Logic](#circular-wrapping-logic)
7. [Sidebar System](#sidebar-system)
8. [Drag-and-Drop System](#drag-and-drop-system)
9. [State Management](#state-management)
10. [Constants and Configuration](#constants-and-configuration)

---

## Overview

This document provides a complete specification for implementing a browser-like interface with advanced screen management in Rust. The system features:

- **Multiple screen types**: Terminal, Code Editor, Browser, Welcome, and Custom screens
- **Advanced resizing**: Directional gravity system that reveals adjacent screens based on resize direction
- **Circular wrapping**: First screen connects to last screen and vice versa
- **Per-screen resize tracking**: Each screen maintains independent resize state
- **Sidebar management**: Collapsible sidebar with workspace/tab organization
- **macOS-style UI**: Dock navigation, rounded corners, window controls
- **Drag-and-drop**: Full DnD support for tabs, folders, and logos

---

## Core Data Structures

### Screen Types

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenType {
    Browser,
    Terminal,
    Code,
    Welcome,
    Custom,
}

#[derive(Debug, Clone)]
pub struct Screen {
    pub id: String,
    pub screen_type: ScreenType,
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub dock_icon: Option<String>, // Lucide icon name for custom screens
}

impl Screen {
    pub fn new(id: String, screen_type: ScreenType, title: String) -> Self {
        Self {
            id,
            screen_type,
            title,
            width: 0.0,  // Will be initialized to container size
            height: 0.0,
            dock_icon: None,
        }
    }
}
```

### Browser Data Structures

```rust
#[derive(Debug, Clone)]
pub struct Tab {
    pub id: String,
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub pinned: bool,
    pub workspace_id: String,
    pub folder_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TabFolder {
    pub id: String,
    pub name: String,
    pub collapsed: bool,
    pub tabs: Vec<Tab>,
    pub workspace_id: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconType {
    Emoji,
    Icon,
    Dot,
}

#[derive(Debug, Clone)]
pub struct WorkspaceIcon {
    pub icon_type: IconType,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub color: String,
    pub icon: WorkspaceIcon,
}

#[derive(Debug, Clone)]
pub struct SVGLogo {
    pub id: u32,
    pub title: String,
    // In Rust, you'd store an enum or trait object for the component
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropPosition {
    Before,
    After,
    Inside,
}
```

---

## Application Architecture

### Main Application State

```rust
pub struct AppState {
    // Sidebar state
    pub sidebar_expanded: bool,
    
    // Workspace management
    pub workspaces: Vec<Workspace>,
    pub active_workspace: String,
    pub workspace_scroll_position: usize,
    
    // Tab and folder management
    pub folders: Vec<TabFolder>,
    pub loose_tabs: Vec<Tab>,
    pub active_tab: String,
    
    // Logo container
    pub logos: Vec<SVGLogo>,
    
    // Screen management
    pub screens: Vec<Screen>,
    pub active_screen_id: String,
    
    // UI state
    pub space_collapsed: bool,
    pub downloads_open: bool,
    pub plus_menu_open: bool,
    pub command_open: bool,
    pub workspace_dialog_open: bool,
    pub media_playing: bool,
    
    // Drag and drop state
    pub active_drag_id: Option<String>,
    pub over_id: Option<String>,
    pub drop_position: Option<DropPosition>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            sidebar_expanded: true,
            workspaces: vec![
                Workspace {
                    id: "1".to_string(),
                    name: "Personal".to_string(),
                    color: "hsl(var(--chart-1))".to_string(),
                    icon: WorkspaceIcon {
                        icon_type: IconType::Emoji,
                        value: "😊".to_string(),
                    },
                },
            ],
            active_workspace: "1".to_string(),
            workspace_scroll_position: 0,
            folders: vec![],
            loose_tabs: vec![],
            active_tab: String::new(),
            logos: vec![],
            screens: vec![
                Screen::new("welcome".to_string(), ScreenType::Welcome, "Welcome".to_string()),
                Screen::new("terminal".to_string(), ScreenType::Terminal, "Terminal".to_string()),
                Screen::new("code".to_string(), ScreenType::Code, "Code Editor".to_string()),
                Screen::new("browser".to_string(), ScreenType::Browser, "Browser".to_string()),
            ],
            active_screen_id: "welcome".to_string(),
            space_collapsed: false,
            downloads_open: false,
            plus_menu_open: false,
            command_open: false,
            workspace_dialog_open: false,
            media_playing: true,
            active_drag_id: None,
            over_id: None,
            drop_position: None,
        }
    }
}
```

---

## Screen Carousel System

### Core Carousel State

```rust
pub struct ScreenCarouselState {
    pub container_size: (f64, f64), // (width, height)
    pub is_dragging: bool,
    pub is_resizing: bool,
    pub resize_counter: u32,
    pub resize_direction: Option<ResizeDirection>,
    pub resize_sides: ResizeSides,
    pub manually_resized_screens: HashSet<String>,
    pub resizing_dimensions: Option<(f64, f64)>, // (width, height) during resize
    pub x_position: f64, // Current carousel X translation
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeDirection {
    Left,
    Right,
    Both,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ResizeSides {
    pub left: bool,
    pub right: bool,
}
```

### Constants

```rust
pub const GAP: f64 = 8.0; // Gap between screens in pixels
pub const MIN_SCREEN_WIDTH: f64 = 400.0;
pub const MIN_SCREEN_HEIGHT: f64 = 300.0;
```

### Container Size Tracking

The carousel uses a ResizeObserver pattern to track container size changes:

1. **Initial Setup**: When the container mounts, measure its dimensions
2. **Sidebar Toggle**: When sidebar expands/collapses, container width changes
3. **Window Resize**: When browser window resizes, container dimensions change

**Implementation Logic**:

```rust
fn update_container_size(&mut self, new_width: f64, new_height: f64) {
    let size_changed = 
        (self.container_size.0 - new_width).abs() > 10.0 ||
        (self.container_size.1 - new_height).abs() > 10.0;
    
    if size_changed {
        self.container_size = (new_width, new_height);
        self.sync_screen_dimensions();
    }
}

fn sync_screen_dimensions(&mut self) {
    for screen in &mut self.screens {
        let was_manually_resized = self.manually_resized_screens.contains(&screen.id);
        
        if screen.width == 0.0 || screen.height == 0.0 {
            // Uninitialized screen - set to full container size
            screen.width = self.container_size.0;
            screen.height = self.container_size.1;
        } else if was_manually_resized {
            // Keep current size, just ensure within bounds
            screen.width = screen.width.min(self.container_size.0);
            screen.height = screen.height.min(self.container_size.1);
        } else {
            // Reset to full width (sidebar toggle case)
            screen.width = self.container_size.0;
            screen.height = self.container_size.1;
        }
    }
}
```

### Screen Positioning Logic

Each screen is positioned in a horizontal layout with gaps:

```rust
fn calculate_screen_position(&self, index: usize) -> f64 {
    // Each screen is positioned at: index * (container_width + GAP)
    index as f64 * (self.container_size.0 + GAP)
}

fn calculate_carousel_position(&self, active_index: usize) -> f64 {
    let active_screen = &self.screens[active_index];
    let left_edge = self.calculate_screen_position(active_index);
    let available_space = self.container_size.0 - active_screen.width;
    
    // Apply directional gravity
    match self.resize_direction {
        Some(ResizeDirection::Both) => {
            // Resized from both sides - center to show both adjacent screens
            -left_edge + available_space / 2.0
        }
        Some(ResizeDirection::Left) => {
            // Resized from left - push screen right to reveal left screen
            -left_edge + available_space
        }
        Some(ResizeDirection::Right) => {
            // Resized from right - push screen left to reveal right screen
            -left_edge
        }
        None => {
            // Default: push to left
            -left_edge
        }
    }
}
```

---

## Advanced Resizing with Directional Gravity

### The Gravity System

The directional gravity system is the most complex feature. It determines how screens are positioned based on which side(s) the user resizes from.

**Key Concepts**:
1. **Track Both Sides**: Independently track left and right resize operations
2. **Determine Direction**: Calculate overall direction (left, right, or both)
3. **Apply Gravity**: Position screen based on direction to reveal adjacent screens
4. **Live Animation**: Update wrapper width during resize for smooth transitions

### Resize Event Handlers

```rust
impl ScreenCarouselState {
    pub fn on_resize_start(&mut self, screen_id: &str, current_width: f64, current_height: f64) {
        self.is_resizing = true;
        self.manually_resized_screens.insert(screen_id.to_string());
        self.resizing_dimensions = Some((current_width, current_height));
    }
    
    pub fn on_resize(&mut self, 
                     screen_id: &str,
                     direction: &str, 
                     delta_width: f64, 
                     delta_height: f64,
                     current_width: f64,
                     current_height: f64) {
        
        let new_width = current_width + delta_width;
        let new_height = current_height + delta_height;
        
        // Track which sides are being resized
        let is_resizing_left = direction.contains("left");
        let is_resizing_right = direction.contains("right");
        
        if is_resizing_left {
            self.resize_sides.left = true;
        }
        if is_resizing_right {
            self.resize_sides.right = true;
        }
        
        // Determine overall direction
        if self.resize_sides.left && self.resize_sides.right {
            self.resize_direction = Some(ResizeDirection::Both);
        } else if is_resizing_left && !is_resizing_right {
            self.resize_direction = Some(ResizeDirection::Left);
        } else if is_resizing_right && !is_resizing_left {
            self.resize_direction = Some(ResizeDirection::Right);
        }
        
        // Store dimensions for live update
        self.resizing_dimensions = Some((new_width, new_height));
        self.resize_counter += 1; // Trigger re-render
        
        // Update carousel position with gravity
        self.update_position_during_resize(new_width);
    }

    pub fn on_resize_stop(&mut self, 
                          screen_id: &str, 
                          final_width: f64, 
                          final_height: f64) {
        // Update the actual screen dimensions
        if let Some(screen) = self.screens.iter_mut().find(|s| s.id == screen_id) {
            screen.width = final_width;
            screen.height = final_height;
        }
        
        self.resizing_dimensions = None;
        self.is_resizing = false;
        self.resize_counter = 0;
    }
    
    fn update_position_during_resize(&mut self, new_width: f64) {
        let active_index = self.get_active_screen_index();
        let left_edge = self.calculate_screen_position(active_index);
        let available_space = self.container_size.0 - new_width;
        
        let target_x = match (self.resize_sides.left, self.resize_sides.right) {
            (true, true) => {
                // Both sides - center
                -left_edge + available_space / 2.0
            }
            (true, false) => {
                // Left only - push right
                -left_edge + available_space
            }
            (false, true) => {
                // Right only - push left
                -left_edge
            }
            (false, false) => {
                // Corners or top/bottom - center
                -left_edge + available_space / 2.0
            }
        };
        
        self.x_position = target_x;
    }
}
```

### Critical Implementation Details

**1. Side Tracking Persistence**:
- `resize_sides` is NOT reset during a single resize operation
- Both `left` and `right` can be true simultaneously
- This allows detecting when user resizes from both sides in one session

**2. Live Wrapper Width Update**:
- During resize, the wrapper width must update in real-time
- Use `resizing_dimensions` ref to avoid state update delays
- Increment `resize_counter` to force re-renders
- This creates smooth animation as adjacent screens slide in

**3. Disable Drag During Resize**:
- When `is_resizing` is true, disable carousel drag
- Prevents weird movement conflicts
- Re-enable drag when resize completes

**4. Per-Screen Resize State**:
- `manually_resized_screens` is a HashSet of screen IDs
- Each screen tracks its own resize state independently
- Resizing one screen doesn't affect others
- When switching screens, resize direction resets

---

## Circular Wrapping Logic

### Concept

The first screen can reveal the last screen on its left side, and the last screen can reveal the first screen on its right side. This creates a circular carousel effect.

### Implementation

```rust
fn should_show_wrapped_last_screen(&self, active_index: usize) -> bool {
    active_index == 0 
        && self.resize_direction.is_some()
        && matches!(
            self.resize_direction, 
            Some(ResizeDirection::Left) | Some(ResizeDirection::Both)
        )
        && self.screens.len() > 1
}

fn should_show_wrapped_first_screen(&self, active_index: usize) -> bool {
    active_index == self.screens.len() - 1
        && self.resize_direction.is_some()
        && matches!(
            self.resize_direction,
            Some(ResizeDirection::Right) | Some(ResizeDirection::Both)
        )
        && self.screens.len() > 1
}
```

### Rendering Wrapped Screens

```rust
fn render_carousel(&self) -> Vec<ScreenElement> {
    let mut elements = Vec::new();
    let active_index = self.get_active_screen_index();
    
    // Render wrapped last screen before first screen
    if self.should_show_wrapped_last_screen(active_index) {
        let last_screen = &self.screens[self.screens.len() - 1];
        elements.push(ScreenElement {
            screen: last_screen.clone(),
            position: -(self.container_size.0 + GAP), // Absolute left position
            is_active: false,
            is_wrapped: true,
        });
    }
    
    // Render all normal screens
    for (index, screen) in self.screens.iter().enumerate() {
        let is_active = index == active_index;
        let position = self.calculate_screen_position(index);
        
        elements.push(ScreenElement {
            screen: screen.clone(),
            position,
            is_active,
            is_wrapped: false,
        });
    }
    
    // Render wrapped first screen after last screen
    if self.should_show_wrapped_first_screen(active_index) {
        let first_screen = &self.screens[0];
        let last_position = self.calculate_screen_position(self.screens.len() - 1);
        elements.push(ScreenElement {
            screen: first_screen.clone(),
            position: last_position + self.container_size.0 + GAP,
            is_active: false,
            is_wrapped: true,
        });
    }
    
    elements
}
```

### Gap Consistency

All screens, including wrapped ones, must have the same 8px gap:

- Normal screens: `margin-right: GAP`
- Wrapped last screen: positioned at `-(container_width + GAP)` (includes gap)
- Wrapped first screen: positioned at `last_position + container_width + GAP` (includes gap)

### Click Navigation on Visible Screens

Non-active screens that are visible (due to resizing) should be clickable:

```rust
fn handle_screen_click(&mut self, screen_id: &str) {
    if screen_id != self.active_screen_id {
        self.active_screen_id = screen_id.to_string();
        // Reset resize tracking when changing screens
        self.resize_direction = None;
        self.resize_sides = ResizeSides::default();
    }
}
```

---

## Sidebar System

### Sidebar State

```rust
pub struct SidebarState {
    pub expanded: bool,
    pub space_collapsed: bool,
    pub downloads_open: bool,
    pub plus_menu_open: bool,
    pub command_open: bool,
}

impl SidebarState {
    pub fn width(&self) -> f64 {
        if self.expanded { 360.0 } else { 56.0 }
    }
}
```

### Sidebar Components

**1. Collapsed Sidebar (56px width)**:
- Vertical workspace icons
- Scroll arrows if more than MAX_VISIBLE_WORKSPACES (5)
- Plus button for new workspace
- Downloads button

**2. Expanded Sidebar (360px width)**:
- Search bar at top
- Logo container (max 12 logos)
- Space section (collapsible)
  - Workspace dropdown
  - Folders (collapsible, with tabs inside)
  - Context menu for workspace operations
- "New Tab" button with separator
- Scrollable tab list
- Media player (if playing)
- Bottom toolbar with workspace switcher

### Logo Container

The logo container is a special droppable area that:
- Holds up to 12 SVG logos
- Accepts tabs and folders dropped onto it
- Converts dropped items to logos
- Logos can be dragged out to create new tabs
- Has visual feedback when hovered during drag

```rust
pub const MAX_LOGOS: usize = 12;

fn handle_drop_on_logo_container(&mut self, dragged_item: DraggedItem) {
    if self.logos.len() >= MAX_LOGOS {
        // Show error: container is full
        return;
    }
    
    match dragged_item {
        DraggedItem::Tab(tab) => {
            // Create logo from tab
            let logo = SVGLogo {
                id: generate_id(),
                title: tab.title.clone(),
            };
            self.logos.push(logo);
            
            // Remove tab from its location
            self.remove_tab(&tab.id);
        }
        DraggedItem::Folder(folder) => {
            // Create logo from folder
            let logo = SVGLogo {
                id: generate_id(),
                title: folder.name.clone(),
            };
            self.logos.push(logo);
            // Folder remains in place
        }
        _ => {}
    }
}
```

### Workspace Management

```rust
pub const MAX_VISIBLE_WORKSPACES: usize = 5;

impl AppState {
    pub fn visible_workspaces(&self) -> &[Workspace] {
        let start = self.workspace_scroll_position;
        let end = (start + MAX_VISIBLE_WORKSPACES).min(self.workspaces.len());
        &self.workspaces[start..end]
    }
    
    pub fn can_scroll_left(&self) -> bool {
        self.workspace_scroll_position > 0
    }
    
    pub fn can_scroll_right(&self) -> bool {
        self.workspace_scroll_position < self.workspaces.len() - MAX_VISIBLE_WORKSPACES
    }
}
```

---

## Drag-and-Drop System

### Drag Types

```rust
#[derive(Debug, Clone)]
pub enum DraggedItem {
    Tab(Tab),
    Folder(TabFolder),
    Logo(SVGLogo),
}

#[derive(Debug, Clone)]
pub enum DropTarget {
    Tab(String),           // Tab ID
    Folder(String),        // Folder ID
    Logo(String),          // Logo ID (as string)
    LogoContainer,
    NewTabButton,
}
```

### Drop Position Logic

When dragging over an item, calculate drop position based on cursor offset:

```rust
fn calculate_drop_position(
    drag_type: &DraggedItem,
    over_type: &DropTarget,
    offset_x: f64,
    offset_y: f64,
) -> Option<DropPosition> {
    match (drag_type, over_type) {
        // Logo over logo: horizontal positioning
        (DraggedItem::Logo(_), DropTarget::Logo(_)) => {
            if offset_x < 0.0 {
                Some(DropPosition::Before)
            } else {
                Some(DropPosition::After)
            }
        }
        
        // Tab/Folder over folder: inside
        (DraggedItem::Tab(_), DropTarget::Folder(_)) |
        (DraggedItem::Folder(_), DropTarget::Folder(_)) => {
            Some(DropPosition::Inside)
        }
        
        // Tab over tab: vertical positioning
        (DraggedItem::Tab(_), DropTarget::Tab(_)) => {
            if offset_y < 0.0 {
                Some(DropPosition::Before)
            } else {
                Some(DropPosition::After)
            }
        }
        
        // Folder over folder: vertical positioning
        (DraggedItem::Folder(_), DropTarget::Folder(_)) => {
            if offset_y < 0.0 {
                Some(DropPosition::Before)
            } else {
                Some(DropPosition::After)
            }
        }
        
        _ => None,
    }
}
```

### Drag Event Handlers

```rust
impl AppState {
    pub fn on_drag_start(&mut self, item_id: &str, item_type: DraggedItem) {
        self.active_drag_id = Some(item_id.to_string());
    }
    
    pub fn on_drag_over(&mut self, over_id: &str, offset_x: f64, offset_y: f64) {
        self.over_id = Some(over_id.to_string());
        
        if let (Some(active_id), Some(over_id)) = (&self.active_drag_id, &self.over_id) {
            // Calculate drop position based on drag/drop types and offset
            self.drop_position = self.calculate_drop_position_for_current_drag(
                offset_x, 
                offset_y
            );
        }
    }
    
    pub fn on_drag_end(&mut self) {
        if let (Some(active_id), Some(over_id)) = 
            (self.active_drag_id.take(), self.over_id.take()) {
            
            if active_id != over_id {
                self.handle_drop(&active_id, &over_id, self.drop_position);
            }
        }
        
        self.drop_position = None;
    }
}
```

### Drop Handling Examples

**Logo Reordering**:
```rust
fn reorder_logos(&mut self, from_id: u32, to_id: u32) {
    let from_index = self.logos.iter().position(|l| l.id == from_id);
    let to_index = self.logos.iter().position(|l| l.id == to_id);
    
    if let (Some(from), Some(to)) = (from_index, to_index) {
        let logo = self.logos.remove(from);
        self.logos.insert(to, logo);
    }
}
```

**Tab into Folder**:
```rust
fn move_tab_to_folder(&mut self, tab_id: &str, folder_id: &str) {
    // Find and remove tab from current location
    let tab = self.remove_tab(tab_id);
    
    if let Some(mut tab) = tab {
        tab.folder_id = Some(folder_id.to_string());
        
        // Add to folder
        if let Some(folder) = self.folders.iter_mut().find(|f| f.id == folder_id) {
            folder.tabs.push(tab);
        }
    }
}
```

---

## State Management

### State Update Patterns

**Immutable Updates**:
```rust
// Update a screen's dimensions
pub fn update_screen_size(&mut self, screen_id: &str, width: f64, height: f64) {
    if let Some(screen) = self.screens.iter_mut().find(|s| s.id == screen_id) {
        screen.width = width;
        screen.height = height;
    }
}

// Update all screens
pub fn update_screens<F>(&mut self, mut updater: F)
where
    F: FnMut(&mut Screen),
{
    for screen in &mut self.screens {
        updater(screen);
    }
}
```

**Filtering and Mapping**:
```rust
// Get tabs for active workspace
pub fn active_workspace_tabs(&self) -> Vec<&Tab> {
    self.loose_tabs
        .iter()
        .filter(|tab| tab.workspace_id == self.active_workspace && tab.folder_id.is_none())
        .collect()
}

// Get folders for active workspace
pub fn active_workspace_folders(&self) -> Vec<&TabFolder> {
    self.folders
        .iter()
        .filter(|folder| folder.workspace_id == self.active_workspace && folder.parent_id.is_none())
        .collect()
}
```

### Helper Methods

```rust
impl AppState {
    pub fn remove_tab(&mut self, tab_id: &str) -> Option<Tab> {
        // Try loose tabs first
        if let Some(pos) = self.loose_tabs.iter().position(|t| t.id == tab_id) {
            return Some(self.loose_tabs.remove(pos));
        }
        
        // Search in folders
        for folder in &mut self.folders {
            if let Some(pos) = folder.tabs.iter().position(|t| t.id == tab_id) {
                return Some(folder.tabs.remove(pos));
            }
        }
        
        None
    }
    
    pub fn add_tab(&mut self, tab: Tab) {
        if let Some(folder_id) = &tab.folder_id {
            if let Some(folder) = self.folders.iter_mut().find(|f| f.id == folder_id) {
                folder.tabs.push(tab);
                return;
            }
        }
        self.loose_tabs.push(tab);
    }
}
```

---

## Constants and Configuration

### UI Constants

```rust
// Screen carousel
pub const GAP: f64 = 8.0;                    // Gap between screens
pub const MIN_SCREEN_WIDTH: f64 = 400.0;     // Minimum screen width
pub const MIN_SCREEN_HEIGHT: f64 = 300.0;    // Minimum screen height

// Sidebar
pub const SIDEBAR_EXPANDED_WIDTH: f64 = 360.0;
pub const SIDEBAR_COLLAPSED_WIDTH: f64 = 56.0;

// Workspace management
pub const MAX_VISIBLE_WORKSPACES: usize = 5;

// Logo container
pub const MAX_LOGOS: usize = 12;

// macOS styling
pub const WINDOW_BORDER_RADIUS: f64 = 20.0;  // macOS Tahoe style
pub const DOCK_TOP_OFFSET: f64 = 16.0;       // Dock distance from top
```

### Color Palette

```rust
pub const COLORS: &[&str] = &[
    "hsl(var(--chart-1))",
    "hsl(var(--chart-2))",
    "hsl(var(--chart-3))",
    "hsl(var(--chart-4))",
    "hsl(var(--chart-5))",
    "hsl(var(--primary))",
    "hsl(var(--destructive))",
    "hsl(var(--accent))",
    "hsl(var(--secondary))",
    "hsl(var(--muted))",
];

pub fn get_workspace_color(index: usize) -> &'static str {
    COLORS[index % COLORS.len()]
}
```

### Animation Configuration

```rust
pub struct AnimationConfig {
    pub spring_stiffness: f64,
    pub spring_damping: f64,
    pub duration_ms: u32,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            spring_stiffness: 300.0,
            spring_damping: 30.0,
            duration_ms: 200,
        }
    }
}
```

---

## macOS Dock System

### Dock Structure

```rust
pub struct DockState {
    pub position: DockPosition,
    pub icon_magnification: f64,
    pub icon_distance: f64,
}

pub struct DockPosition {
    pub left_offset: f64,  // Dynamic based on sidebar width
    pub top_offset: f64,
}

impl DockState {
    pub fn calculate_position(&self, sidebar_width: f64, container_width: f64) -> DockPosition {
        DockPosition {
            left_offset: sidebar_width,
            top_offset: DOCK_TOP_OFFSET,
        }
    }
}
```

### Dock Icons

The dock displays icons for:
1. **Screen icons**: Terminal, Code, Globe (Browser), Welcome
2. **Separator**: Visual divider
3. **Action icons**: Plus (add screen), LayoutGrid (grid view)

```rust
pub fn get_icon_for_screen(screen: &Screen) -> IconType {
    match screen.screen_type {
        ScreenType::Terminal => IconType::Terminal,
        ScreenType::Code => IconType::Code,
        ScreenType::Browser => IconType::Globe,
        ScreenType::Welcome => IconType::Sparkles,
        ScreenType::Custom => {
            // Use custom icon from screen.dock_icon
            screen.dock_icon
                .as_ref()
                .and_then(|name| IconType::from_name(name))
                .unwrap_or(IconType::Box)
        }
    }
}
```

### Dock Interaction

```rust
impl DockState {
    pub fn handle_icon_click(&mut self, screen_id: &str, app_state: &mut AppState) {
        app_state.active_screen_id = screen_id.to_string();
        // Reset resize tracking when switching via dock
        app_state.carousel_state.resize_direction = None;
        app_state.carousel_state.resize_sides = ResizeSides::default();
    }
    
    pub fn handle_add_screen(&mut self, app_state: &mut AppState) {
        let number = app_state.screens.len() + 1;
        let new_screen = Screen {
            id: format!("screen-{}", timestamp()),
            screen_type: ScreenType::Custom,
            title: format!("Screen {}", number),
            width: 0.0,
            height: 0.0,
            dock_icon: Some(get_random_icon_name()),
        };
        app_state.screens.push(new_screen.clone());
        app_state.active_screen_id = new_screen.id;
    }
}
```

---

## Event Handling

### Carousel Drag Events

```rust
pub struct DragInfo {
    pub offset_x: f64,
    pub offset_y: f64,
    pub velocity_x: f64,
    pub velocity_y: f64,
}

impl ScreenCarouselState {
    pub fn on_drag_start(&mut self) {
        if !self.is_resizing {
            self.is_dragging = true;
        }
    }
    
    pub fn on_drag_end(&mut self, info: DragInfo, screens: &[Screen]) -> Option<String> {
        if self.is_resizing {
            return None;
        }
        
        self.is_dragging = false;
        
        let active_index = self.get_active_screen_index();
        let mut new_index = active_index;
        
        // Velocity-based navigation
        if info.velocity_x.abs() > 500.0 {
            new_index = if info.velocity_x > 0.0 {
                active_index.saturating_sub(1)
            } else {
                (active_index + 1).min(screens.len() - 1)
            };
        }
        // Offset-based navigation
        else if info.offset_x.abs() > self.container_size.0 / 4.0 {
            new_index = if info.offset_x > 0.0 {
                active_index.saturating_sub(1)
            } else {
                (active_index + 1).min(screens.len() - 1)
            };
        }
        
        if new_index != active_index {
            // Reset resize tracking when changing screens
            self.resize_direction = None;
            self.resize_sides = ResizeSides::default();
            Some(screens[new_index].id.clone())
        } else {
            None
        }
    }
}
```

### Keyboard Shortcuts

```rust
pub enum KeyCommand {
    NewTab,
    CloseTab,
    NextScreen,
    PrevScreen,
    ToggleSidebar,
    OpenCommand,
    CreateWorkspace,
}

pub fn handle_keyboard_event(key: &str, modifiers: &Modifiers) -> Option<KeyCommand> {
    match (key, modifiers.ctrl, modifiers.shift, modifiers.alt) {
        ("t", true, false, false) => Some(KeyCommand::NewTab),
        ("w", true, false, false) => Some(KeyCommand::CloseTab),
        ("Tab", true, false, false) => Some(KeyCommand::NextScreen),
        ("Tab", true, true, false) => Some(KeyCommand::PrevScreen),
        ("b", true, false, false) => Some(KeyCommand::ToggleSidebar),
        ("k", true, false, false) => Some(KeyCommand::OpenCommand),
        ("n", true, true, false) => Some(KeyCommand::CreateWorkspace),
        _ => None,
    }
}
```

---

## Rendering Logic

### Screen Wrapper Calculation

The wrapper width is critical for the live animation effect:

```rust
fn calculate_wrapper_width(&self, screen: &Screen, is_active: bool) -> f64 {
    if is_active {
        // Active screen: use current dimensions (live during resize)
        if self.is_resizing {
            self.resizing_dimensions
                .map(|(w, _)| w)
                .unwrap_or(screen.width)
        } else {
            screen.width
        }
    } else {
        // Non-active screens: always use full container width
        self.container_size.0
    }
}
```

### Render Order

1. **Wrapped last screen** (if first screen is active and resizing left)
2. **All normal screens** (in order)
3. **Wrapped first screen** (if last screen is active and resizing right)

```rust
pub struct RenderElement {
    pub screen: Screen,
    pub wrapper_width: f64,
    pub position: f64,
    pub is_active: bool,
    pub is_wrapped: bool,
    pub margin_right: f64,
}

impl ScreenCarouselState {
    pub fn get_render_elements(&self) -> Vec<RenderElement> {
        let mut elements = Vec::new();
        let active_index = self.get_active_screen_index();
        
        // Wrapped last screen
        if self.should_show_wrapped_last_screen(active_index) {
            let last_screen = &self.screens[self.screens.len() - 1];
            elements.push(RenderElement {
                screen: last_screen.clone(),
                wrapper_width: self.container_size.0,
                position: -(self.container_size.0 + GAP),
                is_active: false,
                is_wrapped: true,
                margin_right: GAP,
            });
        }
        
        // Normal screens
        for (index, screen) in self.screens.iter().enumerate() {
            let is_active = index == active_index;
            let wrapper_width = self.calculate_wrapper_width(screen, is_active);
            
            elements.push(RenderElement {
                screen: screen.clone(),
                wrapper_width,
                position: self.calculate_screen_position(index),
                is_active,
                is_wrapped: false,
                margin_right: GAP,
            });
        }
        
        // Wrapped first screen
        if self.should_show_wrapped_first_screen(active_index) {
            let first_screen = &self.screens[0];
            let last_pos = self.calculate_screen_position(self.screens.len() - 1);
            
            elements.push(RenderElement {
                screen: first_screen.clone(),
                wrapper_width: self.container_size.0,
                position: last_pos + self.container_size.0 + GAP,
                is_active: false,
                is_wrapped: true,
                margin_right: GAP,
            });
        }
        
        elements
    }
}
```

---

## Critical Implementation Notes

### 1. Resize Side Tracking

**CRITICAL**: The `resize_sides` struct must persist throughout a single resize operation:

```rust
// ❌ WRONG - Resets on every resize event
pub fn on_resize(&mut self, direction: &str) {
    self.resize_sides = ResizeSides::default(); // DON'T DO THIS
    // ... rest of logic
}

// ✅ CORRECT - Only reset on resize start
pub fn on_resize_start(&mut self) {
    // Reset happens here, at the START of a new resize operation
    self.resize_sides = ResizeSides::default();
    self.is_resizing = true;
}

pub fn on_resize(&mut self, direction: &str) {
    // Accumulate side information
    if direction.contains("left") {
        self.resize_sides.left = true;
    }
    if direction.contains("right") {
        self.resize_sides.right = true;
    }
    // ... rest of logic
}
```

### 2. Live Animation Requirements

For smooth live animation during resize:

1. **Use a ref for dimensions**: Store `resizing_dimensions` in a ref/cell that doesn't trigger full re-renders
2. **Update wrapper width immediately**: Calculate wrapper width from `resizing_dimensions` during resize
3. **Force targeted re-renders**: Increment `resize_counter` to trigger only necessary updates
4. **Update X position directly**: Set `x_position` without animation during resize

### 3. Screen Independence

Each screen must maintain independent state:

```rust
// ❌ WRONG - Global resize state affects all screens
pub struct ScreenCarouselState {
    pub screen_width: f64,  // Shared by all screens
}

// ✅ CORRECT - Per-screen state
pub struct Screen {
    pub width: f64,   // Each screen has its own width
    pub height: f64,  // Each screen has its own height
}

pub struct ScreenCarouselState {
    pub manually_resized_screens: HashSet<String>,  // Track which screens were resized
}
```

### 4. Sidebar Toggle Behavior

When sidebar toggles, screen behavior depends on resize history:

```rust
pub fn on_sidebar_toggle(&mut self, new_sidebar_width: f64) {
    let old_container_width = self.container_size.0;
    let width_delta = new_sidebar_width - old_container_width;
    
    // Update container size
    self.container_size.0 += width_delta;
    
    // Update each screen based on whether it was manually resized
    for screen in &mut self.screens {
        if self.manually_resized_screens.contains(&screen.id) {
            // Manually resized: keep current size, ensure within bounds
            screen.width = screen.width.min(self.container_size.0);
            screen.height = screen.height.min(self.container_size.1);
        } else {
            // Not manually resized: reset to full width
            screen.width = self.container_size.0;
            screen.height = self.container_size.1;
        }
    }
}
```

### 5. Drag Constraints

Carousel drag must be constrained to prevent over-scrolling:

```rust
pub fn get_drag_constraints(&self) -> (f64, f64) {
    let left_constraint = -((self.screens.len() - 1) as f64 * (self.container_size.0 + GAP));
    let right_constraint = self.container_size.0;
    
    (left_constraint, right_constraint)
}
```

### 6. Gap Consistency

All screens must have consistent gaps:

- Normal screens: `margin-right: GAP`
- Wrapped screens: positioned with GAP included in calculation
- First screen: no special treatment (gap on right)
- Last screen: no special treatment (gap on right)

### 7. Animation Timing

```rust
pub struct SpringAnimation {
    pub stiffness: f64,
    pub damping: f64,
}

impl Default for SpringAnimation {
    fn default() -> Self {
        Self {
            stiffness: 300.0,
            damping: 30.0,
        }
    }
}

// Apply spring animation to carousel position
pub fn animate_to_position(&mut self, target: f64) {
    // Use spring physics for smooth, natural movement
    // Implementation depends on your animation library
}
```

---

## Complete Resize Flow Example

Here's a complete walkthrough of a resize operation:

### Scenario: User resizes first screen from the left side

**Step 1: Resize Start**
```rust
on_resize_start("screen-1", 1920.0, 1080.0)
├─ is_resizing = true
├─ manually_resized_screens.insert("screen-1")
├─ resizing_dimensions = Some((1920.0, 1080.0))
└─ resize_sides = { left: false, right: false }  // Reset
```

**Step 2: First Resize Event (dragging left edge)**
```rust
on_resize("screen-1", "left", -100.0, 0.0, 1920.0, 1080.0)
├─ new_width = 1820.0
├─ new_height = 1080.0
├─ direction.contains("left") = true
│  └─ resize_sides.left = true
├─ resize_sides = { left: true, right: false }
├─ resize_direction = Some(Left)
├─ resizing_dimensions = Some((1820.0, 1080.0))
├─ resize_counter += 1
└─ update_position_during_resize(1820.0)
    ├─ left_edge = 0.0
    ├─ available_space = 1920.0 - 1820.0 = 100.0
    ├─ Both sides? No (only left)
    ├─ Left only? Yes
    └─ x_position = -0.0 + 100.0 = 100.0  // Push right
```

**Step 3: Continue Resizing (more left drag)**
```rust
on_resize("screen-1", "left", -200.0, 0.0, 1820.0, 1080.0)
├─ new_width = 1620.0
├─ resize_sides.left = true (already true)
├─ resize_sides = { left: true, right: false }  // Unchanged
├─ resize_direction = Some(Left)  // Still left
├─ resizing_dimensions = Some((1620.0, 1080.0))
└─ x_position = 300.0  // More space, push further right
```

**Step 4: Now Resize from Right Side**
```rust
on_resize("screen-1", "right", -150.0, 0.0, 1620.0, 1080.0)
├─ new_width = 1470.0
├─ direction.contains("right") = true
│  └─ resize_sides.right = true
├─ resize_sides = { left: true, right: true }  // BOTH NOW!
├─ resize_direction = Some(Both)  // Changed to Both
├─ resizing_dimensions = Some((1470.0, 1080.0))
└─ x_position = 225.0  // Center: available_space / 2
```

**Step 5: Resize Stop**
```rust
on_resize_stop("screen-1", 1470.0, 1080.0)
├─ screens[0].width = 1470.0
├─ screens[0].height = 1080.0
├─ resizing_dimensions = None
├─ is_resizing = false
└─ resize_counter = 0
```

**Result**: 
- Screen is 1470px wide (was 1920px)
- Screen is centered (both sides visible)
- Last screen visible on left (circular wrapping)
- Second screen visible on right
- Both adjacent screens are clickable

---

## UI Framework Considerations

### For Rust GUI Frameworks

When implementing in Rust, consider these frameworks:

**1. egui** (Immediate Mode)
- Simple to use, good for prototyping
- Built-in drag-and-drop support
- May need custom animation handling
- Good performance for this use case

**2. Iced** (Elm Architecture)
- Declarative, type-safe
- Built-in animation support
- Message-based state updates
- Excellent for complex state management

**3. Tauri + Web Frontend**
- Use existing TypeScript code with minimal changes
- Rust backend for performance-critical operations
- Best compatibility with current implementation
- Easiest migration path

**4. Dioxus** (React-like)
- Similar to React, familiar patterns
- Good animation support via CSS
- Virtual DOM for efficient updates
- Native and web targets

### Recommended Approach

For this specific application, **Tauri + Web Frontend** is recommended because:

1. **Minimal Rewrite**: Keep existing TypeScript/React code
2. **Rust Backend**: Use Rust for file operations, system integration
3. **Animation Libraries**: Leverage Framer Motion (already used)
4. **Drag-and-Drop**: Keep dnd-kit (already implemented)
5. **Gradual Migration**: Move components to Rust as needed

### Alternative: Pure Rust with Iced

If you want pure Rust:

```rust
// Example Iced message types
#[derive(Debug, Clone)]
pub enum Message {
    // Screen carousel
    ScreenResizeStart(String),
    ScreenResize(String, String, f64, f64),
    ScreenResizeStop(String, f64, f64),
    ScreenChange(String),
    CarouselDragStart,
    CarouselDragEnd(f64, f64),
    
    // Sidebar
    ToggleSidebar,
    SetActiveWorkspace(String),
    
    // Tabs
    AddTab,
    CloseTab(String),
    
    // Drag and drop
    DragStart(String, DraggedItem),
    DragOver(String, f64, f64),
    DragEnd,
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resize_direction_left_only() {
        let mut state = ScreenCarouselState::default();
        state.on_resize_start("screen-1", 1920.0, 1080.0);
        state.on_resize("screen-1", "left", -100.0, 0.0, 1920.0, 1080.0);
        
        assert_eq!(state.resize_direction, Some(ResizeDirection::Left));
        assert!(state.resize_sides.left);
        assert!(!state.resize_sides.right);
    }
    
    #[test]
    fn test_resize_direction_both_sides() {
        let mut state = ScreenCarouselState::default();
        state.on_resize_start("screen-1", 1920.0, 1080.0);
        state.on_resize("screen-1", "left", -100.0, 0.0, 1920.0, 1080.0);
        state.on_resize("screen-1", "right", -50.0, 0.0, 1820.0, 1080.0);
        
        assert_eq!(state.resize_direction, Some(ResizeDirection::Both));
        assert!(state.resize_sides.left);
        assert!(state.resize_sides.right);
    }
    
    #[test]
    fn test_circular_wrapping_first_screen() {
        let mut state = ScreenCarouselState::default();
        state.screens = vec![
            Screen::new("1".into(), ScreenType::Welcome, "Welcome".into()),
            Screen::new("2".into(), ScreenType::Terminal, "Terminal".into()),
        ];
        state.active_screen_id = "1".to_string();
        state.resize_direction = Some(ResizeDirection::Left);
        
        assert!(state.should_show_wrapped_last_screen(0));
    }
    
    #[test]
    fn test_per_screen_resize_independence() {
        let mut state = ScreenCarouselState::default();
        state.screens = vec![
            Screen::new("1".into(), ScreenType::Welcome, "Welcome".into()),
            Screen::new("2".into(), ScreenType::Terminal, "Terminal".into()),
        ];
        
        // Resize first screen
        state.on_resize_start("1", 1920.0, 1080.0);
        state.on_resize("1", "left", -200.0, 0.0, 1920.0, 1080.0);
        state.on_resize_stop("1", 1720.0, 1080.0);
        
        // Switch to second screen
        state.active_screen_id = "2".to_string();
        
        // First screen should still be resized
        assert!(state.manually_resized_screens.contains("1"));
        assert_eq!(state.screens[0].width, 1720.0);
        
        // Second screen should be full width
        assert_eq!(state.screens[1].width, 1920.0);
    }
}
```

---

## Performance Optimization

### Hot Path Considerations

**1. Resize Event Handling**
- Resize events fire rapidly (60+ times per second)
- Avoid allocations in `on_resize()`
- Use refs/cells for temporary state
- Batch state updates

```rust
// ❌ Avoid allocations in hot path
pub fn on_resize(&mut self, ...) {
    let new_vec = vec![...];  // Allocation on every event!
}

// ✅ Use pre-allocated buffers or refs
pub fn on_resize(&mut self, ...) {
    self.resizing_dimensions = Some((width, height));  // Just update existing
}
```

**2. Render Calculations**
- Cache screen positions when possible
- Only recalculate on container size change or screen reorder
- Use iterators instead of collecting

```rust
// ✅ Efficient iteration
pub fn visible_screens(&self) -> impl Iterator<Item = &Screen> {
    self.screens.iter().filter(|s| self.is_screen_visible(s))
}
```

**3. Animation Frame Updates**
- Limit re-renders to necessary components
- Use dirty flags for selective updates
- Debounce non-critical updates

### Memory Management

```rust
// Use Rc for shared immutable data
use std::rc::Rc;

pub struct SharedConfig {
    pub colors: Rc<Vec<String>>,
    pub animation_config: Rc<AnimationConfig>,
}

// Use RefCell for interior mutability when needed
use std::cell::RefCell;

pub struct CarouselCache {
    positions: RefCell<HashMap<String, f64>>,
}
```

---

## Common Pitfalls and Solutions

### Pitfall 1: Resetting Resize Sides Too Early

**Problem**: Resetting `resize_sides` on every resize event loses tracking information.

**Solution**: Only reset on `resize_start`, accumulate during `resize`.

### Pitfall 2: Not Using Live Dimensions

**Problem**: Wrapper width doesn't update during resize, causing jumpy animation.

**Solution**: Use `resizing_dimensions` ref and force re-renders with counter.

### Pitfall 3: Applying Global Resize State

**Problem**: Resizing one screen affects all screens.

**Solution**: Use per-screen tracking with `manually_resized_screens` HashSet.

### Pitfall 4: Forgetting Circular Wrapping Gaps

**Problem**: Wrapped screens don't have consistent gaps.

**Solution**: Include GAP in position calculations for wrapped screens.

### Pitfall 5: Not Disabling Drag During Resize

**Problem**: User can drag carousel while resizing, causing conflicts.

**Solution**: Check `is_resizing` flag before allowing drag operations.

### Pitfall 6: Incorrect Gravity Calculation

**Problem**: Screen doesn't reveal adjacent screens correctly.

**Solution**: Calculate `available_space` and apply correct offset based on direction.

---

## Migration Path from TypeScript

### Phase 1: Setup Tauri Project

```bash
# Install Tauri CLI
cargo install tauri-cli

# Create new Tauri project
cargo tauri init

# Keep existing frontend code
# Add Rust backend for system operations
```

### Phase 2: Identify Backend Operations

Move these to Rust:
- File system operations
- System tray integration
- Native window management
- Performance-critical calculations

Keep in TypeScript:
- UI rendering (React components)
- Animations (Framer Motion)
- Drag-and-drop (dnd-kit)
- State management (React hooks)

### Phase 3: Create Rust Commands

```rust
// src-tauri/src/main.rs

#[tauri::command]
fn get_screen_state() -> Result<ScreenCarouselState, String> {
    // Return current state
}

#[tauri::command]
fn update_screen_size(id: String, width: f64, height: f64) -> Result<(), String> {
    // Update screen dimensions
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_screen_state,
            update_screen_size
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Phase 4: Call from TypeScript

```typescript
import { invoke } from '@tauri-apps/api/tauri';

async function updateScreenSize(id: string, width: number, height: number) {
  await invoke('update_screen_size', { id, width, height });
}
```

---

## Summary Checklist

When implementing this system in Rust, ensure:

### Screen Carousel
- [ ] Container size tracking with ResizeObserver pattern
- [ ] Per-screen width/height storage
- [ ] 8px gap between all screens (including wrapped)
- [ ] Active screen index calculation
- [ ] Screen position calculation: `index * (container_width + GAP)`

### Directional Gravity System
- [ ] `resize_sides` struct with `left` and `right` booleans
- [ ] Reset `resize_sides` only on `resize_start`
- [ ] Accumulate side information during `resize`
- [ ] Calculate direction: Left, Right, or Both
- [ ] Apply gravity based on direction
- [ ] Update wrapper width live during resize
- [ ] Use `resizing_dimensions` ref for live updates

### Circular Wrapping
- [ ] Show last screen before first when resizing left
- [ ] Show first screen after last when resizing right
- [ ] Only show wrapped screens when appropriate direction
- [ ] Include GAP in wrapped screen positioning
- [ ] Make wrapped screens clickable

### Per-Screen Independence
- [ ] `manually_resized_screens` HashSet
- [ ] Each screen has own width/height
- [ ] Sidebar toggle respects manual resize state
- [ ] Screen change resets resize direction
- [ ] No global resize state affecting all screens

### Sidebar System
- [ ] Expanded width: 360px
- [ ] Collapsed width: 56px
- [ ] Logo container (max 12 items)
- [ ] Workspace switcher (max 5 visible)
- [ ] Folder/tab hierarchy
- [ ] Drag-and-drop support

### macOS Dock
- [ ] Top-center positioning
- [ ] Dynamic left offset based on sidebar width
- [ ] Screen icons + separator + action icons
- [ ] Icon magnification on hover
- [ ] Click to switch screens

### Drag-and-Drop
- [ ] Tab, Folder, Logo drag types
- [ ] Drop position calculation (before/after/inside)
- [ ] Logo container as drop target
- [ ] Visual feedback during drag
- [ ] Proper state updates on drop

### Performance
- [ ] No allocations in resize hot path
- [ ] Use refs for temporary state
- [ ] Cache calculations when possible
- [ ] Efficient iteration over collections

---

## Conclusion

This guide provides a complete specification for implementing the browser screen system in Rust. The most complex feature is the directional gravity system with its side tracking and live animation requirements.

Key takeaways:
1. **Track both sides independently** throughout a resize operation
2. **Use refs for live updates** to avoid state update delays
3. **Maintain per-screen state** for true independence
4. **Apply consistent gaps** including for wrapped screens
5. **Disable drag during resize** to prevent conflicts

The system is designed to feel natural and fluid, with screens revealing adjacent content as they're resized, creating an intuitive multi-screen workspace experience.
