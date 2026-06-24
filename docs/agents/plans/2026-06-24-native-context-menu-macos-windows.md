---
date: 2026-06-24
git_commit: TBD
branch: main
topic: "Native context menu — macOS + Windows via muda + Wayland xdg_popup"
tags: [plan, rioterm, rio-window, menu, macos, windows, wayland, muda, follow-up]
status: draft
depends_on: 2026-06-24-native-context-menu.md
---

# Follow-Up: macOS + Windows muda Integration & Wayland xdg_popup

## Why this is a follow-up

The original Phase 19 plan (`2026-06-24-native-context-menu.md`) was
scoped down at implementation time: only Linux X11 ships natively;
Wayland keeps the in-canvas Sugarloaf popover; macOS and Windows still
use the in-canvas popover too. This plan covers the three deferred
backends.

## Current State (post Phase 19)

Already in place from the parent plan:

- `muda = "0.19"` target-gated dep on macOS + Windows
  (`frontends/rioterm/Cargo.toml`)
- `RioEvent::NativeContextMenuAction(u32)` user-event variant
  (`rio-backend/src/event/mod.rs`)
- `Router.menu_action_registry: HashMap<u32, MenuAction>`,
  `Router.pending_menu_origin: Option<WindowId>`,
  `Router::open_native_context_menu`
  (`frontends/rioterm/src/router/mod.rs`)
- `Screen::dispatch_menu_action` central dispatcher
  (`frontends/rioterm/src/screen/mod.rs`)
- `native_context_menu.rs::build_registry()` shared id-assignment
- `application.rs::user_event` arm that drains the registry +
  `pending_menu_origin` and dispatches via `Screen::dispatch_menu_action`
- application right-click branch already calls
  `self.router.open_native_context_menu(...)` and falls back to the
  Sugarloaf popover when it returns `false`

What's still stubbed: the macOS / Windows / Wayland branches inside
`Router::open_native_context_menu` return `false` immediately.

## Desired End State

- **macOS**: mouse right-click in a pane shows a real `NSMenu`
  rendered by AppKit, themed by the system, capable of overflowing
  the parent window. Selection routes through the existing
  `RioEvent::NativeContextMenuAction` event.
- **Windows**: same with `HMENU` + `TrackPopupMenuEx`.
- **Linux Wayland**: same UX as the Linux X11 implementation
  (borderless popup that can overflow the parent), backed by a real
  `xdg_popup` surface inside `rio-window`.

## What We're NOT Doing

- Not changing the action vocabulary (`MenuAction`).
- Not changing the in-canvas Sugarloaf popover (still the keyboard
  fallback).
- Not adding muda's GTK Linux backend (rio-window is not GTK).

## Phase 1: macOS + Windows via muda

### Tasks

- [ ] In `Router::new`, register the global muda menu-event handler
  **once**:
  ```rust
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  {
      let proxy = event_proxy.clone();
      muda::MenuEvent::set_event_handler(Some(move |ev| {
          if let Ok(id) = ev.id.0.parse::<u32>() {
              proxy.send_event(
                  RioEventType::Rio(RioEvent::NativeContextMenuAction(id)),
                  rio_window::window::WindowId::dummy(),
              );
          }
      }));
  }
  ```
  Note: `pending_menu_origin` is already set by
  `Router::open_native_context_menu` before the menu is shown, so the
  `user_event` handler can drain it without the closure needing the
  origin.
- [ ] Inside the `#[cfg(any(target_os = "macos", target_os = "windows"))]`
  branch of `Router::open_native_context_menu`:
  1. Build a `muda::Menu` from the same `PaneTitlebarMenu::entries()`
     list, using `format!("{i}")` for each id (matching
     `build_registry()`'s scheme).
  2. Compute physical cursor position as
     `parent.outer_position() + cursor_logical * scale`.
  3. macOS: extract `ns_view` from
     `RawWindowHandle::AppKit(handle).ns_view.as_ptr()`, call
     `menu.show_context_menu_for_nsview(ns_view, Some(Position::Physical(x, y)))`.
  4. Windows: extract `hwnd` from
     `RawWindowHandle::Win32(handle).hwnd.get()`, call
     `menu.show_context_menu_for_hwnd(hwnd as isize, Some(Position::Physical(x, y)))`.
  5. Return `true`.
- [ ] Note: muda's `show_context_menu_for_*` is synchronous on both
  platforms; the call blocks until the menu is dismissed. Clear
  `pending_menu_origin` if no event arrived (cancellation case).

### Automated Verification

- [ ] `cargo build -p rioterm` on macOS passes.
- [ ] `cargo build -p rioterm` on Windows passes.
- [ ] `cargo clippy -p rioterm --all-targets -- -D warnings` clean on
  both platforms.

### Manual Verification

- [ ] macOS: right-click in pane → NSMenu at cursor with system theme.
  Selecting "Split Vertically" splits the pane. Menu overflows the
  parent window when invoked near the right/bottom edge.
- [ ] Windows 11: right-click → HMENU at cursor, follows light/dark
  theme, same dispatch + overflow check.

## Phase 2: Linux Wayland via xdg_popup

### Background

`rio-window`'s Wayland backend currently creates only `xdg_toplevel`
surfaces (see `rio-window/src/platform_impl/linux/wayland/window/mod.rs`
`Window::new`, which calls `state.xdg_shell.create_window(...)`).
`WindowState` is tightly coupled to `SctkWindow` (the toplevel
abstraction). Real popup support means a parallel `Popup` role plus
new dispatch wiring in `state.rs`.

### Tasks

- [ ] In `rio-window/src/window.rs`, add `pub struct PopupAnchor {
  pub parent: RawWindowHandle, pub position: PhysicalPosition<i32>,
  pub size: PhysicalSize<u32> }` and field
  `WindowAttributes.popup_anchor: Option<PopupAnchor>` + builder
  `WindowAttributes::with_popup_anchor`.
- [ ] In
  `rio-window/src/platform_impl/linux/wayland/window/mod.rs`,
  branch `Window::new` on `attributes.popup_anchor.is_some()`:
  1. Build `xdg_positioner` (size = popup_anchor.size, anchor_rect =
     1×1 at popup_anchor.position in parent-surface-local coords,
     anchor = TopLeft, gravity = BottomRight, constraint_adjustment
     = SLIDE_X | SLIDE_Y | FLIP_Y).
  2. Create the surface via `state.xdg_shell.create_popup(parent,
     positioner, qh)` instead of `create_window`.
  3. Track the popup handle in `WinitState` (per-WindowId map) so
     `xdg_popup::popup_done` can be routed to
     `WindowEvent::CloseRequested`.
- [ ] Update
  `rio-window/src/platform_impl/linux/wayland/state.rs` to dispatch
  `xdg_popup` events.
- [ ] In
  `frontends/rioterm/src/router/popup_menu_window.rs::build_popup_attrs`,
  on Wayland branch: call `with_popup_anchor(...)` using the parent
  handle and absolute cursor coordinates. Remove the Wayland
  early-return in `PopupMenuWindow::open`.

### Automated Verification

- [ ] `cargo build -p rio-window --no-default-features --features wayland`
  passes.
- [ ] `cargo build -p rioterm --no-default-features --features=wayland`
  passes.

### Manual Verification

- [ ] On GNOME Wayland: right-click in pane → popup window anchored
  at cursor, can overflow parent window, dismisses on click-outside
  (compositor sends popup_done).
- [ ] On sway: same.
- [ ] On KDE Plasma Wayland: same.

## References

- Parent plan: `docs/agents/plans/2026-06-24-native-context-menu.md`
- muda ContextMenu trait:
  `https://docs.rs/muda/0.19/muda/trait.ContextMenu.html`
- sctk popup example:
  `smithay-client-toolkit 0.19.2 examples/popup.rs`
- xdg-shell xdg_positioner spec:
  `https://wayland.app/protocols/xdg-shell#xdg_positioner`
