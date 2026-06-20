//! Tilix keybinding preset for Rio terminal.
//!
//! Enable by setting `[keyboard] preset = "tilix"` in the configuration file.
//! This module provides the full Tilix-compatible keybinding table as documented
//! in `docs/features/tilix-preset.md`.

use crate::bindings::{
    Action, BindingKey, BindingMode, KeyBinding, MouseBinding, PaneDirection,
};
use rio_window::keyboard::{Key, KeyLocation, ModifiersState, NamedKey};

/// Returns the full set of Tilix-compatible key bindings.
///
/// These bindings replicate the default Tilix keyboard shortcuts for pane
/// splitting, navigation, resize, maximize, tab management, and sessions.
pub fn tilix_key_bindings() -> Vec<KeyBinding> {
    use NamedKey::*;

    let make = |trigger: BindingKey, mods: ModifiersState, action: Action| KeyBinding {
        trigger,
        mods,
        mode: BindingMode::empty(),
        notmode: BindingMode::empty(),
        action,
    };

    let char_key = |c: &str| -> BindingKey {
        BindingKey::Keycode {
            key: Key::Character(c.into()),
            location: KeyLocation::Standard,
        }
    };

    let named_key = |k: NamedKey| -> BindingKey {
        BindingKey::Keycode {
            key: Key::Named(k),
            location: KeyLocation::Standard,
        }
    };

    let ctrl_alt = ModifiersState::CONTROL | ModifiersState::ALT;
    let shift_ctrl = ModifiersState::SHIFT | ModifiersState::CONTROL;
    let shift_alt = ModifiersState::SHIFT | ModifiersState::ALT;

    vec![
        // ── Splits ──────────────────────────────────────────────────────────
        make(char_key("r"), ctrl_alt, Action::SplitRight),
        make(char_key("d"), ctrl_alt, Action::SplitDown),
        make(char_key("a"), ctrl_alt, Action::SplitAuto),
        // ── Focus pane by number (Alt+1–9, Alt+0 → pane 10) ─────────────────
        make(char_key("1"), ModifiersState::ALT, Action::FocusPaneByNumber(1)),
        make(char_key("2"), ModifiersState::ALT, Action::FocusPaneByNumber(2)),
        make(char_key("3"), ModifiersState::ALT, Action::FocusPaneByNumber(3)),
        make(char_key("4"), ModifiersState::ALT, Action::FocusPaneByNumber(4)),
        make(char_key("5"), ModifiersState::ALT, Action::FocusPaneByNumber(5)),
        make(char_key("6"), ModifiersState::ALT, Action::FocusPaneByNumber(6)),
        make(char_key("7"), ModifiersState::ALT, Action::FocusPaneByNumber(7)),
        make(char_key("8"), ModifiersState::ALT, Action::FocusPaneByNumber(8)),
        make(char_key("9"), ModifiersState::ALT, Action::FocusPaneByNumber(9)),
        make(char_key("0"), ModifiersState::ALT, Action::FocusPaneByNumber(10)),
        // ── Focus pane by direction ──────────────────────────────────────────
        make(
            named_key(ArrowUp),
            ModifiersState::ALT,
            Action::FocusPaneByDirection(PaneDirection::Up),
        ),
        make(
            named_key(ArrowDown),
            ModifiersState::ALT,
            Action::FocusPaneByDirection(PaneDirection::Down),
        ),
        make(
            named_key(ArrowLeft),
            ModifiersState::ALT,
            Action::FocusPaneByDirection(PaneDirection::Left),
        ),
        make(
            named_key(ArrowRight),
            ModifiersState::ALT,
            Action::FocusPaneByDirection(PaneDirection::Right),
        ),
        // ── Cycle panes ──────────────────────────────────────────────────────
        make(
            named_key(Tab),
            ModifiersState::CONTROL,
            Action::SelectNextSplit,
        ),
        make(
            named_key(Tab),
            ModifiersState::CONTROL | ModifiersState::SHIFT,
            Action::SelectPrevSplit,
        ),
        // ── Resize pane ──────────────────────────────────────────────────────
        make(
            named_key(ArrowUp),
            shift_alt,
            Action::ResizePaneInDirection(PaneDirection::Up),
        ),
        make(
            named_key(ArrowDown),
            shift_alt,
            Action::ResizePaneInDirection(PaneDirection::Down),
        ),
        make(
            named_key(ArrowLeft),
            shift_alt,
            Action::ResizePaneInDirection(PaneDirection::Left),
        ),
        make(
            named_key(ArrowRight),
            shift_alt,
            Action::ResizePaneInDirection(PaneDirection::Right),
        ),
        // ── Pane maximize / close ────────────────────────────────────────────
        make(char_key("x"), shift_ctrl, Action::TogglePaneMaximized),
        make(char_key("w"), shift_ctrl, Action::CloseCurrentSplitOrTab),
        // ── Tab management ───────────────────────────────────────────────────
        make(char_key("q"), shift_ctrl, Action::TabCloseCurrent),
        make(char_key("t"), shift_ctrl, Action::TabCreateNew),
        // ── Select tab by number (Ctrl+Alt+1–9, Ctrl+Alt+0 → tab 10) ────────
        make(char_key("1"), ctrl_alt, Action::SelectTab(0)),
        make(char_key("2"), ctrl_alt, Action::SelectTab(1)),
        make(char_key("3"), ctrl_alt, Action::SelectTab(2)),
        make(char_key("4"), ctrl_alt, Action::SelectTab(3)),
        make(char_key("5"), ctrl_alt, Action::SelectTab(4)),
        make(char_key("6"), ctrl_alt, Action::SelectTab(5)),
        make(char_key("7"), ctrl_alt, Action::SelectTab(6)),
        make(char_key("8"), ctrl_alt, Action::SelectTab(7)),
        make(char_key("9"), ctrl_alt, Action::SelectTab(8)),
        make(char_key("0"), ctrl_alt, Action::SelectTab(9)),
        // ── Next / prev tab ─────────────────────────────────────────────────
        make(named_key(PageDown), ModifiersState::CONTROL, Action::SelectNextTab),
        make(named_key(PageUp), ModifiersState::CONTROL, Action::SelectPrevTab),
        // ── Move tab ────────────────────────────────────────────────────────
        make(
            named_key(PageDown),
            shift_ctrl,
            Action::MoveCurrentTabToNext,
        ),
        make(
            named_key(PageUp),
            shift_ctrl,
            Action::MoveCurrentTabToPrev,
        ),
        // ── Session sidebar / save / load ────────────────────────────────────
        make(
            named_key(F12),
            ModifiersState::empty(),
            Action::ToggleSessionSidebar,
        ),
        make(char_key("s"), shift_ctrl, Action::SaveSessionLayout),
        make(char_key("o"), shift_ctrl, Action::OpenSessionLayout),
        // ── Fullscreen ───────────────────────────────────────────────────────
        make(
            named_key(F11),
            ModifiersState::empty(),
            Action::ToggleFullscreen,
        ),
    ]
}

/// Returns the Tilix-compatible mouse bindings.
///
/// Tilix does not define distinct mouse shortcuts beyond the standard terminal
/// defaults, so this returns an empty list.  The default mouse bindings from
/// `default_mouse_bindings()` remain active.
#[allow(dead_code)]
pub fn tilix_mouse_bindings() -> Vec<MouseBinding> {
    // Tilix has no non-default mouse bindings; use default_mouse_bindings()
    // alongside this preset.
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Expected number of Tilix key bindings defined in this preset.
    /// Update this constant whenever a new entry is added to `tilix_key_bindings()`.
    const EXPECTED_COUNT: usize = 45;

    #[test]
    fn preset_contains_every_spec_row() {
        let bindings = tilix_key_bindings();
        assert_eq!(
            bindings.len(),
            EXPECTED_COUNT,
            "tilix_key_bindings() returned {} entries, expected {}",
            bindings.len(),
            EXPECTED_COUNT
        );
    }

    #[test]
    fn split_right_is_ctrl_alt_r() {
        let bindings = tilix_key_bindings();
        let found = bindings.iter().any(|b| {
            b.action == Action::SplitRight
                && b.mods
                    == ModifiersState::CONTROL | ModifiersState::ALT
                && matches!(
                    &b.trigger,
                    BindingKey::Keycode { key: rio_window::keyboard::Key::Character(c), .. }
                    if c.as_str() == "r"
                )
        });
        assert!(found, "Ctrl+Alt+R → SplitRight binding not found");
    }

    #[test]
    fn alt_1_focuses_pane_1() {
        let bindings = tilix_key_bindings();
        let found = bindings.iter().any(|b| {
            b.action == Action::FocusPaneByNumber(1)
                && b.mods == ModifiersState::ALT
                && matches!(
                    &b.trigger,
                    BindingKey::Keycode { key: rio_window::keyboard::Key::Character(c), .. }
                    if c.as_str() == "1"
                )
        });
        assert!(found, "Alt+1 → FocusPaneByNumber(1) binding not found");
    }

    #[test]
    fn alt_up_focuses_pane_up() {
        let bindings = tilix_key_bindings();
        let found = bindings.iter().any(|b| {
            b.action == Action::FocusPaneByDirection(PaneDirection::Up)
                && b.mods == ModifiersState::ALT
                && matches!(
                    &b.trigger,
                    BindingKey::Keycode {
                        key: rio_window::keyboard::Key::Named(rio_window::keyboard::NamedKey::ArrowUp),
                        ..
                    }
                )
        });
        assert!(found, "Alt+Up → FocusPaneByDirection(Up) binding not found");
    }

    #[test]
    fn f11_is_toggle_fullscreen() {
        let bindings = tilix_key_bindings();
        let found = bindings.iter().any(|b| {
            b.action == Action::ToggleFullscreen
                && b.mods == ModifiersState::empty()
                && matches!(
                    &b.trigger,
                    BindingKey::Keycode {
                        key: rio_window::keyboard::Key::Named(rio_window::keyboard::NamedKey::F11),
                        ..
                    }
                )
        });
        assert!(found, "F11 → ToggleFullscreen binding not found");
    }

    #[test]
    fn tilix_mouse_bindings_is_empty() {
        assert!(tilix_mouse_bindings().is_empty());
    }
}
