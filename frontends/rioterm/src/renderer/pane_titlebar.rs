// Copyright (c) 2023-present, Raphael Amorim.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use rio_backend::sugarloaf::text::DrawOpts;
use rio_backend::sugarloaf::Sugarloaf;
use taffy::NodeId;

/// Tuple describing a pane for the titlebar renderer:
/// `(node_id, layout_rect_physical_px, title, is_active, is_maximized, sync_on, scale_factor)`
pub type PaneTitlebarEntry = (NodeId, [f32; 4], String, bool, bool, bool, f32);

pub const PANE_TITLEBAR_HEIGHT: f32 = 24.0;
const TITLEBAR_FONT_SIZE: f32 = 11.0;
const BTN_W: f32 = 20.0;
const BTN_MARGIN: f32 = 4.0;
// Render-order (z-layer) for all titlebar elements — sits above pane content.
const TITLEBAR_ORDER: u8 = 5;

/// Per-pane areas returned from `render()` so the mouse handler can hit-test.
#[derive(Debug, Clone, Copy)]
pub struct TitlebarHitMap {
    pub node_id: NodeId,
    /// Full titlebar rect [x, y, w, h] in logical (unscaled) pixels
    pub titlebar: [f32; 4],
    /// Sub-rects for button hit-testing [x, y, w, h] — logical pixels
    pub close_btn: [f32; 4],
    pub maximize_btn: [f32; 4],
    pub menu_btn: [f32; 4],
    pub sync_icon: [f32; 4],
    /// Drag handle = titlebar minus buttons (the centre region).
    /// Populated for future phase 9 drag-and-drop; tested in unit tests.
    #[allow(dead_code)]
    pub drag_handle: [f32; 4],
}

pub struct PaneTitlebar {
    pub hitmaps: Vec<TitlebarHitMap>,
}

impl PaneTitlebar {
    pub fn new() -> Self {
        Self {
            hitmaps: Vec::new(),
        }
    }

    /// Render all per-pane titlebars and rebuild the hitmap.
    ///
    /// Call once per frame when `pane.titlebar` is enabled and
    /// `panel_count() > 1`.
    ///
    /// `panes` contains one entry per visible pane:
    /// `(node_id, layout_rect, title_text, is_active, is_maximized, sync_on)`
    ///
    /// `layout_rect` is `[x, y, width, height]` in **physical** pixels (as
    /// stored on `ContextGridItem::layout_rect`).  `scale` converts logical →
    /// physical.
    pub fn render(
        &mut self,
        sugarloaf: &mut Sugarloaf,
        panes: &[PaneTitlebarEntry],
        colors: &TitlebarColors,
    ) {
        self.hitmaps.clear();

        for &(node_id, rect, ref title, is_active, is_maximized, _sync_on, scale) in panes
        {
            // layout_rect is in physical pixels; convert to logical for the
            // sugarloaf draw calls (sugarloaf.rect re-scales internally).
            let [px, py, pw, _ph] = rect;
            let x = px / scale;
            let y = py / scale;
            let w = pw / scale;

            // The titlebar sits ABOVE the pane's content area.
            let tb_h = PANE_TITLEBAR_HEIGHT;
            let tb_y = y - tb_h;

            // Background
            let bg = if is_active {
                colors.active_bg
            } else {
                colors.inactive_bg
            };
            sugarloaf.rect(None, x, tb_y, w, tb_h, bg, 0.0, TITLEBAR_ORDER);

            // Bottom separator line (1 px)
            sugarloaf.rect(
                None,
                x,
                tb_y + tb_h - 1.0,
                w,
                1.0,
                colors.separator,
                0.0,
                TITLEBAR_ORDER,
            );

            // ── Button layout (right-to-left): [✕] [⛶/restore] [⋯] ─────────
            let close_x = x + w - BTN_W - BTN_MARGIN;
            let max_x = close_x - BTN_W - BTN_MARGIN;
            let menu_x = max_x - BTN_W - BTN_MARGIN;
            // sync icon on the left
            let sync_x = x + BTN_MARGIN;

            let fg = if is_active {
                colors.active_fg
            } else {
                colors.inactive_fg
            };

            // Draw button backgrounds (subtle hover-alike tint so they're
            // visually distinct from the title text area).
            let btn_bg = if is_active {
                colors.btn_bg_active
            } else {
                colors.btn_bg_inactive
            };

            // Close button background
            sugarloaf.rect(
                None,
                close_x,
                tb_y,
                BTN_W,
                tb_h,
                btn_bg,
                0.0,
                TITLEBAR_ORDER,
            );
            // Maximize button background
            sugarloaf.rect(
                None,
                max_x,
                tb_y,
                BTN_W,
                tb_h,
                btn_bg,
                0.0,
                TITLEBAR_ORDER,
            );
            // Menu button background
            sugarloaf.rect(
                None,
                menu_x,
                tb_y,
                BTN_W,
                tb_h,
                btn_bg,
                0.0,
                TITLEBAR_ORDER,
            );

            // Button text labels
            let btn_opts = DrawOpts {
                font_size: TITLEBAR_FONT_SIZE,
                color: color_u8(fg),
                ..DrawOpts::default()
            };
            let text_y = tb_y + (tb_h - TITLEBAR_FONT_SIZE) / 2.0;

            // Close ✕
            let ui = sugarloaf.text_mut();
            let close_label = "✕";
            let close_w = ui.measure(close_label, &btn_opts);
            ui.draw(
                close_x + (BTN_W - close_w) / 2.0,
                text_y,
                close_label,
                &btn_opts,
            );

            // Maximize ⛶ or restore ⊡
            let max_label = if is_maximized { "⊡" } else { "⛶" };
            let ui = sugarloaf.text_mut();
            let max_w = ui.measure(max_label, &btn_opts);
            ui.draw(
                max_x + (BTN_W - max_w) / 2.0,
                text_y,
                max_label,
                &btn_opts,
            );

            // Menu ⋯
            let ui = sugarloaf.text_mut();
            let menu_label = "⋯";
            let menu_w = ui.measure(menu_label, &btn_opts);
            ui.draw(
                menu_x + (BTN_W - menu_w) / 2.0,
                text_y,
                menu_label,
                &btn_opts,
            );

            // Sync icon ⌨
            let sync_fg = colors.sync_icon_fg;
            let sync_opts = DrawOpts {
                font_size: TITLEBAR_FONT_SIZE,
                color: color_u8(sync_fg),
                ..DrawOpts::default()
            };
            let ui = sugarloaf.text_mut();
            let sync_label = "⌨";
            let sync_w = ui.measure(sync_label, &sync_opts);
            ui.draw(
                sync_x + (BTN_W - sync_w) / 2.0,
                text_y,
                sync_label,
                &sync_opts,
            );

            // ── Title text (centre of remaining area, truncated) ─────────────
            // Text region spans from after sync icon to before menu button.
            let text_area_x = sync_x + BTN_W + BTN_MARGIN;
            let text_area_w = (menu_x - BTN_MARGIN) - text_area_x;
            if text_area_w > 0.0 {
                let title_opts = DrawOpts {
                    font_size: TITLEBAR_FONT_SIZE,
                    color: color_u8(fg),
                    ..DrawOpts::default()
                };
                let ui = sugarloaf.text_mut();
                let full_w = ui.measure(title, &title_opts);
                let display_title: std::borrow::Cow<str> = if full_w <= text_area_w {
                    std::borrow::Cow::Borrowed(title.as_str())
                } else {
                    fit_title_to_width(ui, title, text_area_w, &title_opts)
                };
                let dw = ui.measure(&display_title, &title_opts);
                // Centre the title within the text area.
                let title_x = text_area_x + ((text_area_w - dw) / 2.0).max(0.0);
                ui.draw(title_x, text_y, &display_title, &title_opts);
            }

            // ── Build hit map ────────────────────────────────────────────────
            let drag_start_x = sync_x + BTN_W + BTN_MARGIN;
            let drag_end_x = menu_x - BTN_MARGIN;
            let drag_w = (drag_end_x - drag_start_x).max(0.0);

            self.hitmaps.push(TitlebarHitMap {
                node_id,
                titlebar: [x, tb_y, w, tb_h],
                close_btn: [close_x, tb_y, BTN_W, tb_h],
                maximize_btn: [max_x, tb_y, BTN_W, tb_h],
                menu_btn: [menu_x, tb_y, BTN_W, tb_h],
                sync_icon: [sync_x, tb_y, BTN_W, tb_h],
                drag_handle: [drag_start_x, tb_y, drag_w, tb_h],
            });
        }
    }

    /// Hit-test a logical-pixel mouse position.
    ///
    /// Returns `(node_id, hit_result)` if any titlebar was hit.
    pub fn hit_test(
        &self,
        mouse_x: f32,
        mouse_y: f32,
    ) -> Option<(NodeId, TitlebarHitResult)> {
        for hm in &self.hitmaps {
            if !point_in_rect(mouse_x, mouse_y, hm.titlebar) {
                continue;
            }
            let result = if point_in_rect(mouse_x, mouse_y, hm.close_btn) {
                TitlebarHitResult::Close
            } else if point_in_rect(mouse_x, mouse_y, hm.maximize_btn) {
                TitlebarHitResult::Maximize
            } else if point_in_rect(mouse_x, mouse_y, hm.menu_btn) {
                TitlebarHitResult::Menu
            } else if point_in_rect(mouse_x, mouse_y, hm.sync_icon) {
                TitlebarHitResult::SyncToggle
            } else {
                TitlebarHitResult::DragHandle
            };
            return Some((hm.node_id, result));
        }
        None
    }
}

impl Default for PaneTitlebar {
    fn default() -> Self {
        Self::new()
    }
}

/// Which region of a titlebar was clicked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitlebarHitResult {
    Close,
    Maximize,
    Menu,
    SyncToggle,
    DragHandle,
}

/// Colors used to render the per-pane titlebar.
pub struct TitlebarColors {
    pub active_bg: [f32; 4],
    pub inactive_bg: [f32; 4],
    pub active_fg: [f32; 4],
    pub inactive_fg: [f32; 4],
    pub separator: [f32; 4],
    pub btn_bg_active: [f32; 4],
    pub btn_bg_inactive: [f32; 4],
    pub sync_icon_fg: [f32; 4],
}

impl Default for TitlebarColors {
    fn default() -> Self {
        Self {
            active_bg: [0.20, 0.20, 0.20, 1.0],
            inactive_bg: [0.13, 0.13, 0.13, 1.0],
            active_fg: [1.0, 1.0, 1.0, 1.0],
            inactive_fg: [0.65, 0.65, 0.65, 1.0],
            separator: [0.30, 0.30, 0.30, 1.0],
            btn_bg_active: [0.0, 0.0, 0.0, 0.0], // transparent — just text
            btn_bg_inactive: [0.0, 0.0, 0.0, 0.0],
            sync_icon_fg: [0.50, 0.50, 0.50, 1.0],
        }
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

#[inline]
fn point_in_rect(x: f32, y: f32, rect: [f32; 4]) -> bool {
    x >= rect[0] && x < rect[0] + rect[2] && y >= rect[1] && y < rect[1] + rect[3]
}

/// Truncate `title` with a trailing `…` so it fits within `max_width`.
/// Uses the already-borrowed `ui` to measure character widths.
fn fit_title_to_width<'a>(
    ui: &mut rio_backend::sugarloaf::text::Text,
    title: &'a str,
    max_width: f32,
    opts: &DrawOpts,
) -> std::borrow::Cow<'a, str> {
    const ELLIPSIS: char = '…';
    let ellipsis_w = {
        let s = ELLIPSIS.to_string();
        ui.measure(&s, opts)
    };
    let mut accumulated: f32 = 0.0;
    let mut truncate_ix: usize = 0;
    for (ix, c) in title.char_indices() {
        if accumulated + ellipsis_w <= max_width {
            truncate_ix = ix;
        }
        let ch_s = c.to_string();
        accumulated += ui.measure(&ch_s, opts);
        if accumulated > max_width {
            let mut out = String::with_capacity(truncate_ix + ELLIPSIS.len_utf8());
            out.push_str(&title[..truncate_ix]);
            out.push(ELLIPSIS);
            return std::borrow::Cow::Owned(out);
        }
    }
    std::borrow::Cow::Borrowed(title)
}

#[inline]
fn color_u8(c: [f32; 4]) -> [u8; 4] {
    [
        (c[0].clamp(0.0, 1.0) * 255.0) as u8,
        (c[1].clamp(0.0, 1.0) * 255.0) as u8,
        (c[2].clamp(0.0, 1.0) * 255.0) as u8,
        (c[3].clamp(0.0, 1.0) * 255.0) as u8,
    ]
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a `TitlebarHitMap` for a pane at (x, y, w, h) directly,
    /// bypassing the sugarloaf drawing layer so we can test the hit-test logic
    /// without a GPU context.
    fn make_hitmap(
        node_id: NodeId,
        x: f32,
        y: f32,
        w: f32,
        tb_h: f32,
    ) -> TitlebarHitMap {
        let close_x = x + w - BTN_W - BTN_MARGIN;
        let max_x = close_x - BTN_W - BTN_MARGIN;
        let menu_x = max_x - BTN_W - BTN_MARGIN;
        let sync_x = x + BTN_MARGIN;
        let drag_start = sync_x + BTN_W + BTN_MARGIN;
        let drag_end = menu_x - BTN_MARGIN;
        TitlebarHitMap {
            node_id,
            titlebar: [x, y, w, tb_h],
            close_btn: [close_x, y, BTN_W, tb_h],
            maximize_btn: [max_x, y, BTN_W, tb_h],
            menu_btn: [menu_x, y, BTN_W, tb_h],
            sync_icon: [sync_x, y, BTN_W, tb_h],
            drag_handle: [drag_start, y, (drag_end - drag_start).max(0.0), tb_h],
        }
    }

    fn node(raw: u64) -> NodeId {
        NodeId::from(raw)
    }

    #[test]
    fn hitmap_buttons_within_titlebar_rect() {
        let n = node(1);
        // Pane at (0, 100, 400, 300), scale 1.0 → titlebar at y=76..100.
        let tb_y = 100.0 - PANE_TITLEBAR_HEIGHT;
        let hm = make_hitmap(n, 0.0, tb_y, 400.0, PANE_TITLEBAR_HEIGHT);

        let mut tb = PaneTitlebar::new();
        tb.hitmaps.push(hm);

        // Buttons are contained in the titlebar rect.
        let [cx, cy, cw, ch] = tb.hitmaps[0].close_btn;
        assert!(
            cx >= 0.0 && cx + cw <= 400.0,
            "close_btn x out of pane width"
        );
        assert!(
            cy >= tb_y && cy + ch <= tb_y + PANE_TITLEBAR_HEIGHT,
            "close_btn y outside titlebar"
        );

        let [mx, my, mw, mh] = tb.hitmaps[0].maximize_btn;
        assert!(
            mx >= 0.0 && mx + mw <= 400.0,
            "maximize_btn x out of pane width"
        );
        assert!(
            my >= tb_y && my + mh <= tb_y + PANE_TITLEBAR_HEIGHT,
            "maximize_btn y outside titlebar"
        );
    }

    #[test]
    fn hit_test_close_button() {
        let n = node(2);
        let tb_y = 76.0;
        let hm = make_hitmap(n, 0.0, tb_y, 400.0, PANE_TITLEBAR_HEIGHT);
        let mut tb = PaneTitlebar::new();
        tb.hitmaps.push(hm.clone());

        // Centre of the close button.
        let cx = hm.close_btn[0] + BTN_W / 2.0;
        let cy = tb_y + PANE_TITLEBAR_HEIGHT / 2.0;
        let result = tb.hit_test(cx, cy);
        assert_eq!(result, Some((n, TitlebarHitResult::Close)));
    }

    #[test]
    fn hit_test_maximize_button() {
        let n = node(3);
        let tb_y = 76.0;
        let hm = make_hitmap(n, 0.0, tb_y, 400.0, PANE_TITLEBAR_HEIGHT);
        let mut tb = PaneTitlebar::new();
        tb.hitmaps.push(hm.clone());

        let mx = hm.maximize_btn[0] + BTN_W / 2.0;
        let my = tb_y + PANE_TITLEBAR_HEIGHT / 2.0;
        let result = tb.hit_test(mx, my);
        assert_eq!(result, Some((n, TitlebarHitResult::Maximize)));
    }

    #[test]
    fn hit_test_drag_handle() {
        let n = node(4);
        let tb_y = 76.0;
        let hm = make_hitmap(n, 0.0, tb_y, 400.0, PANE_TITLEBAR_HEIGHT);
        let drag_start = hm.drag_handle[0];
        let drag_w = hm.drag_handle[2];
        let mut tb = PaneTitlebar::new();
        tb.hitmaps.push(hm);

        // Point in the middle of the drag handle.
        let dx = drag_start + drag_w / 2.0;
        let dy = tb_y + PANE_TITLEBAR_HEIGHT / 2.0;
        let result = tb.hit_test(dx, dy);
        assert_eq!(result, Some((n, TitlebarHitResult::DragHandle)));
    }

    #[test]
    fn point_outside_titlebar_returns_none() {
        let n = node(5);
        let tb_y = 76.0;
        let hm = make_hitmap(n, 0.0, tb_y, 400.0, PANE_TITLEBAR_HEIGHT);
        let mut tb = PaneTitlebar::new();
        tb.hitmaps.push(hm);

        // Below the titlebar.
        assert_eq!(tb.hit_test(50.0, tb_y + PANE_TITLEBAR_HEIGHT + 1.0), None);
        // Above the titlebar.
        assert_eq!(tb.hit_test(50.0, tb_y - 1.0), None);
        // Entirely to the right.
        assert_eq!(tb.hit_test(401.0, tb_y + 5.0), None);
    }

    #[test]
    fn point_in_rect_boundaries() {
        let rect = [10.0_f32, 20.0, 30.0, 15.0];
        // Top-left corner — inside.
        assert!(point_in_rect(10.0, 20.0, rect));
        // Bottom-right boundary — exclusive.
        assert!(!point_in_rect(40.0, 35.0, rect));
        // One pixel inside the right/bottom.
        assert!(point_in_rect(39.9, 34.9, rect));
    }
}
