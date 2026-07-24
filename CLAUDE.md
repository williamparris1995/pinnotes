# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

PinNotes is an always-on-top sticky-note reminder app. Each note is its own borderless, always-on-top window that lives in the system tray. **"Hide" only snoozes** — a hidden note pops back in place (without stealing focus) at its due time; only "✓ 完成/Complete" removes it. Built with **Tauri 2 (Rust) + Svelte 5 + Vite**, persisted in bundled SQLite. The Rust backend is the single source of truth; the Svelte frontend is a thin view that calls it via `invoke`/`listen`.

## Commands

Run from repo root unless noted.

| Task | Command |
|------|---------|
| Dev (Vite on :1420 + Rust app) | `npm run tauri dev` |
| Build installers (current platform) | `npm run tauri build` |
| Frontend type-check (svelte-check) | `npm run check` |
| Frontend tests (Vitest) | `npm test` |
| Single frontend test file | `npx vitest run src/lib/noteView.test.ts` |
| Rust tests | `cd src-tauri && cargo test` |
| Single Rust test (by name) | `cd src-tauri && cargo test <name_substring>` |
| Rust fast check | `cd src-tauri && cargo check` |

- Vite uses a **fixed port 1420** (`strictPort`); a stale dev server there blocks `tauri dev` — kill orphans before relaunching.
- Vitest is pinned to **`pool: "vmThreads"`** in [vite.config.ts](vite.config.ts). The default `forks` pool fails to collect suites on this Win10 / Node 22 setup. Don't switch it back.
- Release: push a `v*` tag → [.github/workflows/release.yml](.github/workflows/release.yml) builds Windows/macOS/Linux and publishes installers to a GitHub Release. macOS/Linux can't be cross-built from Windows; they're built in CI.

## Architecture

### One webview window per note
`index.html` is a **hash-router SPA** ([src/App.svelte](src/App.svelte)): `#/note?id=<id>` renders a note; `#/completed` and `#/settings` render the auxiliary views. The window declared in [tauri.conf.json](src-tauri/tauri.conf.json) is `visible: false` (a hidden host). The Rust backend creates windows dynamically:
- **Note windows** ([window_manager.rs](src-tauri/src/window_manager.rs)): label `note-<id>`, `decorations:false`, `always_on_top:true`, `skip_taskbar:true`, `resizable:false`, URL `index.html#/note?id=<id>`.
- **Aux windows** (completed/settings): single-instance, opened by `lib::open_simple` at `index.html#/<route>`.

### Backend modules (`src-tauri/src/`)
- **lib.rs** — Tauri builder wiring: opens SQLite at `<app_data_dir>/pinnotes.sqlite`, builds `AppState`, tray icon + global Ctrl+N shortcut, sets default autostart on first run, runs **startup load** (opens active note windows; for hidden notes either re-arms a snooze timer or repops immediately if it expired while away), registers the `invoke_handler` command list.
- **commands.rs** — the command surface. Each action is a plain `*_impl` function with a thin `#[tauri::command]` wrapper. **Always put new logic in `*_impl`** so the tray menu and shortcuts can reuse it without command dispatch.
- **db.rs** — `type Db = Mutex<Connection>`; schema = `notes` + `settings(key,val)`; `Note` struct + `NoteRepository`. Note: `is_hidden` is `i64` in SQLite but `bool` on the struct (converted in `row_to_note`).
- **snooze.rs** — `SnoozeScheduler`: in-process tokio timers keyed by note id; rescheduling cancels the prior timer. `should_repop(completed, hidden_until, now)` is the due predicate.
- **window_manager.rs** — open/show/hide/close/move/resize note windows + the drag-debounce hook (below).
- **geometry.rs** — multi-monitor clamp; `clamp_into_work_area` picks the monitor with the most overlap and keeps an 8px margin.
- **tray.rs** (icon + click handling: left/right-click → HTML menu) / **tray_menu.rs** (custom HTML popup window: cursor positioning, focus/click-outside/Escape dismissal — see ADR-0001) / **autostart.rs** (`tauri-plugin-autostart`) / **state.rs** (`AppState`) / **main.rs** (thin binary entry → `pinnotes_lib::run()`).

### Things that bit us — read before changing these
1. **Hide is a snooze, not a delete.** `hide_note` sets `is_hidden`+`hidden_until` and schedules a tokio timer; `repop_note` clears the snooze and re-shows **without focus**. Only `complete_note` closes the note for good. Startup load re-arms or repops hidden notes accordingly.
2. **Show/hide go through raw Win32 `ShowWindow`, not Tauri's `show()`/`hide()`.** Tauri tracks an internal visibility flag that raw `SW_SHOWNOACTIVATE` (the no-focus repop) doesn't update, so mixing them leaves the flag stale and the next Tauri show/hide becomes a no-op ("can't hide after repop"). That's what the `windows` crate dependency is for. Non-Windows falls back to Tauri's API.
3. **`reactivate` must stay `async`.** `WebviewWindowBuilder::build()` needs the main thread's message loop; a sync command runs on the main thread inside the IPC handler and **deadlocks** `build()`. The `async` form runs on the runtime, which marshals onto a free main thread. (Comment at [commands.rs:172](src-tauri/src/commands.rs#L172).)
4. **Drag position writes are debounced.** `WindowEvent::Moved` fires per pixel; `AppState.drag_writes` holds one pending `JoinHandle` per note that's cancelled + rescheduled ~250ms later on a background task — no per-pixel SQLite on the main thread.
5. **Coordinates are logical px.** `Moved` carries a `PhysicalPosition`; divide by `scale_factor` to match stored logical x/y. `move_note` clamps the full rect into the best-overlap monitor.
6. **No Acrylic/window effect on note windows.** Acrylic + transparent + always_on_top froze the WebView2 render process on Windows 10. Notes are plain translucent cards (rgba tint over a transparent window). Documented inline in [window_manager.rs](src-tauri/src/window_manager.rs).

### Persistence & first run
SQLite at `<app_data_dir>/pinnotes.sqlite`; two tables, `notes` and `settings` (generic key/val). First run creates a welcome note (`commands::maybe_welcome_note`, guarded by the `first_run_done` setting) and enables autostart (guarded by `autostart_configured`); later runs respect the user's settings.

## Agent skills

### Issue tracker

Issues and PRDs live as GitHub issues (uses the `gh` CLI). See `docs/agents/issue-tracker.md`.

### Triage labels

Five canonical roles, label string equal to its name (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context — one `CONTEXT.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.
