// Copyright (c) 2023-present, Raphael Amorim.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use rio_backend::sugarloaf::text::DrawOpts;
use rio_backend::sugarloaf::Sugarloaf;

// Layout constants (logical pixels)
const MENU_WIDTH: f32 = 200.0;
const ENTRY_HEIGHT: f32 = 22.0;
const PADDING_LEFT: f32 = 8.0;
const PADDING_TOP: f32 = 4.0;
const FONT_SIZE: f32 = 13.0;
const SEPARATOR_HEIGHT: f32 = 1.0;

// Colors
const BG_COLOR: [f32; 4] = [0.13, 0.13, 0.15, 0.97];
const BORDER_COLOR: [f32; 4] = [0.3, 0.3, 0.35, 1.0];
const HOVER_COLOR: [f32; 4] = [0.25, 0.4, 0.7, 0.8];
const TEXT_COLOR: [f32; 4] = [0.9, 0.9, 0.9, 1.0];
const SEPARATOR_COLOR: [f32; 4] = [0.35, 0.35, 0.4, 1.0];

// Render order — above normal pane content but below high-priority overlays
const MENU_ORDER: u8 = 15;
const DEPTH_BG: f32 = 0.05;
const DEPTH_BORDER: f32 = 0.06;
const DEPTH_HOVER: f32 = 0.07;
const DEPTH_SEP: f32 = 0.08;
#[allow(dead_code)]
const DEPTH_TEXT: f32 = 0.09;

#[inline]
fn color_u8(c: [f32; 4]) -> [u8; 4] {
    [
        (c[0].clamp(0.0, 1.0) * 255.0) as u8,
        (c[1].clamp(0.0, 1.0) * 255.0) as u8,
        (c[2].clamp(0.0, 1.0) * 255.0) as u8,
        (c[3].clamp(0.0, 1.0) * 255.0) as u8,
    ]
}

/// Actions that can be triggered from the pane titlebar popover menu.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    SearchForward,
    ToggleReadOnly,
    DetachPaneToWindow,
    CloseCurrentSplitOrTab,
    Dismiss,
    // Stub entries (no-op — submenu items not yet implemented):
    AssistantsPasswordManager,
    AssistantsBookmarks,
    ProfilesEditProfile,
    OtherShowFileBrowser,
    OtherSaveOutput,
    OtherReset,
    OtherResetAndClear,
    OtherEncoding,
    OtherLayoutOptions,
    OtherMonitorSilence,
}

/// An individual menu row — either a separator or a clickable label.
#[derive(Debug, Clone)]
pub struct MenuEntry {
    pub label: String,
    pub is_separator: bool,
    #[allow(dead_code)]
    pub action: Option<MenuAction>,
}

impl MenuEntry {
    fn item(label: impl Into<String>, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            is_separator: false,
            action: Some(action),
        }
    }

    fn separator() -> Self {
        Self {
            label: String::new(),
            is_separator: true,
            action: None,
        }
    }
}

/// Pane titlebar popover menu widget.
pub struct PaneTitlebarMenu {
    pub enabled: bool,
    /// Top-left corner in logical (unscaled) pixels.
    pub x: f32,
    pub y: f32,
    /// Current read-only state of the pane — drives the checkbox label.
    pub read_only: bool,
    hovered_index: Option<usize>,
}

#[allow(dead_code)]
impl PaneTitlebarMenu {
    pub fn new() -> Self {
        Self {
            enabled: false,
            x: 0.0,
            y: 0.0,
            read_only: false,
            hovered_index: None,
        }
    }

    /// Open the menu at the given logical-pixel position.
    pub fn open(&mut self, x: f32, y: f32, read_only: bool) {
        self.enabled = true;
        self.x = x;
        self.y = y;
        self.read_only = read_only;
        self.hovered_index = None;
    }

    /// Close / dismiss the menu.
    pub fn close(&mut self) {
        self.enabled = false;
        self.hovered_index = None;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Build the current entry list. Re-generated on each call so the
    /// read-only checkbox label is always up to date.
    pub fn entries(&self) -> Vec<MenuEntry> {
        let read_only_label = if self.read_only {
            "Read-Only ☑"
        } else {
            "Read-Only ☐"
        };
        vec![
            MenuEntry::item("Find…", MenuAction::SearchForward),
            MenuEntry::item(read_only_label, MenuAction::ToggleReadOnly),
            MenuEntry::separator(),
            MenuEntry::item("Assistants ▶", MenuAction::AssistantsPasswordManager),
            MenuEntry::item("Profiles ▶", MenuAction::ProfilesEditProfile),
            MenuEntry::separator(),
            MenuEntry::item("Other ▶", MenuAction::OtherShowFileBrowser),
            MenuEntry::separator(),
            MenuEntry::item("Detach", MenuAction::DetachPaneToWindow),
            MenuEntry::separator(),
            MenuEntry::item("Close", MenuAction::CloseCurrentSplitOrTab),
        ]
    }

    /// Total height of the menu in logical pixels.
    fn total_height(&self) -> f32 {
        let entries = self.entries();
        let mut h = PADDING_TOP * 2.0;
        for e in &entries {
            if e.is_separator {
                h += SEPARATOR_HEIGHT + 4.0; // 2px top + 2px bottom padding
            } else {
                h += ENTRY_HEIGHT;
            }
        }
        h
    }

    /// Update hovered row based on current logical-pixel mouse position.
    pub fn hover(&mut self, x: f32, y: f32) {
        if !self.enabled {
            return;
        }
        self.hovered_index = self.row_at(x, y);
    }

    /// Hit-test a logical-pixel mouse click.
    ///
    /// Returns `Some(action)` when a clickable row was hit.
    /// Returns `Some(Dismiss)` when the click is outside the menu
    /// (so the caller can close it).
    /// Returns `None` when clicking inside the menu on a non-actionable
    /// area (e.g., a separator).
    pub fn hit_test(&self, x: f32, y: f32) -> Option<MenuAction> {
        if !self.enabled {
            return None;
        }

        let total_h = self.total_height();
        // Outside menu bounds → dismiss
        if x < self.x
            || x > self.x + MENU_WIDTH
            || y < self.y
            || y > self.y + total_h
        {
            return Some(MenuAction::Dismiss);
        }

        if let Some(idx) = self.row_at(x, y) {
            let entries = self.entries();
            if let Some(entry) = entries.get(idx) {
                return entry.action.clone();
            }
        }
        // Clicked inside menu but on a separator or empty area — no action
        None
    }

    /// Return the entry index (into `self.entries()`) that the logical
    /// position falls on, or `None` if outside the item area.
    fn row_at(&self, x: f32, y: f32) -> Option<usize> {
        if x < self.x || x > self.x + MENU_WIDTH {
            return None;
        }
        let entries = self.entries();
        let mut cursor_y = self.y + PADDING_TOP;
        for (i, entry) in entries.iter().enumerate() {
            if entry.is_separator {
                let sep_h = SEPARATOR_HEIGHT + 4.0;
                cursor_y += sep_h;
            } else {
                if y >= cursor_y && y < cursor_y + ENTRY_HEIGHT {
                    return Some(i);
                }
                cursor_y += ENTRY_HEIGHT;
            }
        }
        None
    }

    /// Render the menu into the sugarloaf.
    ///
    /// `scale` is the HiDPI scale factor — sugarloaf accepts logical
    /// coordinates, so we don't multiply by it here; it's kept for
    /// potential future sub-pixel tweaks.
    pub fn render(&self, sugarloaf: &mut Sugarloaf, _scale: f32) {
        if !self.enabled {
            return;
        }

        let total_h = self.total_height();

        // Background fill
        sugarloaf.rect(
            None,
            self.x,
            self.y,
            MENU_WIDTH,
            total_h,
            BG_COLOR,
            DEPTH_BG,
            MENU_ORDER,
        );

        // Border (1px right and bottom edges via an outer/inner rect trick)
        // Left border
        sugarloaf.rect(
            None,
            self.x,
            self.y,
            1.0,
            total_h,
            BORDER_COLOR,
            DEPTH_BORDER,
            MENU_ORDER,
        );
        // Right border
        sugarloaf.rect(
            None,
            self.x + MENU_WIDTH - 1.0,
            self.y,
            1.0,
            total_h,
            BORDER_COLOR,
            DEPTH_BORDER,
            MENU_ORDER,
        );
        // Top border
        sugarloaf.rect(
            None,
            self.x,
            self.y,
            MENU_WIDTH,
            1.0,
            BORDER_COLOR,
            DEPTH_BORDER,
            MENU_ORDER,
        );
        // Bottom border
        sugarloaf.rect(
            None,
            self.x,
            self.y + total_h - 1.0,
            MENU_WIDTH,
            1.0,
            BORDER_COLOR,
            DEPTH_BORDER,
            MENU_ORDER,
        );

        let entries = self.entries();
        let text_opts = DrawOpts {
            font_size: FONT_SIZE,
            color: color_u8(TEXT_COLOR),
            ..DrawOpts::default()
        };

        let mut cursor_y = self.y + PADDING_TOP;

        for (i, entry) in entries.iter().enumerate() {
            if entry.is_separator {
                let sep_top = cursor_y + 2.0;
                sugarloaf.rect(
                    None,
                    self.x + PADDING_LEFT,
                    sep_top,
                    MENU_WIDTH - PADDING_LEFT * 2.0,
                    SEPARATOR_HEIGHT,
                    SEPARATOR_COLOR,
                    DEPTH_SEP,
                    MENU_ORDER,
                );
                cursor_y += SEPARATOR_HEIGHT + 4.0;
            } else {
                // Hover highlight
                if self.hovered_index == Some(i) {
                    sugarloaf.rect(
                        None,
                        self.x + 1.0,
                        cursor_y,
                        MENU_WIDTH - 2.0,
                        ENTRY_HEIGHT,
                        HOVER_COLOR,
                        DEPTH_HOVER,
                        MENU_ORDER,
                    );
                }

                let text_y = cursor_y + (ENTRY_HEIGHT - FONT_SIZE) / 2.0;
                sugarloaf.text_mut().draw(
                    self.x + PADDING_LEFT,
                    text_y,
                    &entry.label,
                    &text_opts,
                );

                cursor_y += ENTRY_HEIGHT;
            }
        }
    }
}

impl Default for PaneTitlebarMenu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_layout_contains_all_required_entries() {
        let menu = PaneTitlebarMenu::new();
        assert!(menu.entries().iter().any(|e| e.label.contains("Find")));
        assert!(menu.entries().iter().any(|e| e.label.contains("Read")));
        assert!(menu.entries().iter().any(|e| e.label.contains("Detach")));
        assert!(menu.entries().iter().any(|e| e.label.contains("Close")));
    }

    #[test]
    fn open_sets_enabled_and_position() {
        let mut menu = PaneTitlebarMenu::new();
        assert!(!menu.is_enabled());
        menu.open(50.0, 100.0, false);
        assert!(menu.is_enabled());
        assert_eq!(menu.x, 50.0);
        assert_eq!(menu.y, 100.0);
        assert!(!menu.read_only);
    }

    #[test]
    fn close_disables_menu() {
        let mut menu = PaneTitlebarMenu::new();
        menu.open(0.0, 0.0, false);
        menu.close();
        assert!(!menu.is_enabled());
    }

    #[test]
    fn read_only_checkbox_label_reflects_state() {
        let mut menu = PaneTitlebarMenu::new();
        menu.open(0.0, 0.0, false);
        let unchecked = menu
            .entries()
            .iter()
            .any(|e| e.label.contains("☐") && e.label.contains("Read"));
        assert!(unchecked);

        menu.read_only = true;
        let checked = menu
            .entries()
            .iter()
            .any(|e| e.label.contains("☑") && e.label.contains("Read"));
        assert!(checked);
    }

    #[test]
    fn hit_test_outside_returns_dismiss() {
        let mut menu = PaneTitlebarMenu::new();
        menu.open(100.0, 100.0, false);
        assert_eq!(menu.hit_test(0.0, 0.0), Some(MenuAction::Dismiss));
    }

    #[test]
    fn hit_test_disabled_returns_none() {
        let menu = PaneTitlebarMenu::new();
        // Not enabled: hit_test always returns None.
        assert_eq!(menu.hit_test(0.0, 0.0), None);
    }

    #[test]
    fn hit_test_first_item_returns_search_forward() {
        let mut menu = PaneTitlebarMenu::new();
        menu.open(0.0, 0.0, false);
        // First item starts at y = PADDING_TOP, height = ENTRY_HEIGHT
        let hit_y = PADDING_TOP + ENTRY_HEIGHT / 2.0;
        let result = menu.hit_test(PADDING_LEFT, hit_y);
        assert_eq!(result, Some(MenuAction::SearchForward));
    }

    #[test]
    fn hover_updates_hovered_index() {
        let mut menu = PaneTitlebarMenu::new();
        menu.open(0.0, 0.0, false);
        assert_eq!(menu.hovered_index, None);
        let hit_y = PADDING_TOP + ENTRY_HEIGHT / 2.0;
        menu.hover(PADDING_LEFT, hit_y);
        assert_eq!(menu.hovered_index, Some(0));
    }

    #[test]
    fn separator_entries_have_no_action() {
        let menu = PaneTitlebarMenu::new();
        let seps: Vec<_> = menu.entries().into_iter().filter(|e| e.is_separator).collect();
        assert!(!seps.is_empty());
        for sep in seps {
            assert!(sep.action.is_none());
        }
    }

    #[test]
    fn total_height_is_positive() {
        let menu = PaneTitlebarMenu::new();
        assert!(menu.total_height() > 0.0);
    }
}
