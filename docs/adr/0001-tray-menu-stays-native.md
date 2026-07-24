# Tray menu is a custom HTML popup window (not native, not acrylic)

Status: accepted.

The tray menu is a custom borderless, always-on-top webview window
(`src-tauri/src/tray_menu.rs` + `src/lib/trayMenuView.svelte`) opened at the
cursor on left/right-click. It is NOT a native OS menu and NOT an Acrylic/Mica
popup. (An earlier version of this ADR recorded "keep the native menu"; that was
reversed after the native menu's limits made the prototype unreachable.)

Why not native (muda/Win32): can't display SVG directly (needs rasterization),
can't control font/icon size or item spacing, can't color a single item (e.g. a
red 退出), and can't render a logo header — every fidelity ask hit a wall.

Why not Acrylic/Mica: a frosted popup needs a transparent window, and on Win10
transparent + always-on-top is the combination that froze WebView2 before (see
`CLAUDE.md`, "Things that bit us" #6). Transparent regions also composite with a
frosted DWM edge on Win10, so the menu is an **opaque solid card** (square
corners; Win11 would auto-round opaque windows).

Consequences: the menu is plain HTML/CSS (inline SVG icons, red 退出, live
active-count), fully styleable via WebView2 — at the cost of hand-rolling
focus/click-outside/Escape dismissal and cursor positioning (in `tray_menu.rs`).
Window creation from a sync command deadlocks `WebviewWindowBuilder::build()`, so
`tray_menu_action` is `async` (same lesson as `reactivate`).
