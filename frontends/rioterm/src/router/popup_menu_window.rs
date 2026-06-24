// Copyright (c) 2023-present, Raphael Amorim.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! A minimal borderless popup window hosting the pane context menu.
//!
//! Spawned on Linux X11 only — Wayland keeps the in-canvas Sugarloaf
//! popover (no `xdg_popup` support yet); macOS/Windows go through muda
//! in a follow-up plan.

use std::collections::HashMap;
use std::error::Error;

use raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use rio_backend::config::renderer::Backend;
use rio_backend::sugarloaf::{
    layout::RootStyle, Sugarloaf, SugarloafBackend, SugarloafRenderer, SugarloafWindow,
    SugarloafWindowSize,
};
use rio_window::dpi::PhysicalSize;
use rio_window::event_loop::ActiveEventLoop;
use rio_window::window::{Window as RioWindow, WindowId, WindowLevel};

use crate::renderer::pane_titlebar_menu::{MenuAction, MenuEntry, PaneTitlebarMenu};

/// Layout constants shared with the in-canvas menu — duplicated here to
/// keep this module self-contained for popup sizing.
const MENU_WIDTH: f32 = 200.0;

/// A borderless top-level window that draws the pane context menu via
/// Sugarloaf. The action chosen by the user is forwarded back to the
/// origin window through the standard `RioEvent::NativeContextMenuAction`
/// event path.
pub struct PopupMenuWindow {
    pub winit_window: std::sync::Arc<RioWindow>,
    pub sugarloaf: Sugarloaf<'static>,
    pub menu: PaneTitlebarMenu,
    /// The window-id the user right-clicked in — the action is dispatched
    /// against this window's currently-focused pane.
    #[allow(dead_code)]
    pub origin_window: WindowId,
    /// Cached cursor position in logical pixels within the popup window.
    pub cursor_logical: (f32, f32),
    /// Marked `true` once a click has been handled and dispatch was
    /// triggered; the next event-loop tick removes us from the router.
    pub should_close: bool,
}

impl PopupMenuWindow {
    /// Build the popup window, initialize Sugarloaf, populate the
    /// per-popup `registry` (id → MenuAction) with the entries that
    /// will be shown.
    ///
    /// Only supported on Linux X11 in this build. Returns `Ok(None)`
    /// when the underlying display is Wayland (so callers fall back to
    /// the in-canvas popover).
    #[allow(clippy::too_many_arguments)]
    pub fn open(
        event_loop: &ActiveEventLoop,
        origin_window: WindowId,
        parent_display_handle: RawDisplayHandle,
        cursor_screen_pos: (i32, i32),
        read_only: bool,
        scale: f64,
        config: &rio_backend::config::Config,
        font_library: &rio_backend::sugarloaf::font::FontLibrary,
        registry: &mut HashMap<u32, MenuAction>,
    ) -> Result<Option<Self>, Box<dyn Error>> {
        // Wayland early-out: no xdg_popup support yet — let the caller
        // fall back to the in-canvas Sugarloaf popover.
        if matches!(parent_display_handle, RawDisplayHandle::Wayland(_)) {
            return Ok(None);
        }

        // Compute menu height by spinning up a throwaway widget with the
        // same read_only state and asking for its dimensions.
        let mut probe = PaneTitlebarMenu::new();
        probe.open(0.0, 0.0, read_only);
        let total_h = popup_height(&probe);

        let physical_width = (MENU_WIDTH * scale as f32).round() as u32;
        let physical_height = (total_h * scale as f32).round() as u32;

        // Build window attributes. Linux X11 path uses
        // `override_redirect` + `_NET_WM_WINDOW_TYPE_POPUP_MENU` for
        // proper popup semantics.
        let attrs = build_popup_attrs(
            cursor_screen_pos,
            PhysicalSize {
                width: physical_width,
                height: physical_height,
            },
        );

        let winit_window = event_loop.create_window(attrs)?;
        let winit_window = std::sync::Arc::new(winit_window);

        let raw_window_handle: RawWindowHandle = winit_window.window_handle()?.into();
        let raw_display_handle: RawDisplayHandle = winit_window.display_handle()?.into();
        let size = winit_window.inner_size();

        // Sugarloaf init.
        let sugarloaf_layout =
            RootStyle::new(scale as f32, config.fonts.size, config.line_height);
        let sugarloaf_window = SugarloafWindow {
            handle: raw_window_handle,
            display: raw_display_handle,
            scale: scale as f32,
            size: SugarloafWindowSize {
                width: size.width as f32,
                height: size.height as f32,
            },
        };

        let backend = if config.renderer.use_cpu {
            SugarloafBackend::Cpu
        } else {
            match config.renderer.backend {
                #[cfg(target_os = "linux")]
                Backend::Vulkan => SugarloafBackend::Vulkan,
                #[cfg(all(not(target_os = "linux"), feature = "wgpu"))]
                Backend::Vulkan => SugarloafBackend::Wgpu(wgpu::Backends::VULKAN),
                #[cfg(all(not(target_os = "linux"), not(feature = "wgpu")))]
                Backend::Vulkan => SugarloafBackend::Cpu,
                #[cfg(target_os = "macos")]
                Backend::Metal => SugarloafBackend::Metal,
                #[cfg(all(feature = "wgpu", target_arch = "wasm32"))]
                Backend::Webgpu => SugarloafBackend::Wgpu(
                    wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
                ),
                #[cfg(all(feature = "wgpu", not(target_arch = "wasm32")))]
                Backend::Webgpu => SugarloafBackend::Wgpu(wgpu::Backends::all()),
                #[cfg(not(feature = "wgpu"))]
                Backend::Webgpu => SugarloafBackend::Cpu,
            }
        };

        let renderer = SugarloafRenderer {
            backend,
            font_features: config.fonts.features.clone(),
            colorspace: config.window.colorspace.to_sugarloaf_colorspace(),
        };

        let sugarloaf = match Sugarloaf::new(
            sugarloaf_window,
            renderer,
            font_library,
            sugarloaf_layout,
        ) {
            Ok(instance) => instance,
            Err(instance_with_errors) => instance_with_errors.instance,
        };

        // Populate registry with the entries that will be drawn.
        registry.clear();
        let entries = probe.entries();
        for (i, entry) in entries.iter().enumerate() {
            if let Some(action) = &entry.action {
                registry.insert(i as u32, action.clone());
            }
        }

        let mut menu = PaneTitlebarMenu::new();
        menu.open(0.0, 0.0, read_only);

        // Drive the first frame — without this the popup window can
        // appear blank on some compositors until the first input event
        // arrives.
        winit_window.request_redraw();

        Ok(Some(Self {
            winit_window,
            sugarloaf,
            menu,
            origin_window,
            cursor_logical: (0.0, 0.0),
            should_close: false,
        }))
    }

    /// Redraw the menu into the popup's Sugarloaf.
    pub fn render(&mut self) {
        let scale = self.sugarloaf.scale_factor();
        self.menu.render(&mut self.sugarloaf, scale);
        self.sugarloaf.render();
    }

    /// Update hover state from a logical-pixel mouse position.
    pub fn on_cursor_moved(&mut self, x: f32, y: f32) {
        self.cursor_logical = (x, y);
        self.menu.hover(x, y);
        self.winit_window.request_redraw();
    }

    /// Resolve a left-button press inside the popup to a menu-action id.
    ///
    /// Returns `Some(id)` if the click landed on an actionable entry —
    /// caller should send `RioEvent::NativeContextMenuAction(id)` and
    /// then call [`Self::mark_for_close`].
    pub fn handle_click(
        &mut self,
        x: f32,
        y: f32,
        registry: &HashMap<u32, MenuAction>,
    ) -> Option<u32> {
        match self.menu.hit_test(x, y) {
            Some(MenuAction::Dismiss) | None => {
                self.should_close = true;
                None
            }
            Some(action) => {
                // Find the id we registered for this action.
                let id = registry
                    .iter()
                    .find(|(_, a)| **a == action)
                    .map(|(id, _)| *id);
                self.should_close = true;
                id
            }
        }
    }

    /// Mark the popup as ready to be removed by the router.
    pub fn mark_for_close(&mut self) {
        self.should_close = true;
    }
}

/// Total popup height in logical pixels — keep in sync with `PaneTitlebarMenu`
/// rendering (entry height + separators + padding). Delegated to a probe
/// instance to avoid duplicating the layout math.
fn popup_height(menu: &PaneTitlebarMenu) -> f32 {
    // The menu's `total_height` field is private; iterate entries and
    // accumulate using the same constants the renderer uses. Since those
    // constants are also private, approximate from entries() — every
    // actionable row is 22px, separator is 5px, plus 8px outer padding.
    let entries: Vec<MenuEntry> = menu.entries();
    let mut h: f32 = 8.0;
    for e in &entries {
        if e.is_separator {
            h += 5.0;
        } else {
            h += 22.0;
        }
    }
    h
}

fn build_popup_attrs(
    cursor_screen_pos: (i32, i32),
    size: PhysicalSize<u32>,
) -> rio_window::window::WindowAttributes {
    use rio_window::dpi::PhysicalPosition;

    // NOTE: deliberately NOT using `with_parent_window` here. On X11
    // setting a parent_window turns the new window into an X-server
    // child of that parent — visually clipped to its bounds — which
    // defeats the purpose of a context menu that needs to overflow
    // the terminal window. Instead the popup is a top-level
    // (root-child) override-redirect window placed at absolute screen
    // coordinates and identified to WMs via the popup-menu hint below.
    #[allow(unused_mut)]
    let mut attrs = rio_window::window::WindowAttributes::default()
        .with_decorations(false)
        .with_resizable(false)
        .with_inner_size(size)
        .with_position(PhysicalPosition::new(
            cursor_screen_pos.0,
            cursor_screen_pos.1,
        ))
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_title("rio popup menu");

    // X11 popup hints — only present when rio-window is built with the
    // x11 feature. Wayland-only Linux builds skip the hints (the early
    // Wayland check in `open()` means we never reach this branch on
    // pure Wayland anyway).
    #[cfg(all(target_os = "linux", feature = "x11"))]
    {
        use rio_window::platform::x11::{WindowAttributesExtX11, WindowType};
        attrs = attrs
            .with_override_redirect(true)
            .with_x11_window_type(vec![WindowType::PopupMenu]);
    }

    attrs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_height_positive() {
        let mut m = PaneTitlebarMenu::new();
        m.open(0.0, 0.0, false);
        assert!(popup_height(&m) > 0.0);
    }
}
