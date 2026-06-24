// Copyright (c) 2023-present, Raphael Amorim.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Native OS context-menu façade for the pane right-click menu.
//!
//! Per-platform dispatch:
//! - macOS: muda `NSMenu` via `show_context_menu_for_nsview` (TODO,
//!   tracked in a follow-up plan).
//! - Windows: muda `HMENU` via `show_context_menu_for_hwnd` (TODO,
//!   tracked in a follow-up plan).
//! - Linux X11: borderless rio-window popup hosting Sugarloaf — see
//!   [`crate::router::popup_menu_window`].
//! - Linux Wayland: returns `MenuShown::FallbackToSugarloaf` so the
//!   caller renders the in-canvas popover (no `xdg_popup` support yet
//!   in rio-window).
//!
//! All paths populate [`Router::menu_action_registry`] with the same
//! `u32` → [`MenuAction`] mapping; menu activations are forwarded back
//! to [`RioEvent::NativeContextMenuAction`] which the application
//! resolves to a pane and dispatches via
//! [`crate::screen::Screen::dispatch_menu_action`].

use std::collections::HashMap;

use rio_window::window::WindowId;

use crate::renderer::pane_titlebar_menu::MenuAction;

/// Outcome of opening a native context menu — tells the caller whether
/// they still need to invoke the Sugarloaf in-canvas fallback.
#[allow(dead_code)]
#[derive(Debug)]
pub enum MenuShown {
    /// A native menu (or popup window) was shown — no further action
    /// required by the caller.
    Native,
    /// The current platform/display cannot host a native menu; the
    /// caller should fall back to the in-canvas Sugarloaf popover.
    FallbackToSugarloaf,
}

/// Populate `registry` with the entries that will be displayed.
///
/// Shared by all backends so the muda follow-up and the Linux popup
/// agree on id assignment.
pub fn build_registry(
    entries: &[crate::renderer::pane_titlebar_menu::MenuEntry],
    registry: &mut HashMap<u32, MenuAction>,
) {
    registry.clear();
    for (i, entry) in entries.iter().enumerate() {
        if let Some(action) = &entry.action {
            registry.insert(i as u32, action.clone());
        }
    }
}

// Suppress unused-import warning on Linux where WindowId isn't referenced
// in the visible API (only by the future muda integration).
#[allow(dead_code)]
fn _force_windowid_used(_: WindowId) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::pane_titlebar_menu::PaneTitlebarMenu;

    #[test]
    fn registry_round_trip_covers_actionable_entries() {
        let mut menu = PaneTitlebarMenu::new();
        menu.open(0.0, 0.0, false);
        let entries = menu.entries();
        let mut registry = HashMap::new();
        build_registry(&entries, &mut registry);
        // Every actionable entry shows up exactly once.
        let actionable = entries.iter().filter(|e| e.action.is_some()).count();
        assert_eq!(registry.len(), actionable);
        // Ids are 0..entries.len() (skipping separators).
        for (id, action) in &registry {
            let entry = &entries[*id as usize];
            assert_eq!(entry.action.as_ref(), Some(action));
        }
    }
}
