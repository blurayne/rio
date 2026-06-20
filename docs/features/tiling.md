# Tiling Features

Rio supports a Tilix-inspired tiling model that lets you divide a session into
multiple panes, navigate between them with the keyboard, and manage them with
drag-and-drop.

## Pane splits

| Action | Description |
|---|---|
| `SplitRight` | Add a new pane to the right of the current one |
| `SplitDown` | Add a new pane below the current one |
| `SplitAuto` | Split in the direction that best fits the pane's aspect ratio |

Panes are arranged in a recursive Taffy flex tree so any nesting depth is
supported.

## Pane navigation

| Action | Description |
|---|---|
| `FocusPaneByNumber(n)` | Jump directly to the Nth pane (visual creation order) |
| `FocusPaneByDirection(dir)` | Focus the geometrically nearest pane in Up/Down/Left/Right |
| `SelectNextSplit` | Cycle to the next pane |
| `SelectPrevSplit` | Cycle to the previous pane |

## Pane resize

| Action | Description |
|---|---|
| `ResizePaneInDirection(dir)` | Move the shared border in the given direction |
| `DistributePanesEvenly` | Reset all panes to equal sizes within the session |

Drag the visual divider bar with the mouse for freeform resize.

## Pane maximize

`TogglePaneMaximized` expands the current pane to fill the entire session area
while keeping the other panes in memory.  Toggling again restores the original
layout.

## Drag-and-drop

Panes can be dragged by their title bar and dropped onto quadrant targets that
appear over the destination pane.  Dropping on the edge targets reorders the
pane; dropping in the centre swaps the two panes.

### Cross-session drag

Dragging a pane onto a different session tab tears it out of its original
session and inserts it into the target session.

### Tear-off to window

`DetachPaneToWindow` promotes the current pane into an independent top-level
window.

## Synchronized input

| Action | Description |
|---|---|
| `ToggleSyncInputSession` | All panes in the session receive each keystroke simultaneously |
| `TogglePaneSyncInputOverride` | Opt a single pane out of session-wide sync |

## Read-only pane

`TogglePaneReadOnly` blocks PTY writes for the current pane.  The pane still
displays output but ignores keyboard input.

## Session sidebar

`ToggleSessionSidebar` shows or hides a collapsible sidebar that lists all open
sessions, enabling quick switching without the tab strip.

## Session save and restore

| Action | Description |
|---|---|
| `SaveSessionLayout` | Serialize the current pane layout to a JSON file |
| `OpenSessionLayout` | Deserialize a previously saved layout and apply it |

## Tilix keybinding preset

All tiling actions are available as individually configurable bindings via the
`[bindings]` section of your configuration file.  For a batteries-included
experience that mirrors Tilix defaults, set:

```toml
[keyboard]
preset = "tilix"
```

See [tilix-preset.md](tilix-preset.md) for the full binding table.
