---
name: Sharkord Plus
description: Fast, privacy-respecting, self-hosted Discord alternative with a Pure Neutral Dark aesthetic and native GPU acceleration
colors:
  bg-base: "#0a0b0e"
  bg-surface: "#111216"
  bg-subsurface: "#16171d"
  bg-elevated: "#1c1e24"
  border-hairline: "rgba(255, 255, 255, 0.08)"
  border-subtle: "rgba(255, 255, 255, 0.12)"
  text-primary: "#f3f4f6"
  text-secondary: "#9ca3af"
  text-muted: "#6b7280"
  accent: "#3b82f6"
  accent-hover: "#60a5fa"
  status-online: "#22c55e"
  status-idle: "#eab308"
  status-dnd: "#ef4444"
  status-offline: "#6b7280"
typography:
  display:
    fontFamily: "Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
    fontSize: "1.5rem"
    fontWeight: 600
    lineHeight: 1.2
  headline:
    fontFamily: "Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
    fontSize: "1.125rem"
    fontWeight: 600
    lineHeight: 1.3
  body:
    fontFamily: "Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
    fontSize: "0.875rem"
    fontWeight: 400
    lineHeight: 1.5
  code:
    fontFamily: "'JetBrains Mono', 'Fira Code', monospace"
    fontSize: "0.8125rem"
    fontWeight: 400
    lineHeight: 1.4
rounded:
  sm: "4px"
  md: "8px"
  lg: "12px"
  full: "9999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "24px"
components:
  channel-item:
    backgroundColor: "transparent"
    textColor: "{colors.text-secondary}"
    rounded: "{rounded.md}"
    padding: "6px 8px"
  channel-item-active:
    backgroundColor: "{colors.bg-elevated}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.md}"
    padding: "6px 8px"
  message-bubble:
    backgroundColor: "transparent"
    textColor: "{colors.text-primary}"
    padding: "4px 16px"
---

## Overview

Sharkord Plus uses a **Pure Neutral Dark** design system inspired by tools like Linear and Raycast. It abandons saturated neon purples in favor of deep, refined neutral grays, disciplined spatial hierarchy, and clear typography.

Every surface is engineered for high information density, low eye strain during prolonged night sessions, and immediate muscle-memory familiarity for Discord power users.

## Colors

The palette is strictly neutral, using tonal depth rather than color washes to distinguish layers:

- **Base Canvas (`#0a0b0e`):** The foundational dark background behind the entire viewport.
- **Surface Level 1 (`#111216`):** Left server rail and main message canvas.
- **Surface Level 2 (`#16171d`):** Channel sidebar, direct messages list, and member panel.
- **Surface Level 3 (`#1c1e24`):** Active item selection, chat input box, and hovering states.
- **Surface Level 4 (`#242630`):** Modals, popovers, contextual menus, and floating tooltips.
- **Hairline Borders (`rgba(255, 255, 255, 0.08)`):** 1px sharp structural boundaries separating panels without visual weight.
- **Functional Accents:**
  - Status Indicators: Online (`#22c55e`), Idle (`#eab308`), DND (`#ef4444`), Streaming/Voice (`#8b5cf6`).
  - Active Accent: Precision cold blue (`#3b82f6`) for selected states, badges, and primary action buttons.

## Typography

The typography uses clean grotesque sans-serifs (`Inter` or system native) paired with `JetBrains Mono` for code blocks and IDs.

- **Display & Section Headers:** Inter SemiBold (16px–24px), tight tracking (`-0.01em`).
- **Body & Messages:** Inter Regular (14px), 1.5 line height for high reading endurance.
- **Metadata & Timestamps:** Inter Medium (11px–12px), muted text color (`#6b7280`).
- **Code & Snippets:** JetBrains Mono (13px), with high contrast syntax coloring.

## Layout

### Desktop Layout (Three-Column Spatial Architecture)
1. **Server Rail (72px fixed):** Vertical pill list of server icons, home DM shortcut, and quick settings.
2. **Channel & Navigation Panel (240px fixed):** Server banner, categories, voice/text channels, and docked voice controls at bottom.
3. **Primary Canvas (Flex 1):** Header bar with channel topic and search, scrollable message stream, and docked bottom input bar.
4. **Member List Panel (240px collapsible):** Grouped by role hierarchy, toggleable via TopBar button.

### Mobile & Touch Layout (Adaptive Sliding Drawer Hierarchy)
- **Viewport < 768px:**
  - Primary chat canvas takes 100% viewport width.
  - A permanent, accessible Top Navigation Bar displays:
    - Hamburger button (`Menu` icon) to open the Left Drawer (Server rail + Channel list).
    - Current Channel title and topic indicator.
    - Member list button (`Users` icon) to open the Right Drawer.
  - Edge-swipe gestures open and close drawers smoothly.
  - Touch targets maintain a minimum 44px hit-box.
  - Voice active indicator stays docked at the bottom with touch-friendly Mute/Disconnect buttons.

## Elevation & Depth

No heavy blurred drop shadows. Elevation is achieved through:
- **Tonal Contrast:** Rising layers become progressively lighter (`#0a0b0e` -> `#111216` -> `#16171d` -> `#1c1e24`).
- **Hairline Division:** 1px `border-white/[0.08]` gives precise component boundaries.
- **Floating Modals & Sheets:** Use a subtle backdrop blur (`backdrop-blur-md`) and a dark overlay (`bg-black/60`).

## Shapes

- **Corner Radii:**
  - Micro elements (badges, buttons, tags): `rounded-md` (6px–8px).
  - Cards, panels, popovers: `rounded-lg` (8px–12px).
  - Avatars & Status dots: `rounded-full`.
  - Server Rail Icons: Rounded-2xl morphing to rounded-xl on hover/active.

## Components

### Message Feed
- Grouped by sender within 5-minute windows to eliminate repetitive headers.
- Hover reveals floating contextual action bar (Reactions, Reply, Edit, More).
- Media embeds render with rounded corners and border containment.

### Voice Grid & User Cards
- Speaking state: Precision green ring (`ring-2 ring-emerald-500/80`) around user card.
- Quality indicator badge (Hardware acceleration, codec indicator, ping).

### Channel Sidebar
- Muted icons (`#6b7280`) illuminate on hover (`#f3f4f6`).
- Unread indicator: Discrete white pill on the left margin.

## Do's and Don'ts

### Do
- Keep contrast sharp and text easily readable in low-light environments.
- Maintain seamless backwards compatibility with vanilla Sharkord server protocols.
- Preserve Discord-familiar hotkeys (`Ctrl+K`, `Escape` to close, `Arrow` keys).
- Test touch hit-areas on mobile devices to prevent mis-clicks.

### Don't
- Don't use garish saturated background colors or bright neon washes.
- Don't hide channel navigation on mobile without a visible hamburger button.
- Don't block UI threads with synchronous operations (keep all RPC and voice signaling async).
