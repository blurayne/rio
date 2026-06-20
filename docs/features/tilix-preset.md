# Tilix Keybinding Preset

Rio ships a built-in keybinding preset that mirrors the default keyboard
shortcuts of the [Tilix](https://gnunn1.github.io/tilix-web/) terminal
emulator.  Enabling it gives you familiar shortcuts for pane splits,
navigation, resize, tab management, and sessions without hand-crafting every
binding.

## Enabling the preset

Add the following to your `rio.toml` (or `config.toml`) configuration file:

```toml
[keyboard]
preset = "tilix"
```

Any bindings you define under `[bindings]` are merged on top of the preset and
always take precedence.

## Full binding table

| Key combination | Action |
|---|---|
| Ctrl+Alt+R | Split current pane to the right (`SplitRight`) |
| Ctrl+Alt+D | Split current pane downward (`SplitDown`) |
| Ctrl+Alt+A | Auto-orient split (`SplitAuto`) |
| Alt+1 | Focus pane 1 (`FocusPaneByNumber(1)`) |
| Alt+2 | Focus pane 2 (`FocusPaneByNumber(2)`) |
| Alt+3 | Focus pane 3 (`FocusPaneByNumber(3)`) |
| Alt+4 | Focus pane 4 (`FocusPaneByNumber(4)`) |
| Alt+5 | Focus pane 5 (`FocusPaneByNumber(5)`) |
| Alt+6 | Focus pane 6 (`FocusPaneByNumber(6)`) |
| Alt+7 | Focus pane 7 (`FocusPaneByNumber(7)`) |
| Alt+8 | Focus pane 8 (`FocusPaneByNumber(8)`) |
| Alt+9 | Focus pane 9 (`FocusPaneByNumber(9)`) |
| Alt+0 | Focus pane 10 (`FocusPaneByNumber(10)`) |
| Alt+Up | Focus pane above (`FocusPaneByDirection(Up)`) |
| Alt+Down | Focus pane below (`FocusPaneByDirection(Down)`) |
| Alt+Left | Focus pane to the left (`FocusPaneByDirection(Left)`) |
| Alt+Right | Focus pane to the right (`FocusPaneByDirection(Right)`) |
| Ctrl+Tab | Select next pane (`SelectNextSplit`) |
| Ctrl+Shift+Tab | Select previous pane (`SelectPrevSplit`) |
| Shift+Alt+Up | Resize pane upward (`ResizePaneInDirection(Up)`) |
| Shift+Alt+Down | Resize pane downward (`ResizePaneInDirection(Down)`) |
| Shift+Alt+Left | Resize pane leftward (`ResizePaneInDirection(Left)`) |
| Shift+Alt+Right | Resize pane rightward (`ResizePaneInDirection(Right)`) |
| Shift+Ctrl+X | Toggle pane maximized (`TogglePaneMaximized`) |
| Shift+Ctrl+W | Close current pane or tab (`CloseCurrentSplitOrTab`) |
| Shift+Ctrl+Q | Close current tab (`TabCloseCurrent`) |
| Shift+Ctrl+T | Open new tab (`TabCreateNew`) |
| Ctrl+Alt+1 | Switch to tab 1 (`SelectTab(0)`) |
| Ctrl+Alt+2 | Switch to tab 2 (`SelectTab(1)`) |
| Ctrl+Alt+3 | Switch to tab 3 (`SelectTab(2)`) |
| Ctrl+Alt+4 | Switch to tab 4 (`SelectTab(3)`) |
| Ctrl+Alt+5 | Switch to tab 5 (`SelectTab(4)`) |
| Ctrl+Alt+6 | Switch to tab 6 (`SelectTab(5)`) |
| Ctrl+Alt+7 | Switch to tab 7 (`SelectTab(6)`) |
| Ctrl+Alt+8 | Switch to tab 8 (`SelectTab(7)`) |
| Ctrl+Alt+9 | Switch to tab 9 (`SelectTab(8)`) |
| Ctrl+Alt+0 | Switch to tab 10 (`SelectTab(9)`) |
| Ctrl+PageDown | Select next tab (`SelectNextTab`) |
| Ctrl+PageUp | Select previous tab (`SelectPrevTab`) |
| Ctrl+Shift+PageDown | Move current tab to next slot (`MoveCurrentTabToNext`) |
| Ctrl+Shift+PageUp | Move current tab to previous slot (`MoveCurrentTabToPrev`) |
| F12 | Toggle session sidebar (`ToggleSessionSidebar`) |
| Shift+Ctrl+S | Save session layout (`SaveSessionLayout`) |
| Shift+Ctrl+O | Open session layout (`OpenSessionLayout`) |
| F11 | Toggle fullscreen (`ToggleFullscreen`) |

## Notes

- Pane numbers are assigned in visual creation order (top-to-bottom,
  left-to-right).
- `SplitAuto` chooses horizontal or vertical orientation based on the current
  pane's aspect ratio (wider → right split, taller → down split).
- The preset does not override default copy/paste or scrolling shortcuts; those
  remain as configured by your platform defaults or explicit `[bindings]`
  entries.
