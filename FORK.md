# Rio Fork — Modifications

This is a fork of [Rio terminal](https://github.com/raphamorim/rio) with Tilix-style tiling and pane management added across Phases 1–18.

---

## What's Changed

### Tiling Layout Engine (Phases 1–8)

Full recursive pane splitting backed by [Taffy](https://github.com/DioxusLabs/taffy) flexbox/grid layout.

- **Split pane right / down / auto** — divides the current pane at 50 %
- **Pane navigation** — jump by number (`Alt+1–9`), direction (`Alt+Arrow`), or cycle (`Ctrl+Tab`)
- **Keyboard resize** — `Shift+Alt+Arrow` shifts the adjacent splitter by 10 px per press
- **Splitter double-click** — redistributes all panes on that axis to equal size
- **Maximize / restore** — one pane fills the session area; `Shift+Ctrl+X` to toggle
- **Per-pane titlebars** — thin bar with title, close, maximize, sync, drag handle (shown when 2+ panes)
- **Session sidebar** — collapsible left panel listing all sessions (`F12` to toggle)

### Pane Drag-and-Drop (Phases 9–11)

- **Intra-session DnD** — drag a titlebar and drop onto another pane; quadrant (L/R/T/B) determines where it lands
- **Cross-session DnD** — drop onto a tab in the tab strip to move the pane into that session
- **Tear-off to new window** — drop outside all windows to open the pane in a new Rio window; `DetachPaneToWindow` action also available as a keybinding

### Synchronized Input (Phase 12)

- **Session-wide sync** — broadcast every keystroke to all panes in the session simultaneously
- **Per-pane exclusion** — toggle individual panes out of the sync group without disabling it globally

### Read-Only Pane (Phase 13)

- Lock a pane so keyboard input is silently discarded — useful for monitoring logs while typing elsewhere

### Titlebar Context Menu (Phase 14)

- **Right-click anywhere** in the terminal (or on a titlebar) to open the pane menu
- Entries: Find, Read-Only (live checkbox), Assistants ▶ (stub), Profiles ▶ (stub), Other ▶ (stub), Detach, Close
- Dismiss with `Escape` or click outside

### Middle-Click Close (Phase 15)

- Middle-clicking a pane titlebar closes that pane (opt-in via config)
- Optional: close the whole window when the last session is closed

### Tilix Keybinding Preset (Phase 18)

45 keybindings matching the Tilix terminal defaults, enabled with a single config line.

### Native Context Menu on Mouse Right-Click (Phase 19)

- **Linux X11**: mouse right-click spawns a borderless override-redirect popup window (rio-window) hosting the menu via Sugarloaf. Menu can extend past the parent window's edge. Dismiss on click-outside, focus-loss, or `Escape`.
- **Linux Wayland**: keeps the in-canvas Sugarloaf popover (no `xdg_popup` support in rio-window yet — tracked for a follow-up).
- **macOS / Windows**: still on the in-canvas Sugarloaf popover; native `NSMenu` / `HMENU` via the `muda` crate is wired structurally and tracked in `docs/agents/plans/2026-06-24-native-context-menu-macos-windows.md`.
- The keyboard-triggered popover (current and any future Shift+F10 / Menu-key path) always stays on the in-canvas Sugarloaf renderer.

---

## New Configuration Keys

All keys below are additions — existing Rio config keys are unchanged.

### `[navigation]`

```toml
[navigation]
use_split = true   # enable pane splitting (default: true in this fork)
```

### `[keyboard]`

```toml
[keyboard]
preset = "tilix"   # load the full Tilix keybinding set (default: "default")
```

`"default"` keeps Rio's original bindings. `"tilix"` loads 45 shortcuts — see [`docs/features/tilix-preset.md`](docs/features/tilix-preset.md) for the full table.

User-defined `[[keyboard.bindings]]` entries always take priority over the preset.

### `[pane]`

All three fields default to `false` — no behaviour change unless you opt in.

```toml
[pane]
titlebar = true                      # show per-pane titlebar when 2+ panes exist
close_on_middle_click = true         # middle-click titlebar closes that pane
close_window_with_last_session = true  # close the OS window when the last session is closed
```

---

## Recommended Config

```toml
[shell]
program = "/bin/zsh"
args = []

[navigation]
mode = "Tab"
use_split = true
clickable = true

[pane]
titlebar = true
close_on_middle_click = true
close_window_with_last_session = true

[keyboard]
preset = "tilix"

[window]
width = 1200
height = 800
opacity = 1.0

[fonts]
size = 14

[cursor]
shape = "block"
blinking = true

[scroll]
multiplier = 3.0
divider = 1.0

copy_on_select = true
confirm_before_quit = false
hide_cursor_when_typing = true
```

---

## Key Tilix Shortcuts (preset = "tilix")

| Action | Key |
|--------|-----|
| Split right | `Ctrl+Alt+R` |
| Split down | `Ctrl+Alt+D` |
| Auto split | `Ctrl+Alt+A` |
| Focus pane 1–9 | `Alt+1` – `Alt+9` |
| Focus by direction | `Alt+Arrow` |
| Cycle panes | `Ctrl+Tab` / `Ctrl+Shift+Tab` |
| Resize pane | `Shift+Alt+Arrow` |
| Maximize / restore | `Shift+Ctrl+X` |
| Close pane | `Shift+Ctrl+W` |
| New session | `Shift+Ctrl+T` |
| Close session | `Shift+Ctrl+Q` |
| Toggle sidebar | `F12` |
| Fullscreen | `F11` |
| Detach pane to window | `DetachPaneToWindow` (bind manually) |

Full table: [`docs/features/tilix-preset.md`](docs/features/tilix-preset.md)  
Feature overview: [`docs/features/tiling.md`](docs/features/tiling.md)

---

## Build

Requires Docker (see `Dockerfile` + `docker-compose.yml`).

```bash
# Build release binary — copies to ./rio on host
mise run build

# Debug build on host (requires system libs)
cargo build -p rioterm

# Run tests
mise run test-only
```
