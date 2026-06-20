// Copyright (c) 2023-present, Raphael Amorim.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.
//
// sidebar.rs — collapsible left-side session panel (NavigationMode::Sidebar, US-8.5)

use rio_backend::sugarloaf::text::DrawOpts;
use rio_backend::sugarloaf::Sugarloaf;

pub const SIDEBAR_WIDTH: f32 = 180.0;
pub const SIDEBAR_WIDTH_COLLAPSED: f32 = 0.0;

const ROW_HEIGHT: f32 = 34.0;
const TITLE_FONT_SIZE: f32 = 12.0;
const TEXT_PADDING_X: f32 = 12.0;

pub struct Sidebar {
    pub is_open: bool,
    /// Current width — snaps between SIDEBAR_WIDTH and SIDEBAR_WIDTH_COLLAPSED.
    pub current_width: f32,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            is_open: true,
            current_width: SIDEBAR_WIDTH,
        }
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
        self.current_width = if self.is_open {
            SIDEBAR_WIDTH
        } else {
            SIDEBAR_WIDTH_COLLAPSED
        };
    }

    /// Returns the current rendered width of the sidebar (0 when collapsed).
    /// Used by hit-testing, margin computation, and tests. Future phases will
    /// call this to offset the terminal render area.
    #[allow(dead_code)]
    pub fn effective_width(&self) -> f32 {
        self.current_width
    }

    /// Render the sidebar.
    ///
    /// `session_titles` is a slice of (index, title, is_active) tuples.
    pub fn render(
        &self,
        sugarloaf: &mut Sugarloaf,
        window_height: f32,
        session_titles: &[(usize, String, bool)],
        colors: &SidebarColors,
    ) {
        if self.current_width <= 0.0 {
            return;
        }

        let w = self.current_width;

        // Background panel
        sugarloaf.rect(None, 0.0, 0.0, w, window_height, colors.bg, 0.0, 0);

        // Thin right-edge border
        let border_color = [
            colors.fg[0] * 0.4,
            colors.fg[1] * 0.4,
            colors.fg[2] * 0.4,
            0.6,
        ];
        sugarloaf.rect(None, w - 0.5, 0.0, 0.5, window_height, border_color, 0.1, 1);

        // Session rows
        for (slot, (_idx, title, is_active)) in session_titles.iter().enumerate() {
            let row_y = slot as f32 * ROW_HEIGHT;

            let row_bg = if *is_active {
                colors.active_bg
            } else {
                colors.bg
            };

            // Row background (only paint when non-default to reduce overdraw)
            if *is_active {
                sugarloaf.rect(None, 0.0, row_y, w - 0.5, ROW_HEIGHT, row_bg, 0.0, 1);
            }

            let text_color = if *is_active {
                colors.active_fg
            } else {
                colors.fg
            };

            let text_opts = DrawOpts {
                font_size: TITLE_FONT_SIZE,
                color: color_u8(text_color),
                ..DrawOpts::default()
            };

            // Max text width inside the sidebar minus padding on both sides
            let max_text_w = (w - TEXT_PADDING_X * 2.0).max(0.0);
            if max_text_w > 0.0 {
                let ui = sugarloaf.text_mut();
                let measured = ui.measure(title, &text_opts);
                // Simple right-clip: if title overflows, the GPU clips it.
                // Full ellipsis truncation can be added in a follow-up once
                // a `char_advance` helper is exposed without a Sugarloaf borrow.
                let _ = measured; // used to avoid "unused" warning
                let text_x = TEXT_PADDING_X;
                let text_y = row_y + (ROW_HEIGHT / 2.0) - (TITLE_FONT_SIZE / 2.0);
                ui.draw(text_x, text_y, title, &text_opts);
            }
        }
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SidebarColors {
    pub bg: [f32; 4],
    pub fg: [f32; 4],
    pub active_bg: [f32; 4],
    pub active_fg: [f32; 4],
}

impl Default for SidebarColors {
    fn default() -> Self {
        Self {
            bg: [0.15, 0.15, 0.15, 1.0],
            fg: [0.8, 0.8, 0.8, 1.0],
            active_bg: [0.25, 0.25, 0.25, 1.0],
            active_fg: [1.0, 1.0, 1.0, 1.0],
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_reserves_collapsed_zero_width() {
        let mut s = Sidebar::new();
        s.toggle(); // close it
        assert_eq!(s.effective_width(), SIDEBAR_WIDTH_COLLAPSED);
    }

    #[test]
    fn expanded_width_matches_config() {
        let s = Sidebar::new();
        assert_eq!(s.effective_width(), SIDEBAR_WIDTH);
    }

    #[test]
    fn toggle_twice_returns_to_open() {
        let mut s = Sidebar::new();
        s.toggle();
        s.toggle();
        assert!(s.is_open);
        assert_eq!(s.effective_width(), SIDEBAR_WIDTH);
    }
}
