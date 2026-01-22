# OpenKit — Cross-Platform CSS-Styled UI Framework

> A Rust framework for building beautiful desktop applications with CSS-powered styling, featuring a Tailwind-inspired design system that looks consistent across Windows, Linux, and macOS.

---

## MVP (v0.1.0) — Minimal Viable Window with CSS Styling

The absolute minimum to open a styled window with basic widgets on all three platforms.

### Platform Abstraction Layer
- [ ] Define `Platform` trait for OS-specific implementations
- [ ] Windows backend: Win32 for window, custom rendering for widgets
- [ ] macOS backend: AppKit for window, custom rendering for widgets
- [x] Linux backend: winit (direct X11/Wayland) for window, custom rendering for widgets (NO GTK!)
- [ ] Runtime platform detection and backend selection

### Rendering Engine
- [ ] GPU-accelerated 2D renderer using `wgpu`
- [ ] Software fallback renderer (CPU-based)
- [ ] Text rendering with `cosmic-text` or `fontdue`
- [ ] Font loading (system fonts + bundled)
- [ ] Anti-aliased shape primitives (rect, rounded rect, circle, line)
- [ ] Image rendering (PNG, JPEG, WebP)

### CSS Engine (Core)
- [ ] CSS parser (subset of CSS3)
- [ ] Selector matching (class, id, type, pseudo-classes)
- [ ] Cascade and specificity calculation
- [ ] CSS custom properties (`--var-name`)
- [ ] `calc()` expressions
- [ ] Shorthand property expansion
- [ ] Inherited vs non-inherited properties

### Tailwind-Inspired Default Theme
- [ ] Design token system (colors, spacing, typography, shadows, radii)
- [ ] Color palette: slate, gray, zinc, neutral, stone, red, orange, amber, yellow, lime, green, emerald, teal, cyan, sky, blue, indigo, violet, purple, fuchsia, pink, rose
- [ ] Spacing scale: 0, 0.5, 1, 1.5, 2, 2.5, 3, 3.5, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16, 20, 24, 28, 32, 36, 40, 44, 48, 52, 56, 60, 64, 72, 80, 96
- [ ] Font sizes: xs, sm, base, lg, xl, 2xl, 3xl, 4xl, 5xl, 6xl, 7xl, 8xl, 9xl
- [ ] Font weights: thin, extralight, light, normal, medium, semibold, bold, extrabold, black
- [ ] Border radius: none, sm, default, md, lg, xl, 2xl, 3xl, full
- [ ] Box shadows: sm, default, md, lg, xl, 2xl, inner, none
- [ ] Default font stack: Inter, system-ui fallbacks

### Theme System
- [ ] Light theme (default)
- [ ] Dark theme
- [ ] Auto-detect OS theme preference
- [ ] Runtime theme switching
- [ ] Theme as CSS variables (easy customization)
- [ ] Semantic color tokens: `--background`, `--foreground`, `--primary`, `--secondary`, `--muted`, `--accent`, `--destructive`, `--border`, `--ring`

### Window Management
- [ ] Create native window with title
- [ ] Set window size (width × height)
- [ ] Set window position (x, y)
- [ ] Show/hide window
- [ ] Close window programmatically
- [ ] Window state: minimized, maximized, fullscreen, restored
- [ ] Window background color from theme

### Event Loop
- [ ] Platform-native event loop integration
- [ ] Window events: open, close, resize, move, focus, blur
- [ ] Mouse events: click, double-click, move, scroll, enter, leave
- [ ] Keyboard events: key down, key up, key press (with modifiers)
- [ ] Event dispatch system (sync)

### Basic Widgets (Styled)
- [ ] `Label` — styled text with font-size, color, weight
- [ ] `Button` — hover, active, focus, disabled states
- [ ] `TextField` — single-line input with focus ring
- [ ] `Checkbox` — custom styled checkbox with animations
- [ ] All widgets respond to CSS classes

### Layout (Minimal)
- [ ] Absolute positioning
- [ ] Basic container (vertical stack)
- [ ] Padding and margin from CSS

### Build & CI
- [ ] GitHub Actions: build on Windows, macOS, Linux
- [ ] Cargo feature flags for optional platform backends
- [ ] Bundle Inter font (or similar) as default
- [ ] Example: styled "Hello, World" window with dark/light toggle

---

## v0.2.0 — Core Widget Set & Layout

Expand the widget library with full CSS styling support.

### Additional Widgets
- [ ] `TextArea` — multi-line text input
- [ ] `RadioButton` — single-select from group
- [ ] `Select` / `Dropdown` — styled dropdown menu
- [ ] `Slider` — range input with track and thumb styling
- [ ] `ProgressBar` — determinate/indeterminate with animations
- [ ] `Image` — display raster images with object-fit
- [ ] `Separator` / `Divider` — horizontal/vertical line
- [ ] `Badge` — small status indicator
- [ ] `Avatar` — circular image/initials
- [ ] `Card` — container with shadow and rounded corners
- [ ] `Tooltip` — hover hints with arrow

### CSS Layout Engine
- [ ] Flexbox layout (display: flex)
  - [ ] flex-direction: row, column, row-reverse, column-reverse
  - [ ] justify-content: start, end, center, space-between, space-around, space-evenly
  - [ ] align-items: start, end, center, stretch, baseline
  - [ ] flex-wrap: nowrap, wrap, wrap-reverse
  - [ ] flex-grow, flex-shrink, flex-basis
  - [ ] gap (row-gap, column-gap)
- [ ] CSS Grid (display: grid)
  - [ ] grid-template-columns, grid-template-rows
  - [ ] grid-column, grid-row (span)
  - [ ] grid-gap
  - [ ] auto-fit, auto-fill with minmax()
- [ ] Box model: width, height, min/max variants, padding, margin, border
- [ ] Overflow: visible, hidden, scroll, auto

### CSS Properties (Extended)
- [ ] Background: color, gradient, image, size, position, repeat
- [ ] Border: width, style, color, radius (per-corner)
- [ ] Box shadow (multiple shadows)
- [ ] Opacity
- [ ] Cursor types
- [ ] Outline (for focus states)
- [ ] Text: color, align, decoration, transform, overflow, white-space

### Pseudo-Classes
- [ ] `:hover`
- [ ] `:active`
- [ ] `:focus`
- [ ] `:focus-visible`
- [ ] `:disabled`
- [ ] `:checked`
- [ ] `:first-child`, `:last-child`, `:nth-child()`

---

## v0.3.0 — Containers & Navigation

### Container Widgets
- [ ] `ScrollArea` — styled scrollbars (custom or native)
- [ ] `SplitPane` — resizable panes with styled handle
- [ ] `Tabs` — tabbed content with animated indicator
- [ ] `Accordion` — collapsible sections with animations
- [ ] `Dialog` / `Modal` — overlay with backdrop blur
- [ ] `Sheet` — slide-in panel (bottom, side)
- [ ] `Popover` — floating content anchored to trigger

### Window Features
- [ ] Multiple windows per application
- [ ] Modal dialogs (styled, not native)
- [ ] Native file dialogs (OS-provided)
- [ ] Toast / Notification system
- [ ] Window icons
- [ ] Frameless/borderless windows with custom title bar
- [ ] Window transparency / blur effects (where supported)

### Menu System
- [ ] Styled menu bar
- [ ] Styled context menus
- [ ] Menu items with icons
- [ ] Keyboard shortcuts display
- [ ] Nested submenus
- [ ] System tray icon with styled menu

---

## v0.4.0 — Data Display & Lists

### List Widgets
- [ ] `List` — vertical list with styled items
- [ ] `Table` — data table with headers, sorting, resizable columns
- [ ] `Tree` — hierarchical tree with expand/collapse
- [ ] Virtual scrolling for 100k+ items
- [ ] Selection styles: single, multiple, range
- [ ] Row hover and stripe styles
- [ ] Empty state styling

### Data Binding (Reactive)
- [ ] Observable state primitive (`Signal<T>`)
- [ ] Automatic UI updates on state change
- [ ] Computed/derived values
- [ ] Two-way binding for inputs
- [ ] Conditional rendering
- [ ] List rendering with keys

---

## v0.5.0 — Advanced CSS Features

### Transitions & Animations
- [ ] CSS `transition` property
  - [ ] transition-property
  - [ ] transition-duration
  - [ ] transition-timing-function (ease, linear, ease-in, ease-out, ease-in-out, cubic-bezier)
  - [ ] transition-delay
- [ ] CSS `@keyframes` animations
  - [ ] animation-name
  - [ ] animation-duration
  - [ ] animation-timing-function
  - [ ] animation-delay
  - [ ] animation-iteration-count
  - [ ] animation-direction
  - [ ] animation-fill-mode
- [ ] Transform: translate, scale, rotate, skew
- [ ] Transform-origin

### Advanced Selectors
- [ ] Attribute selectors: `[attr]`, `[attr=value]`, `[attr^=]`, `[attr$=]`, `[attr*=]`
- [ ] Combinators: descendant (` `), child (`>`), sibling (`+`, `~`)
- [ ] `:not()`, `:is()`, `:where()`
- [ ] `::before`, `::after` pseudo-elements
- [ ] `::placeholder`, `::selection`

### Media Queries
- [ ] `prefers-color-scheme: dark | light`
- [ ] `prefers-reduced-motion: reduce`
- [ ] `prefers-contrast: high`
- [ ] Width/height breakpoints (for responsive layouts)

### Rich Text
- [ ] Styled text spans (inline CSS)
- [ ] Inline links with hover states
- [ ] Text selection styling
- [ ] Markdown rendering widget

---

## v0.6.0 — Advanced Input & Graphics

### Drag & Drop
- [ ] Internal drag & drop with styled ghost
- [ ] External drag & drop (files from OS)
- [ ] Styled drop zones
- [ ] Drag handle styling

### Canvas / Custom Paint
- [ ] `Canvas` widget for custom 2D drawing
- [ ] Drawing primitives: line, rect, ellipse, arc, path
- [ ] Fill and stroke styles
- [ ] Gradients (linear, radial, conic)
- [ ] Shadows and blur
- [ ] Clipping paths
- [ ] Blend modes

### Advanced Inputs
- [ ] `ColorPicker` — color selection with swatches
- [ ] `DatePicker` — calendar date selection
- [ ] `TimePicker` — time selection
- [ ] `NumberInput` — with increment/decrement
- [ ] `SearchInput` — with clear button and loading state
- [ ] `TagInput` — multi-value input with chips

---

## v0.7.0 — Accessibility (A11y)

### Screen Reader Support
- [ ] ARIA-like roles and properties
- [ ] Accessible names and descriptions
- [ ] Live regions for dynamic content
- [ ] Windows: UI Automation provider
- [ ] macOS: NSAccessibility protocol
- [ ] Linux: ATK/AT-SPI2

### Keyboard Accessibility
- [ ] Full keyboard operability
- [ ] Focus management (tab order, focus trap)
- [ ] Skip links / landmark navigation
- [ ] `:focus-visible` for keyboard-only focus

### Visual Accessibility
- [ ] High contrast theme
- [ ] `prefers-reduced-motion` respected
- [ ] Minimum touch target sizes
- [ ] Color contrast validation (dev mode)
- [ ] Scalable UI (respect OS text scaling)

---

## v0.8.0 — Internationalization (i18n)

### Text & Locale
- [ ] Full Unicode / emoji support
- [ ] Bidirectional text (RTL languages)
- [ ] `direction: ltr | rtl` CSS property
- [ ] Locale-aware formatting
- [ ] Font fallback chains for CJK, Arabic, etc.

### Input Methods
- [ ] IME support (CJK input)
- [ ] Compose key sequences
- [ ] Dead keys for diacritics

---

## v0.9.0 — Developer Experience

### Style DevTools
- [ ] Widget inspector (like browser DevTools)
- [ ] Computed styles panel
- [ ] Box model visualization
- [ ] CSS rule inspector with specificity
- [ ] Live CSS editing

### Debugging & Profiling
- [ ] Event monitor / logger
- [ ] Layout performance profiler
- [ ] Render frame timing
- [ ] Memory usage tracking

### Hot Reload
- [ ] Hot reload CSS changes
- [ ] Hot reload theme changes
- [ ] State preservation on reload

### Documentation
- [ ] Complete API documentation
- [ ] CSS property reference
- [ ] Widget gallery / Storybook-style showcase
- [ ] Theme customization guide
- [ ] Migration guide from web CSS

### Testing
- [ ] Headless rendering mode
- [ ] Visual regression testing
- [ ] Accessibility testing utilities
- [ ] Style assertion helpers

---

## v1.0.0 — Stable Release

### Stability Guarantees
- [ ] API freeze (semver compatibility)
- [ ] CSS compatibility spec documented
- [ ] All public APIs documented
- [ ] No known critical bugs
- [ ] Performance benchmarks published

### Platform Parity
- [ ] Identical styling across Windows, macOS, Linux
- [ ] Consistent behavior documented
- [ ] Platform-specific escape hatches documented

### Production Readiness
- [ ] Used in at least one production application
- [ ] Memory leak testing
- [ ] Long-running stability testing
- [ ] Security audit

### Ecosystem
- [ ] Published to crates.io
- [ ] Logo and branding
- [ ] Website with live playground
- [ ] Theme gallery
- [ ] Community themes repository

---

## Future / Post-1.0

- [ ] Mobile support (iOS, Android)
- [ ] Web target (WebAssembly)
- [ ] Tailwind-style utility class syntax (compile-time)
- [ ] CSS-in-Rust macro DSL
- [ ] Figma/Sketch plugin for export
- [ ] Visual theme editor
- [ ] Component marketplace
- [ ] Server-side rendering for CLI tools

---

## Architecture Principles

1. **Consistent Styling**: Your app looks identical on Windows, macOS, and Linux. No platform-specific visual quirks.

2. **CSS-Powered**: Style everything with familiar CSS. If you know web CSS, you know OpenKit styling.

3. **Tailwind by Default**: Ship with a beautiful, modern design system inspired by Tailwind CSS. Dark and light themes included.

4. **GPU-First Rendering**: All widgets are custom-rendered with GPU acceleration for smooth 60fps animations.

5. **Accessible by Default**: Every widget includes proper accessibility semantics. Works with screen readers out of the box.

6. **Zero JavaScript Runtime**: Pure Rust. No Electron. No WebView. Native performance with web-like styling.

7. **Themeable**: Override any design token. Create your own theme. Ship multiple themes.

---

## Default Theme: Design Tokens

### Color Palette (Tailwind-compatible)

```css
/* Light Theme */
:root {
  --background: 0 0% 100%;          /* white */
  --foreground: 222.2 84% 4.9%;     /* slate-950 */
  --card: 0 0% 100%;
  --card-foreground: 222.2 84% 4.9%;
  --popover: 0 0% 100%;
  --popover-foreground: 222.2 84% 4.9%;
  --primary: 221.2 83.2% 53.3%;     /* blue-500 */
  --primary-foreground: 210 40% 98%;
  --secondary: 210 40% 96%;
  --secondary-foreground: 222.2 47.4% 11.2%;
  --muted: 210 40% 96%;
  --muted-foreground: 215.4 16.3% 46.9%;
  --accent: 210 40% 96%;
  --accent-foreground: 222.2 47.4% 11.2%;
  --destructive: 0 84.2% 60.2%;     /* red-500 */
  --destructive-foreground: 210 40% 98%;
  --border: 214.3 31.8% 91.4%;
  --input: 214.3 31.8% 91.4%;
  --ring: 221.2 83.2% 53.3%;
  --radius: 0.5rem;
}

/* Dark Theme */
:root.dark {
  --background: 222.2 84% 4.9%;     /* slate-950 */
  --foreground: 210 40% 98%;
  --card: 222.2 84% 4.9%;
  --card-foreground: 210 40% 98%;
  --popover: 222.2 84% 4.9%;
  --popover-foreground: 210 40% 98%;
  --primary: 217.2 91.2% 59.8%;     /* blue-400 */
  --primary-foreground: 222.2 47.4% 11.2%;
  --secondary: 217.2 32.6% 17.5%;
  --secondary-foreground: 210 40% 98%;
  --muted: 217.2 32.6% 17.5%;
  --muted-foreground: 215 20.2% 65.1%;
  --accent: 217.2 32.6% 17.5%;
  --accent-foreground: 210 40% 98%;
  --destructive: 0 62.8% 30.6%;
  --destructive-foreground: 210 40% 98%;
  --border: 217.2 32.6% 17.5%;
  --input: 217.2 32.6% 17.5%;
  --ring: 224.3 76.3% 48%;
}
```

### Typography

```css
:root {
  --font-sans: 'Inter', ui-sans-serif, system-ui, -apple-system, sans-serif;
  --font-mono: 'JetBrains Mono', ui-monospace, 'Cascadia Code', monospace;

  --text-xs: 0.75rem;     /* 12px */
  --text-sm: 0.875rem;    /* 14px */
  --text-base: 1rem;      /* 16px */
  --text-lg: 1.125rem;    /* 18px */
  --text-xl: 1.25rem;     /* 20px */
  --text-2xl: 1.5rem;     /* 24px */
  --text-3xl: 1.875rem;   /* 30px */
  --text-4xl: 2.25rem;    /* 36px */
}
```

### Spacing Scale

```css
:root {
  --space-0: 0;
  --space-1: 0.25rem;   /* 4px */
  --space-2: 0.5rem;    /* 8px */
  --space-3: 0.75rem;   /* 12px */
  --space-4: 1rem;      /* 16px */
  --space-5: 1.25rem;   /* 20px */
  --space-6: 1.5rem;    /* 24px */
  --space-8: 2rem;      /* 32px */
  --space-10: 2.5rem;   /* 40px */
  --space-12: 3rem;     /* 48px */
  --space-16: 4rem;     /* 64px */
}
```

---

## Rendering Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Application Code                        │
├─────────────────────────────────────────────────────────────┤
│                     Widget Tree (Rust)                       │
├─────────────────────────────────────────────────────────────┤
│                      CSS Style Engine                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │  Parser  │→ │ Cascade  │→ │ Computed │→ │  Layout  │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
├─────────────────────────────────────────────────────────────┤
│                    Flexbox/Grid Layout                       │
├─────────────────────────────────────────────────────────────┤
│                      Render Layer                            │
│  ┌──────────────────────┐  ┌──────────────────────────┐    │
│  │   GPU (wgpu/WebGPU)  │  │  Software (tiny-skia)    │    │
│  └──────────────────────┘  └──────────────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                   Platform Window                            │
│  ┌────────────┐  ┌────────────┐  ┌────────────────────┐    │
│  │   Win32    │  │   AppKit   │  │   X11/Wayland      │    │
│  └────────────┘  └────────────┘  └────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

---

## Example: Styled Button

```rust
use openkit::prelude::*;

fn main() {
    App::new()
        .theme(Theme::Dark) // or Theme::Light, Theme::Auto
        .run(|cx| {
            Button::new("Click Me")
                .class("btn-primary")
                .on_click(|_| println!("Clicked!"))
        });
}
```

```css
/* Built-in, but customizable */
.btn-primary {
  background: var(--primary);
  color: var(--primary-foreground);
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius);
  font-weight: 500;
  transition: background 150ms ease;
}

.btn-primary:hover {
  background: color-mix(in oklch, var(--primary), black 10%);
}

.btn-primary:active {
  background: color-mix(in oklch, var(--primary), black 20%);
}

.btn-primary:focus-visible {
  outline: 2px solid var(--ring);
  outline-offset: 2px;
}
```

---

## License Considerations

- Framework: MIT or Apache-2.0 (dual license)
- Default theme: MIT (derived from shadcn/ui concepts)
- Bundled fonts: Check individual font licenses (Inter: OFL, JetBrains Mono: OFL)
- Examples: CC0 / Public Domain
- Documentation: CC-BY-4.0
