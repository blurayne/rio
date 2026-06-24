---
date: 2026-06-24T11:57:19+00:00
git_commit: 256251e508314298334677b4cd613708928f8a3c
branch: main
topic: "Native OS context menu on mouse right-click"
tags: [plan, rioterm, rio-window, sugarloaf, menu, wayland, x11, macos, windows]
status: in_progress
scope_note: |
  This session implements Linux X11 only. Wayland keeps the existing
  Sugarloaf in-canvas popover (zero regression). macOS + Windows muda
  paths are scaffolded as todo!() stubs and tracked for a follow-up
  implementation plan. See `Scope This Session` below.
---

# Native OS Context Menu on Mouse Right-Click — Implementation Plan

## Scope This Session

Mid-implementation scope reduction (recorded 2026-06-24): the original
plan attempted all platforms in one phase. Investigation revealed that
proper Wayland `xdg_popup` support requires a fork-level refactor of
rio-window's wayland window/state machinery (sctk's `SctkWindow`
toplevel-only abstraction is deeply embedded in `WindowState`,
`WinitState`, event dispatch).

**Mid-implementation course correction**: initial popup_menu_window
code passed `with_parent_window(parent)` to make the popup transient
to the terminal. On X11 this actually creates an X-server *child*
window of the parent, which gets *clipped* to the parent's bounds —
defeating the entire purpose of overflow. Fixed by dropping
`with_parent_window` and relying on absolute screen coordinates +
`override_redirect` + `_NET_WM_WINDOW_TYPE_POPUP_MENU` hint, which
makes the popup a top-level root-child override-redirect window that
can overflow freely.

To ship a working PR this session:

| Platform | Status | Path |
|----------|--------|------|
| Linux X11 | Implemented | borderless override-redirect popup window + Sugarloaf, uses existing `WindowAttributesExtX11::with_override_redirect` + `with_x11_window_type(PopupMenu)` |
| Linux Wayland | Unchanged | falls back to existing Sugarloaf in-canvas popover (zero regression) |
| macOS | Stubbed | `todo!()` in cfg branch; follow-up plan at `docs/agents/plans/2026-06-24-native-context-menu-macos-windows.md` |
| Windows | Stubbed | same as macOS |

Wayland xdg_popup support is its own multi-day project; a follow-up
plan will cover it.

## Overview

Right-clicking with the mouse opens the OS-native context menu instead of the
in-canvas Sugarloaf popover. Native menus can extend beyond the parent
window's bounds, match the OS theme, and provide proper keyboard navigation.
The existing Sugarloaf popover remains in place for keyboard-triggered menu
invocation (e.g., Shift+F10 or a future `OpenPaneMenu` action).

## Current State Analysis

- The pane context menu is currently drawn entirely inside the wgpu canvas
  via `frontends/rioterm/src/renderer/pane_titlebar_menu.rs` using Sugarloaf
  rect / text primitives. Menu is clipped to window bounds and uses
  hard-coded RGBA constants — no OS theming.
- Right-click anywhere in a pane routes to `Screen::open_pane_titlebar_menu`
  at `frontends/rioterm/src/application.rs:1315-1342`.
- 13 menu entries are defined by `PaneTitlebarMenu::entries()` at
  `frontends/rioterm/src/renderer/pane_titlebar_menu.rs:138-160`, mapped to
  a `MenuAction` enum (`pane_titlebar_menu.rs:46-65`). Actions are dispatched
  by `Screen::handle_titlebar_menu_click` at
  `frontends/rioterm/src/screen/mod.rs:2789-2844`.
- `rio-window` already exposes `HasWindowHandle` / `HasDisplayHandle`
  (`rio-window/src/window.rs:1721,1733`) and `raw-window-handle = 0.6.2`
  (`Cargo.toml:39`). `rioterm` already extracts handles at
  `frontends/rioterm/src/router/mod.rs:709-710`.
- On **macOS** and **Windows**, `rio-window` already pulls in `objc2`,
  `objc2-app-kit` (with `NSMenu` + `NSMenuItem` features), and
  `windows-sys` (`Win32_UI_WindowsAndMessaging` includes `TrackPopupMenu`).
  Native menu calls are reachable from `rioterm` once it has the raw
  handle.
- On **Wayland**, `rio-window` has **no `xdg_popup` / `xdg_positioner` /
  `parent_window` / `with_position` support** (verified by grep — only
  references to "popup" in the Wayland tree are in unrelated comments).
  `sctk = 0.19.2` and `wayland-protocols = 0.32.8` are already deps, so
  the missing piece is wiring, not a new dependency.
- On **X11**, `rio-window` already supports
  `_NET_WM_WINDOW_TYPE_POPUP_MENU`
  (`rio-window/src/platform_impl/linux/x11/util/hint.rs:36`), parent
  windows (`rio-window/src/platform_impl/linux/x11/window.rs:172`), and
  absolute positioning via `with_position`. No new X11 code needed beyond
  exposing a popup-window convenience builder.

## Desired End State

When a user clicks the right mouse button inside any pane:

- **macOS**: a real `NSMenu` appears at the cursor, rendered by AppKit;
  selection sends a `MenuEvent` that is forwarded into rio's event loop
  via `EventLoopProxy` and dispatched through the existing `MenuAction`
  handler. The menu can extend past the window edge. Escape / click-away
  closes it natively.
- **Windows**: a real `HMENU` shown via `TrackPopupMenuEx`, behaving
  identically. Theme follows the OS (light / dark).
- **Linux X11**: an undecorated, transient, override-redirect popup
  window owned by `rioterm` is created at the cursor position, hosting
  a Sugarloaf instance that draws the existing `PaneTitlebarMenu`
  widget. Click-outside / Escape / focus-loss closes it.
- **Linux Wayland**: an `xdg_popup` (anchored to the parent window's
  `wl_surface`) hosts the same Sugarloaf-rendered menu. The compositor
  positions and dismisses it; `xdg_popup::popup_done` triggers cleanup.

The existing Sugarloaf in-canvas popover is **kept and untouched** for
non-mouse triggers (currently none exposed; this leaves room for a
future `OpenPaneMenu` keyboard action — Shift+F10 / Menu key).

## What We're NOT Doing

- Not building submenus natively. The `Assistants ▶` / `Profiles ▶` /
  `Other ▶` items remain stubs (same as today). Native submenu support
  can be a follow-up — muda supports it but the current entries are
  no-ops.
- Not adding the GTK dep on Linux. muda's Linux backend requires
  `gtk3` + `libxdo` and a `gtk::Window`, neither of which fit
  `rio-window`'s non-GTK Wayland/X11 implementation. Linux gets its
  "native feel" via a borderless rio-window-owned popup, not via muda.
- Not adding new menu entries. The 13 entries from
  `PaneTitlebarMenu::entries()` are the source of truth.
- Not rewriting the existing Sugarloaf popover. It stays for the
  keyboard path. No `cfg`-gated removal.
- Not changing the right-click trigger location. The same condition
  (any pane area, any pixel) opens the menu, just via a different
  backend.
- Not changing macOS application menubar code (`with_default_menu` /
  `MenuBar`). This plan touches **context menus only**.

## UI Mockups

### Today (Sugarloaf in-canvas, clipped)

```
┌─────────────── Terminal Window ──────────────┐
│ $ ls                                          │
│ file1.txt  file2.txt                          │
│ $ ▏                                           │
│         ┌────────────────────┐                │
│         │ Split Vertically   │ ← clipped to   │
│         │ Split Horizontally │   wgpu canvas, │
│         │ ─────────────────  │   custom paint │
│         │ Find…              │                │
│         │ Read-Only ☐        │                │
│         │ ─────────────────  │                │
│         │ Assistants ▶       │                │
│         │ Profiles ▶         │                │
│         │ ─────────────────  │                │
│         │ Other ▶            │                │
│         │ ─────────────────  │                │
│         │ Detach             │                │
│         │ ─────────────────  │                │
│         │ Close              │                │
│         └────────────────────┘                │
└───────────────────────────────────────────────┘
```

### After — macOS / Windows (true native NSMenu / HMENU, can overflow)

```
┌─────────────── Terminal Window ──────────────┐
│ $ ls                                          │
│ file1.txt  file2.txt                          │
│ $ ▏                                           │
│                                  ┌────────────┼─────────┐
│                                  │ Split Vert │ically   │ ← OS draws,
│                                  │ Split Hori │zontally │   overflows
│                                  │ ─────────  │ ─       │   window
│                                  │ Find…      │         │   bounds,
│                                  │ Read-Only ☐│         │   themed
│                                  │ ...        │         │
│                                  └────────────┼─────────┘
└───────────────────────────────────────────────┘
```

### After — Linux (rio-window borderless popup hosting Sugarloaf)

```
┌─────────────── Terminal Window ──────────────┐
│ $ ls                                          │
│ file1.txt  file2.txt                          │
│ $ ▏                                           │
│                                  ╔════════════╪═════════╗
│                                  ║ Split Vert │ically   ║ ← separate
│                                  ║ Split Hori │zontally ║   borderless
│                                  ║ ─────────  │ ─       ║   window,
│                                  ║ Find…      │         ║   own Sugarloaf
│                                  ║ ...        │         ║   instance,
│                                  ╚════════════╪═════════╝   can overflow
└───────────────────────────────────────────────┘
```

## Architecture and Code Reuse

### Backend selection (compile-time)

```
                ┌─────────────────────────────────────────┐
                │ MouseButton::Right pressed inside pane  │
                │ (application.rs MouseInput handler)     │
                └────────────────────┬────────────────────┘
                                     │
                  ┌──────────────────┴──────────────────┐
                  │   route::open_native_context_menu   │
                  └─────┬──────────┬──────────┬─────────┘
                        │          │          │
              cfg(macos)│  cfg(win)│ cfg(unix,│
                        ▼          ▼   !macos) ▼
                ┌────────────┐ ┌─────────┐ ┌────────────────┐
                │   muda::   │ │ muda::  │ │ PopupMenuWindow│
                │ NSMenu via │ │ HMENU   │ │ (rio-window +  │
                │ nsview     │ │ via hwnd│ │ Sugarloaf)     │
                └─────┬──────┘ └────┬────┘ └────────┬───────┘
                      │             │               │
                      └─────────────┴───────────────┘
                                    │
                                    ▼
                  ┌──────────────────────────────────────┐
                  │ EventLoopProxy.send_event(           │
                  │   RioEvent::NativeContextMenuAction) │
                  └──────────────────┬───────────────────┘
                                     ▼
                  ┌──────────────────────────────────────┐
                  │ Screen::handle_titlebar_menu_click   │
                  │ (existing dispatcher reused as-is)   │
                  └──────────────────────────────────────┘
```

### Code reuse

- `MenuAction` enum (`pane_titlebar_menu.rs:46-65`) is the action vocabulary
  — used by all three backends.
- `PaneTitlebarMenu::entries()` (`pane_titlebar_menu.rs:138-160`) is the
  source of truth for menu structure (labels + actions + separators +
  read-only checkbox state) — both the native build path and the Linux
  Sugarloaf popup re-use it.
- `Screen::handle_titlebar_menu_click` (`screen/mod.rs:2789-2844`) is the
  single dispatcher — both the Sugarloaf path and the new native paths
  funnel into it.
- `Sugarloaf` is reused on Linux for popup rendering — instantiated per
  popup window.
- Existing `RawWindowHandle` extraction in `router/mod.rs:709` provides
  the `ns_view` / `hwnd` pointers muda needs.

### Affected file tree

- `Cargo.toml` — workspace dep entry for muda (target-gated)
- `rio-backend/src/event/mod.rs`
  - `RioEvent::NativeContextMenuAction(u32)` — new user-event variant
    carrying an opaque action id. The id → `MenuAction` mapping lives in
    `Router` (in `rioterm`); `rio-backend` stays UI-agnostic.
- `frontends/rioterm`
  - `Cargo.toml` — target-gated `muda = "0.19"` (macOS + Windows only)
  - `src/screen/mod.rs`
    - **Refactor**: extract the action-dispatch body of
      `Screen::handle_titlebar_menu_click` into a new public method
      `Screen::dispatch_menu_action(action: MenuAction, clipboard:
      &mut Clipboard)`. Keep `handle_titlebar_menu_click` as a thin
      wrapper that hit-tests the Sugarloaf menu and forwards to
      `dispatch_menu_action` — preserves the keyboard path with no
      behaviour change.
  - `src/router/mod.rs`
    - `Router.popup_menus: FxHashMap<WindowId, PopupMenuWindow>` — Linux
      only, popup-window registry
    - `Router.pending_menu_origin: Option<(WindowId, PaneNodeId)>` — the
      pane that was right-clicked. Set when the menu opens, cleared
      after dispatch (or after muda's synchronous menu loop returns on
      macOS/Windows).
    - `Router.menu_action_registry: HashMap<u32, MenuAction>` — populated
      when building each menu (cleared first). Looked up in
      `user_event` when `RioEvent::NativeContextMenuAction(id)` arrives.
    - `Router::open_native_context_menu(origin: WindowId, pane_id,
       cursor: PhysicalPosition)` — dispatches per-cfg
  - `src/router/native_context_menu.rs` — **new module**
    - `pub fn show_native_menu(...)` — cfg-gated bodies
    - macOS body: build `muda::Menu`, call
      `show_context_menu_for_nsview(ns_view, Some(Physical(x, y)))`
    - Windows body: build `muda::Menu`, call
      `show_context_menu_for_hwnd(hwnd, Some(Physical(x, y)))`
    - Linux body: call `Router::spawn_popup_menu_window(...)`
    - `fn build_muda_menu(entries: &[MenuEntry], registry: &mut
       HashMap<MenuId, MenuAction>) -> muda::Menu` — shared by macOS +
      Windows builders
  - `src/router/popup_menu_window.rs` — **new module, Linux only**
    - `PopupMenuWindow { winit_window, sugarloaf, menu, origin }` —
      owns a single popup
    - `render()`, `hit_test_and_dispatch(...)`, `close()`
  - `src/application.rs`
    - `MouseInput::Right` branch (line 1315-1342) gated by cfg:
      - Mouse path: call `open_native_context_menu`
      - Keyboard path: unchanged (currently no caller, future-proofed)
    - New `UserEvent` handler branch for
      `RioEvent::NativeContextMenuAction`
    - Linux popup-window event dispatch: pop-out events targeting a
      `WindowId` in `Router.popup_menus`
- `rio-window`
  - `src/window.rs`
    - `WindowAttributes.popup_anchor: Option<PopupAnchor>` — new field
    - `WindowAttributesExt::with_popup_anchor(parent_handle, position)` —
      builder
    - `pub struct PopupAnchor { parent: WindowHandle, position: (i32, i32) }`
  - `src/platform_impl/linux/wayland/window/mod.rs`
    - In `Window::new`, when `popup_anchor.is_some()`: build
      `xdg_positioner` (size, anchor rect, gravity, constraints) and
      create the surface via `sctk`'s `Popup` role instead of
      `XdgToplevel`
    - Handle `xdg_popup::popup_done` → emit `WindowEvent::CloseRequested`
  - `src/platform_impl/linux/wayland/state.rs`
    - Track xdg_popup handles per WindowId
  - `src/platform_impl/linux/x11/window.rs`
    - When `popup_anchor.is_some()`: set `override_redirect = true`,
      `_NET_WM_WINDOW_TYPE_POPUP_MENU`, transient_for(parent), absolute
      position
  - `src/platform_impl/macos/window_delegate.rs`
    - When `popup_anchor.is_some()`: panel-style child window with
      `borderless | nonactivating` mask, transient_for parent — **only
      used as fallback if muda is unavailable for some reason. In normal
      operation macOS uses muda and never creates a popup window.**
  - `src/platform_impl/windows/window.rs`
    - Same shape as macOS: child popup window fallback. Normal path is
      muda.
- `Dockerfile` — no change (no new system deps)
- `pkgRio.nix` / `flake.nix` — no change
- `FORK.md` — add a "Native context menu (Phase 19)" section
- `docs/features/native-context-menu.md` — **new** user-facing doc

### Third-party API reference

- `muda` 0.19 trait `ContextMenu`:
  - `show_context_menu_for_hwnd(&self, hwnd: isize, position:
    Option<Position>)`
  - `show_context_menu_for_nsview(&self, view: *mut c_void, position:
    Option<Position>)`
  - Synchronous on both — the call blocks until the menu is dismissed
    or an item is selected.
- `muda::MenuEvent`:
  - `MenuEvent::set_event_handler(Some(handler))` — set ONCE at app
    init; handler closure runs on the thread that posted the event.
    Forward into `EventLoopProxy::send_event` so dispatch lands on the
    main thread.
- `sctk` (smithay-client-toolkit 0.19.2):
  - `xdg::popup::Popup` — popup surface role, anchored to parent
    `XdgSurface` via `xdg_positioner`. Sample usage in sctk's
    `examples/popup.rs`.
- `wayland-protocols` 0.32.8:
  - `xdg::shell::client::xdg_positioner` — already in the dep tree.

## Performance Considerations

- muda menus on macOS / Windows are zero-allocation per right-click
  beyond the `Menu` + `MenuItem` heap; building 13 items per click is
  trivial.
- Linux popup window: each right-click creates a new Wayland surface +
  Sugarloaf instance, draws a single frame, and tears down on dismiss.
  Cost is dominated by Sugarloaf init (one-time wgpu surface +
  shader pipeline). **Caching is an explicit non-goal of this plan** —
  if measured open latency exceeds ~50 ms in manual verification,
  a follow-up plan can add a `Router.popup_pool` that hides instead
  of destroys. The initial implementation creates a fresh popup per
  right-click for simplicity.
- `MenuEvent::set_event_handler` is set once at `Router::new`; no
  per-click handler registration.

## Migration Notes

- No persisted state changes. No new config keys.
- The keyboard-triggered Sugarloaf popover is currently unreachable
  (no keybinding maps to `OpenPaneMenu`). Future code may bind it.
  This plan leaves that path intact and untouched.
- Behaviour change on right-click is unconditional — there is no opt-in
  config flag. If users dislike native menus, that warrants a follow-up
  preference (`pane.native_context_menu: bool`, default `true`).

## Phase 1: Native Right-Click Context Menu

### A. Workspace dependency

**Tasks**:
- [x] Add `muda = { version = "0.19", default-features = false }` to
  `frontends/rioterm/Cargo.toml` under
  `[target.'cfg(any(target_os = "macos", target_os = "windows"))'.dependencies]`
  — Linux is intentionally excluded (no GTK).
- [x] Verify Cargo.lock updates only for macOS/Windows targets and that
  Linux `cargo build --no-default-features --features=wayland`
  succeeds with no muda transitive deps pulled in.

**Automated Verification**:
- [x] `cargo build --no-default-features --features=wayland -p rioterm`
  passes inside the Docker build container.
- [x] `cargo tree -p rioterm -e features` on Linux shows no `muda`.

### B. rio-window: PopupAnchor builder API (skipped — existing API sufficient)

**Outcome**: investigation showed `rio-window`'s existing builders
(`with_decorations(false)`, `with_resizable(false)`, `with_position`,
`with_parent_window`, `with_window_level(AlwaysOnTop)` plus the X11
extensions `with_override_redirect` + `with_x11_window_type(PopupMenu)`)
cover the X11 popup case without changes. Wayland `xdg_popup` is the
real missing piece and is deferred to a follow-up plan.

Original tasks (left here as documentation for the follow-up):

**Tasks**:
- [ ] In `rio-window/src/window.rs`, add
  `pub struct PopupAnchor { pub parent: RawWindowHandle, pub position:
  dpi::PhysicalPosition<i32> }` and field
  `WindowAttributes.popup_anchor: Option<PopupAnchor>` (default `None`).
- [ ] Add builder `WindowAttributes::with_popup_anchor(self, anchor:
  PopupAnchor) -> Self`.
- [ ] In `rio-window/src/platform_impl/linux/wayland/window/mod.rs`, in
  `Window::new`, when `attributes.popup_anchor.is_some()`:
   1. Build `xdg_positioner` via sctk: size = inner_size, anchor_rect =
      a 1×1 rect at `(position.x, position.y)` in the **parent
      surface's local coordinates** (this is where the popup "comes
      from"), anchor = `TopLeft`, gravity = `BottomRight`,
      constraint_adjustment = `SLIDE_X | SLIDE_Y | FLIP_Y`.
      **Rationale**: a context menu conceptually grows down-and-right
      from the cursor; `gravity = BottomRight` with the anchor at the
      cursor produces that. `SLIDE_X/Y` lets the compositor push the
      popup back inside the screen when it would overflow on the right
      or bottom edge; `FLIP_Y` lets it flip above the cursor when the
      menu would extend past the bottom of the screen — matching
      native menu behaviour (cf. xdg-shell spec
      `xdg_positioner::set_constraint_adjustment`).
   2. Create the surface with `sctk::shell::xdg::popup::Popup` role,
      parent from `popup_anchor.parent`.
   3. Wire `xdg_popup::popup_done` → emit
      `WindowEvent::CloseRequested` for this window.
- [ ] In `rio-window/src/platform_impl/linux/wayland/state.rs`, store
  popup handles in a per-WindowId map so `popup_done` can be routed.
- [ ] In `rio-window/src/platform_impl/linux/x11/window.rs`, when
  `popup_anchor.is_some()`: set `override_redirect = true`,
  `_NET_WM_WINDOW_TYPE_POPUP_MENU`, `XSetTransientForHint` to parent,
  call `XMoveWindow` at `position` after map.
- [ ] macOS and Windows code-paths: leave a `// popup_anchor handled via
  muda` comment, ignore the field. (Cross-platform builder still
  compiles.)

**Automated Verification**:
- [ ] Unit test in `rio-window/tests/popup_attributes.rs`:
  `WindowAttributes::default().with_popup_anchor(...)` round-trips.
- [ ] `cargo build -p rio-window --no-default-features --features wayland`
  passes.
- [ ] `cargo build -p rio-window --no-default-features --features x11`
  passes.
- [ ] `cargo build -p rio-window` (default features, both back-ends) passes.

**Manual Verification**:
- [ ] Build an `examples/popup_window.rs` sample in `rio-window` that
  spawns a parent + popup. Verify on Wayland (sway / GNOME / KDE)
  that the popup anchors at the requested position relative to the
  parent and dismisses on `popup_done`.
- [ ] Same example on X11 verifies the popup appears at the absolute
  screen position and is destroyed on focus-loss.

### C. rioterm: Native menu façade

**Tasks**:
- [x] Create `frontends/rioterm/src/router/native_context_menu.rs` with
  the cfg-stub façade + shared `build_registry()` helper that populates
  `Router.menu_action_registry`. (Linux dispatches via
  `Router::open_native_context_menu` directly; macOS / Windows muda
  bodies remain stubbed for the follow-up.)
- [x] Build entry list via `PaneTitlebarMenu::new()` + `entries()` —
  shared with the popup-window code (no duplication).
- [ ] cfg(macOS): build a `muda::Menu`. For each entry, clear and
  rebuild `Router.menu_action_registry`, generating ids as
  `format!("{i}")` (parsed back to `u32` in the muda handler).
  Append `PredefinedMenuItem::separator()` for separators and
  `MenuItem::with_id(id_str, label, enabled, None)` for actionable
  rows. Convert `cursor_logical` to physical and call
  `menu.show_context_menu_for_nsview(ns_view, Some(Position::Physical{x,y}))`.
- [ ] cfg(Windows): same as macOS but
  `show_context_menu_for_hwnd(hwnd as isize, ...)`.
- [ ] cfg(unix, not(macos)): call
  `Router::open_popup_menu_window(...)` (defined in next task group).
- [ ] In `Router::new`, set the global muda event handler **once**.
  The handler closure captures only an `EventLoopProxy` clone — no
  origin window-id, no registry. It parses the id back to `u32` and
  sends a `RioEvent::NativeContextMenuAction(u32)`:
  ```rust
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  {
      let proxy = event_proxy.clone();
      muda::MenuEvent::set_event_handler(Some(move |ev| {
          if let Ok(id) = ev.id.0.parse::<u32>() {
              // Origin window is filled in by application.rs from
              // Router.pending_menu_origin when the event arrives.
              let _ = proxy.send_event(EventPayload::new(
                  RioEventType::Rio(RioEvent::NativeContextMenuAction(id)),
                  rio_window::window::WindowId::dummy(),
              ));
          }
      }));
  }
  ```
  `Router` is single-threaded (accessed only from the main event loop
  thread); the registry is a plain `HashMap`, no `Mutex` required.

**Automated Verification**:
- [x] Unit test `registry_round_trip_covers_actionable_entries`
  passes (`frontends/rioterm/src/router/native_context_menu.rs`).
- [x] `cargo build -p rioterm --no-default-features --features=wayland`
  passes.
- [ ] `cargo build -p rioterm` on macOS / Windows — deferred to muda
  follow-up plan.

### D. rioterm: Route mouse right-click to native path

**Tasks**:
- [x] In `frontends/rioterm/src/application.rs` (right-click branch),
  call `self.router.open_native_context_menu(event_loop, window_id,
  (mx, my), &self.config)`. When it returns `false` (Wayland today,
  macOS/Windows until muda lands), fall back to the existing
  `Screen::open_pane_titlebar_menu(mx, my)` path.
- [x] `Router::open_native_context_menu` reads `route.window.screen`
  for read-only state, extracts both raw window + raw display handles
  from `route.window.winit_window`, computes absolute screen position
  as `parent.outer_position() + cursor_logical * scale`, then on Linux
  calls `PopupMenuWindow::open(...)`.
- [x] Existing island right-click handler still runs first
  (`application.rs` `handled_by_island` early-return).
- [ ] On macOS/Windows — deferred to muda follow-up.
- [x] On Linux (X11), `open_native_context_menu` returns immediately
  after spawning the popup; the popup's own window-event loop drives
  dispatch and dismiss. On Wayland the method returns `false` and
  the caller falls back to the in-canvas popover.

**Automated Verification**:
- [x] `cargo test -p rioterm --no-default-features --features=wayland`:
  206 passed (incl. updated `hit_test_first_item_returns_split_right`
  to match the post-256251e508 entry order).
- [x] `cargo clippy -p rioterm --no-default-features --features=wayland
  -- -D warnings` clean.

**Manual Verification**:
- [ ] On macOS: right-click anywhere in a pane → an NSMenu appears at
  the cursor, with system theme. Selecting "Split Vertically" splits
  the pane. Selecting "Close" closes it. Clicking outside dismisses
  with no action.
- [ ] On Windows 11: same as macOS — HMENU appears, theme follows
  system light/dark, Split / Close / Detach all dispatch correctly.
- [ ] Right-clicking near the edge of the window produces a menu that
  **extends past the window** (confirms native overflow behaviour).

### E. rioterm: Linux popup-menu window

**Tasks**:
- [x] Create `frontends/rioterm/src/router/popup_menu_window.rs`:
  ```rust
  pub struct PopupMenuWindow {
      pub window: Arc<rio_window::Window>,
      pub sugarloaf: Sugarloaf,
      pub menu: PaneTitlebarMenu,
      pub origin: (WindowId, PaneNodeId),
  }
  impl PopupMenuWindow {
      pub fn open(...) -> Self { /* create window + sugarloaf + open menu at (0,0) */ }
      pub fn render(&mut self);
      pub fn handle_mouse_input(&mut self, pos: (f32, f32),
          state: ElementState, button: MouseButton, proxy: &EventLoopProxy<_>);
      pub fn handle_focus(&mut self, focused: bool) -> bool; // returns true if should close
  }
  ```
- [x] In `Router`, add `popup_menus: FxHashMap<WindowId,
  PopupMenuWindow>` and a `Router::open_native_context_menu(...)` method
  that internally calls `PopupMenuWindow::open(...)`
  that:
   1. Computes popup size from `PaneTitlebarMenu::total_height()` +
      MENU_WIDTH.
   2. Creates a new `rio_window::Window` via
      `WindowAttributes::default().with_decorations(false)
       .with_resizable(false).with_inner_size(size)
       .with_window_level(WindowLevel::AlwaysOnTop)
       .with_popup_anchor(PopupAnchor { parent, position })`.
   3. Constructs a `Sugarloaf` for it (mirror existing
      `Screen::new` init path, minus terminal grid).
   4. Sets `menu.open(0.0, 0.0, read_only)`.
   5. Inserts into `Router.popup_menus`.
- [x] In `application.rs::window_event` (top of function, Linux-only
  cfg), check `Router.popup_menus.contains_key(&window_id)` and
  dispatch to `Application::handle_popup_menu_event` before the normal
  route lookup.
- [x] On `MouseInput::Left::Pressed` inside a popup: `menu.hit_test`
  → on action, send `RioEvent::NativeContextMenuAction(id)` via
  `event_proxy.send_event` and remove the popup from
  `Router.popup_menus`.
- [x] On `Focused(false)`, `CloseRequested`, or `KeyboardInput`
  with `Escape`: remove the popup. Drop cascades through Sugarloaf
  → wgpu surface → underlying X11/Wayland window.
- [x] Reuse `PaneTitlebarMenu::render()` for drawing inside the popup.

**Automated Verification**:
- [x] Unit test `popup_height_positive` in
  `popup_menu_window::tests` covers the layout helper.
- [x] `cargo build -p rioterm --no-default-features --features=wayland`
  passes.
- [x] `cargo build -p rioterm --no-default-features --features=x11`
  passes.

**Manual Verification**:
- [ ] On Wayland (GNOME, sway, KDE): right-click in pane → popup window
  appears anchored at the cursor, can extend past the parent window's
  edge, dismisses on click-outside (popup_done). Selecting "Split
  Vertically" splits the originating pane.
- [ ] On X11 (i3, GNOME/X11): right-click → override-redirect popup at
  cursor; click outside dismisses; Escape on focus-out dismisses;
  actions dispatch correctly.

### F. Event plumbing & Screen dispatcher refactor

**Tasks**:
- [x] In `rio-backend/src/event/mod.rs` (around line 88, the existing
  `RioEvent` enum), add variant `NativeContextMenuAction(u32)`. The
  `u32` is an opaque action id resolved via `Router.menu_action_registry`
  in the rioterm crate; `rio-backend` deliberately does NOT learn
  about `MenuAction`.
- [x] In `frontends/rioterm/src/screen/mod.rs:2789-2844`, refactor
  `Screen::handle_titlebar_menu_click(&mut self, clipboard: &mut
  Clipboard) -> bool`:
  1. Extract the `match self.renderer.pane_titlebar_menu.hit_test(...)`
     body into a new method `Screen::dispatch_menu_action(&mut self,
     action: MenuAction, clipboard: &mut Clipboard)`.
  2. `handle_titlebar_menu_click` becomes a thin wrapper: hit-test,
     close on `Dismiss`, forward other actions to `dispatch_menu_action`.
  3. Behaviour for the existing Sugarloaf keyboard path is unchanged.
- [x] In `frontends/rioterm/src/application.rs::user_event` handler,
  add a new arm matching `RioEventType::Rio(RioEvent::NativeContextMenuAction(id))`:
  1. Look up `action = self.router.menu_action_registry.remove(&id)`.
  2. Look up `(window_id, _pane_id) = self.router.pending_menu_origin.take()`.
     **Adjusted**: dropped the pane-id pair — pane is resolved via
     `route.window.screen.context_manager.current()` at dispatch
     time, matching the existing Sugarloaf-path semantics. Origin is
     now a single `WindowId`.
  3. Fetch the route for `window_id`, call
     `route.window.screen.dispatch_menu_action(action, &mut self.router.clipboard)`.
  4. Request redraw.
- [x] In `Router`, add fields
  `pending_menu_origin: Option<WindowId>` and
  `menu_action_registry: HashMap<u32, MenuAction>`. Both reset before
  each menu open.

**Automated Verification**:
- [ ] Unit test
  `application::tests::native_menu_action_routes_to_origin_pane`
  (Unit) — fabricate a `RioEvent::NativeContextMenuAction`, verify
  the dispatcher calls `handle_titlebar_menu_click` with the correct
  action on the correct pane.

### G. Dismiss handling

**Tasks**:
- [ ] macOS / Windows: muda's `show_context_menu_for_*` calls block
  until dismiss — deferred to muda follow-up plan.
- [ ] Linux Wayland: `xdg_popup::popup_done` — deferred to follow-up
  xdg_popup plan. Wayland users see the in-canvas Sugarloaf popover
  (no regression vs. pre-this-PR behaviour).
- [x] Linux X11: handle `Focused(false)`, `CloseRequested`, and
  `KeyboardInput { logical_key: Escape }` on the popup window →
  remove from `Router.popup_menus`.
- [x] Popup-window `Drop` tears down Sugarloaf + the X11 window via
  standard ownership (no explicit close call needed).

**Automated Verification**:
- [ ] Unit test
  `popup_menu_window::tests::focus_loss_marks_for_close`
  (Unit) — verify `handle_focus(false)` returns `true`.
- [ ] Unit test
  `popup_menu_window::tests::escape_marks_for_close` (Unit).

### H. Documentation

**Tasks**:
- [x] Append a "Native Context Menu on Mouse Right-Click (Phase 19)"
  section to `FORK.md` summarising per-platform behaviour + follow-up
  link.
- [x] Write follow-up plan
  `docs/agents/plans/2026-06-24-native-context-menu-macos-windows.md`
  covering macOS + Windows muda integration and Wayland xdg_popup
  protocol support — both deferred from this session.

**Automated Verification**:
- [x] `test -f docs/agents/plans/2026-06-24-native-context-menu-macos-windows.md`
  passes (file exists).

---

## References

- Existing Sugarloaf popover: `frontends/rioterm/src/renderer/pane_titlebar_menu.rs`
- Right-click dispatch site: `frontends/rioterm/src/application.rs:1315-1342`
- Action dispatcher (reused): `frontends/rioterm/src/screen/mod.rs:2789-2844`
- Raw window handle access: `frontends/rioterm/src/router/mod.rs:709-710`
- rio-window x11 popup hint already present: `rio-window/src/platform_impl/linux/x11/util/hint.rs:36`
- rio-window winit-style WindowAttributes: `rio-window/src/window.rs:144-200`
- sctk popup example pattern: `smithay-client-toolkit 0.19.2` `examples/popup.rs`
- muda ContextMenu trait: `https://docs.rs/muda/0.19/muda/trait.ContextMenu.html`
- muda winit integration sample (Linux explicitly skipped):
  `https://github.com/tauri-apps/muda/blob/dev/examples/winit.rs`
- Prior fork phase plan format: `docs/agents/plans/2026-06-19-tilix-tiling.md`
