---
date: 2026-06-19T21:47:49+00:00
git_commit: e1946a7b98a5a5a4074f384437f0256abf1df75b
branch: main
topic: "Tilix-style tiling for Rio terminal"
tags: [plan, tiling, panes, sessions, drag-and-drop, sugarloaf, layout, bindings]
status: draft
---

# Tilix-style Tiling Implementation Plan

## Overview

Port the full Tilix tiling behavioural specification (Sections 1–11) to Rio. Rio already provides ~40% of the surface — split panes via a Taffy-driven `ContextGrid`, splitter-drag, divider keys, tab strip with drag-reorder, full-screen toggle, and multi-window plumbing. This plan adds the missing tiling primitives (auto-orient split, spatial focus, keyboard resize, pane maximize, distribute-evenly), a per-pane titlebar UI (opt-in), inter-pane drag-and-drop with quadrant drop targets, cross-session drag, tear-off-to-new-window, synchronized input, per-pane read-only mode, session JSON save/restore, sidebar navigation mode, and a complete Tilix keybinding preset.

## Current State Analysis

### Already present (do not re-implement)

| Tilix US | Rio symbol | File:line |
|---|---|---|
| US-1.1 SplitRight | `Action::SplitRight` → `Screen::split_right` → `ContextGrid::split_right` | `bindings/mod.rs:476`, `screen/mod.rs:1404`, `layout/mod.rs:1320` |
| US-1.2 SplitDown | `Action::SplitDown` → `Screen::split_down` → `ContextGrid::split_down` | `bindings/mod.rs:479`, `screen/mod.rs:1412`, `layout/mod.rs:1335` |
| US-1.4 nested splits | `ContextGrid` is a `TaffyTree` of `Flex` containers, recursive by construction | `layout/mod.rs:108` |
| US-2.3 cycle next/prev pane | `Action::SelectNextSplit` / `SelectPrevSplit` using `get_ordered_keys` (top-bottom, left-right) | `bindings/mod.rs:482`, `layout/mod.rs:950` |
| US-3.2 splitter drag | `ResizeState` + `find_border_at_position` + `resize_border` | `layout/mod.rs:39`, `layout/mod.rs:348`, `layout/mod.rs:397` |
| US-7.1 close pane | `Action::CloseCurrentSplitOrTab` → `ContextGrid::remove_current` (collapses single-child containers) | `bindings/mod.rs:437`, `layout/mod.rs:1240` |
| US-8.1 new session/tab | `Action::TabCreateNew` | `bindings/mod.rs:420` |
| US-8.2 select tab by number | `Action::SelectTab(usize)` | `bindings/mod.rs:465` |
| US-8.3 next/prev tab | `Action::SelectNextTab` / `SelectPrevTab` | `bindings/mod.rs:429` |
| US-8.4 reorder tab | `Action::MoveCurrentTabToPrev` / `ToNext` + Island tab drag | `bindings/mod.rs:422`, `renderer/island.rs:266` |
| US-10.1 full-screen | `Action::ToggleFullscreen` | `bindings/mod.rs:444` |
| Multi-window plumbing | `Router::create_window` + `RioEvent::CreateWindow` | `router/mod.rs:501`, `application.rs:752` |
| Spatial neighbour math | `find_horizontal_neighbors` / `find_vertical_neighbors` | `layout/mod.rs:777`, `layout/mod.rs:837` |
| Sugarloaf primitives | `Sugarloaf::rect`, `quad`, `line`, `polygon`, `text_mut` (z-ordered) | `sugarloaf/src/sugarloaf.rs:603+` |
| File-drop event | `WindowEvent::DroppedFile` | `application.rs:1797` |

### Missing (this plan covers)

| Tilix US | What's needed |
|---|---|
| US-1.3 | `Action::SplitAuto` — orient by aspect ratio of current pane |
| US-2.1 | `Action::FocusPaneByNumber(u8)` — creation-order index, persistent across removals |
| US-2.2 | `Action::FocusPaneByDirection(Dir)` — geometric nearest neighbour |
| US-3.1 | `Action::ResizePaneInDirection(Dir)` — 10 px × scale per press |
| US-3.3 | Double-click splitter → distribute panes evenly along axis (and `Action::DistributePanesEvenly`) |
| US-4 | `Action::TogglePaneMaximized` + grid-level `maximized: Option<NodeId>` + swap-while-maximized |
| US-5 | Per-pane drag state machine, thumbnail draw, quadrant drop-target overlay, cross-session move, tear-off |
| US-6 | Opt-in per-pane titlebar renderer (close, maximize, menu, sync-input icon, drag handle), hit-test |
| US-7.2 / 7.3 | `pane.close_window_with_last_session` config + restore-before-close for maximized |
| US-8.5 | `NavigationMode::Sidebar` + collapsible left-side sessions panel + `Action::ToggleSessionSidebar` |
| US-8.6 | `Action::SaveSessionLayout` / `OpenSessionLayout` — Rio-native JSON (not Tilix-compatible) |
| US-9 | Session-level + per-pane sync-input + titlebar icon state |
| US-11 | `keyboard.preset = "tilix"` — full default-binding alternative |
| Per-pane read-only | `Action::TogglePaneReadOnly` — block PTY writes for the pane |

## Desired End State

A Rio user can:

1. Run with stock keybindings unchanged (Rio defaults preserved).
2. Set `keyboard.preset = "tilix"` to switch to the full Tilix shortcut set documented in spec §11.
3. Set `pane.titlebar = true` to opt into per-pane titlebars enabling drag-and-drop, click-maximize, click-close, menu, and sync-input toggle.
4. Use all spec keyboard shortcuts (split, focus, resize, maximize, distribute, sync, etc.) regardless of titlebar setting (titlebar is only needed for mouse-based actions).
5. Drag a pane's titlebar to:
   - rearrange within the current session via 4-quadrant drop targets,
   - move it to a different session,
   - tear it off into a new window.
6. Toggle synchronized input session-wide and opt individual panes out via the titlebar icon.
7. Save/restore session layouts to/from a Rio-native JSON file.
8. Use sidebar navigation mode (`F12` toggle) as an alternative to the tab strip.

## What We're NOT Doing

- Importing Tilix's exact JSON layout format (Rio-native only — confirmed with user).
- Changing existing Rio default keybindings (only adding the `tilix` preset; `default` preset unchanged).
- Touching the macOS NativeTab mode (the new sidebar mode is additive).
- Implementing the Tilix "Profiles" submenu beyond a stub menu item (Rio's profile model differs; full profile UI is out of scope).
- Implementing the Tilix "Assistants" (password manager, bookmarks) submenu (out of scope, stub label only).
- Adding Tilix's "Encoding selector", "Monitor Silence", "Save Output", "Reset and Clear" submenu entries (stub labels only; behaviour out of scope).
- Backward-compatibility with any previously-experimental Rio pane numbering schemes.

## UI Mockups

### Per-pane titlebar (opt-in, US-6)

```
┌──────────────────────────────────────────────────────────────┐
│ ⌨  zsh — ~/projects/rio                  ⛶  ⋯  ✕ │  ← titlebar
├──────────────────────────────────────────────────────────────┤
│ $ cargo build                                                │
│ ...                                                          │
│                                                              │
└──────────────────────────────────────────────────────────────┘
   ⌨ = sync-input icon (greyed when off)
   ⛶ = maximize / restore button
   ⋯ = popover menu trigger
   ✕ = close button
   title area also serves as drag handle (entire bar grabbable)
```

### Quadrant drop-target highlight (US-5.2)

```
┌──────────────────────────┐
│■■■■■■■■■■■■■■■■■■■■■■■■■■│  ← top edge highlight (4 px)
│  ╲                    ╱  │     when cursor is in top triangle
│    ╲                ╱    │
│      ╲            ╱      │
│        ╲        ╱        │
│  L       ╲    ╱       R  │
│            ╳             │
│          ╱    ╲          │
│        ╱        ╲        │
│      ╱            ╲      │
│    ╱                ╲    │
│  ╱                    ╲  │
└──────────────────────────┘
   diagonals split the pane into 4 triangles (top / right / bottom / left).
   The triangle under the cursor gets its outer edge highlighted at 4 px.
```

### Sidebar mode (US-8.5)

```
┌─────────────────────────────────────────────────────────────┐
│ ◀                                                           │
├─────────┬───────────────────────────────────────────────────┤
│ Sess A *│ ┌──────────────┬──────────────┐                   │
│ Sess B  │ │   pane 1     │   pane 2     │                   │
│ Sess C  │ │              │              │                   │
│ Sess D  │ ├──────────────┴──────────────┤                   │
│         │ │           pane 3            │                   │
│         │ └─────────────────────────────┘                   │
│         │                                                   │
│         │                                                   │
└─────────┴───────────────────────────────────────────────────┘
   ◀ collapses the sidebar; F12 toggles. Drag-reorder sessions
   inside the list (left-mouse drag on session row).
```

## Architecture and Code Reuse

### High-level mapping (Tilix → Rio terminology)

| Tilix | Rio | Why |
|---|---|---|
| Window (`AppWindow`) | `Route` in `Router::routes` | both are 1 OS window |
| Session | `ContextGrid` (one entry in `ContextManager.contexts`) | both are a named pane-tree shown as a tab/sidebar entry |
| Pane | `Context` (one leaf in the `ContextGrid` Taffy tree) | both are a single terminal |
| Titlebar | NEW: `PaneTitlebar` widget rendered above each Context | did not exist |
| Splitter | Taffy gap between flex children + `PanelBorder` hit-test | already present |
| Maximized pane | NEW: `ContextGrid::maximized: Option<NodeId>` | did not exist |

### Code reuse highlights

- **`ContextGrid` / Taffy tree** (`layout/mod.rs:108`) is the canonical pane tree. All split/resize/focus/close logic already routes through it. The plan extends it; it does not replace it.
- **`get_ordered_keys`** (`layout/mod.rs:950`) gives visual-order traversal — re-use for distribute-evenly walk and for sequential pane numbering.
- **`find_horizontal_neighbors` / `find_vertical_neighbors`** (`layout/mod.rs:777`, `:837`) already detect adjacent panes — re-use for `FocusPaneByDirection` and `ResizePaneInDirection`.
- **`Island` tab strip + `TabDrag`** (`renderer/island.rs:64`, `:266`) is the only DnD precedent in Rio — re-use the drag-arm pattern for pane drags.
- **`Sugarloaf` primitives** (`sugarloaf.rs:603+`) — `rect()`, `quad()`, `line()`, `polygon()` cover every overlay we need (quadrant highlight, thumbnail, drag preview).
- **`Router::create_window`** (`router/mod.rs:501`) is the entry point for new windows — re-use for tear-off.
- **`WindowEvent::DroppedFile`** (`application.rs:1797`) shows how to handle drag end at the OS boundary — re-use mental model.

### Affected files (tree view)

```
rio-backend/
  src/config/
    layout.rs                 — add Pane { titlebar, close_window_with_last_session }
    navigation.rs             — add NavigationMode::Sidebar variant
    keyboard.rs               — add `preset` field
    bindings.rs               — unchanged
frontends/rioterm/
  src/
    bindings/
      mod.rs                  — add new Action variants (P1), parse strings (P1),
                                wire Tilix preset (P18)
      tilix_preset.rs (NEW)   — full Tilix default binding set
    layout/
      mod.rs                  — SplitAuto orientation pick, maximize state,
                                resize_in_direction, distribute_evenly,
                                find_neighbour_in_direction (spatial nav),
                                pane creation-order index, sync_input fields
      compute_tests.rs        — extend
    context/
      mod.rs                  — Context.read_only, Context.sync_input_override,
                                detach helpers, save/load layout
    renderer/
      pane_titlebar.rs (NEW)  — per-pane titlebar widget
      pane_dnd.rs (NEW)       — pane drag state machine + thumbnail + quadrant overlay
      sidebar.rs (NEW)        — session sidebar widget (NavigationMode::Sidebar)
      island.rs               — coexist with sidebar
    screen/
      mod.rs                  — dispatch new actions (split-out below by phase),
                                pane numbering, sync-input broadcast,
                                pane-titlebar hit-test routing
    mouse/
      mod.rs                  — track pane drag, double-click splitter detection
    application.rs            — wire new WindowEvent handling for pane DnD,
                                pane detach event, RioEvent additions
    router/
      mod.rs                  — DetachPaneToWindow handling
    session_layout.rs (NEW)   — serde structs + serializer + deserializer
docs/
  features/tiling.md (NEW)    — user-facing docs for new features
  features/tilix-preset.md (NEW) — preset binding reference
```

### New event-loop events (rio-backend)

```rust
// rio-backend/src/event.rs (additive)
pub enum RioEvent {
    // ...existing...
    DetachPaneToWindow {
        source_window: WindowId,
        source_tab: usize,
        source_node: NodeId,        // pane to extract
        target_position: (i32, i32),// absolute screen position for new window
    },
    TransferPaneToSession {
        source_window: WindowId,
        source_tab: usize,
        source_node: NodeId,
        target_window: WindowId,
        target_tab: usize,
        target_node: NodeId,
        target_quadrant: PaneDropQuadrant,
    },
}
```

## Performance Considerations

- Per-pane titlebar adds a fixed-height row (default 24 px) per pane only when opt-in. Renderer cost is one `rect()` + one `text_mut()` per visible pane — negligible.
- Pane drag thumbnail uses an alpha-blended copy of the source pane's last rendered frame. Capture only on drag start (one extra rect upload per drag) — negligible.
- Synchronized input multiplies each key event by the number of receiving panes. For very large sessions (>16 panes) this could be felt; mitigate by batching the PTY writes (single `Msg::Input` per pane, no extra allocation per key).
- Sidebar rendering reuses the existing renderable-content damage scheme; only redraws on session list change.
- The session JSON serializer walks the Taffy tree once per save (rare event) — no performance concern.

## Migration Notes

- All new behaviour is **off by default**. Existing users see no change unless they opt in via config.
- `pane.titlebar` default = `false`.
- `pane.close_window_with_last_session` default = `false` (matches today's behaviour).
- `keyboard.preset` default = `"default"` (current Rio bindings).
- `navigation.mode` default = `Tab` (unchanged).
- No migration of user data required.
- Saved session JSON files (US-8.6) are versioned (`{"version": 1, ...}`) so future schema changes don't break old files.

---

## Phase 1: Foundation — action enum, direction enum, config plumbing

**No dependencies.**

Stubs the entire surface so subsequent phases can land in parallel without touching the same files.

**Tasks**:
- [ ] Add `Direction` enum (`Up`, `Down`, `Left`, `Right`) to `frontends/rioterm/src/bindings/mod.rs`.
- [ ] Add new `Action` variants to `frontends/rioterm/src/bindings/mod.rs`:
   ```rust
   SplitAuto,
   FocusPaneByNumber(u8),              // 1..=10
   FocusPaneByDirection(Direction),
   ResizePaneInDirection(Direction),
   TogglePaneMaximized,                // distinct from ToggleMaximized (window)
   DistributePanesEvenly,
   ToggleSyncInputSession,
   TogglePaneSyncInputOverride,
   TogglePaneReadOnly,
   SaveSessionLayout,
   OpenSessionLayout,
   ToggleSessionSidebar,
   DetachPaneToWindow,
   ```
- [ ] Extend the `impl From<String> for Action` parser to accept the new lowercase names (`splitauto`, `focuspanebynumber(n)`, `focuspanebydirection(up|down|left|right)`, etc.). Re-use existing regex pattern from `selecttab(n)`.
- [ ] Pre-stub every new action's match arm in `screen/mod.rs` action dispatch with `Act::Foo => { /* TODO(phase N) */ }`. This prevents later phases from conflicting at the same match expression.
- [ ] Add config fields:
   - `rio-backend/src/config/layout.rs`:
     - `Pane { pub titlebar: bool, pub close_window_with_last_session: bool }` with serde defaults (`false`, `false`).
     - Wire `pane: Pane` into the top-level config struct.
   - `rio-backend/src/config/keyboard.rs`:
     - `pub preset: String` (default `"default"`).
   - `rio-backend/src/config/navigation.rs`:
     - `NavigationMode::Sidebar` variant + `SIDEBAR_STR = "Sidebar"` + `FromStr` arm + display.
- [ ] Add new `RioEvent` variants in `rio-backend/src/event.rs` (no-op handlers in router):
   - `DetachPaneToWindow { ... }`
   - `TransferPaneToSession { ... }`
- [ ] Add `PaneDropQuadrant` enum (`Left`, `Top`, `Right`, `Bottom`) in `rio-backend/src/event.rs`.

**Automated Verification**:
- [ ] `cargo build -p rioterm --no-default-features --features=wayland` passes.
- [ ] `cargo build -p rioterm --no-default-features --features=x11` passes.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] `cargo test -p rio-backend` passes (new config defaults round-trip via serde).
- [ ] New unit test `bindings::tests::parses_new_action_names` — string-to-Action parser covers every new action.

---

## Phase 2: SplitAuto

**Dependencies: Phase 1.**

Implements US-1.3 — orient the split by the current pane's aspect ratio.

**Tasks**:
- [ ] Add `ContextGrid::split_auto(&mut self, context: Context<T>, sugarloaf: &mut Sugarloaf)` in `layout/mod.rs`. If current pane's width > height → call `split_right`, else `split_down`. Use the pane's `layout_rect[2]` / `[3]` for measurement.
- [ ] Add `Screen::split_auto` in `screen/mod.rs` (mirror existing `split_right` / `split_down`, allocate rich_text_id).
- [ ] Replace the Phase-1 stub `Act::SplitAuto => { /* TODO */ }` with a call to `Screen::split_auto`.

**Automated Verification**:
- [ ] New unit test `layout::compute_tests::split_auto_picks_horizontal_for_wide_pane`.
- [ ] New unit test `layout::compute_tests::split_auto_picks_vertical_for_tall_pane`.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] `cargo test --features wgpu` passes.

---

## Phase 3: FocusPaneByNumber (creation-order index)

**Dependencies: Phase 1.**

Implements US-2.1 — `Alt+1..Alt+9` and `Alt+0` jump to pane N in creation order, persistent across removals.

**Tasks**:
- [ ] Add `pub pane_number: u32` field to `ContextGridItem` in `layout/mod.rs`. Set on creation.
- [ ] Add `next_pane_number: u32` counter to `ContextGrid` (monotonic, incremented on each split). Numbers do NOT compact on removal — Tilix renumbers visually by sort order; we implement the same: pane number is the position of the NodeId in `get_ordered_keys()` (1-indexed).
- [ ] **Decision**: matches Tilix exactly — re-number by visual order at query time. Add `ContextGrid::focus_pane_by_number(&mut self, n: u8) -> bool`:
   ```rust
   let keys = self.get_ordered_keys();
   if let Some(&id) = keys.get((n.saturating_sub(1)) as usize) {
       self.current = id;
       return true;
   }
   false
   ```
- [ ] Replace the Phase-1 stub `Act::FocusPaneByNumber(n) => { /* TODO */ }` to call `context_manager.current_grid_mut().focus_pane_by_number(*n)` and emit a redraw if it returned `true`.
- [ ] Add helper `ContextGrid::current_pane_number(&self) -> Option<u8>` for titlebar rendering (Phase 8).

**Automated Verification**:
- [ ] New unit test `layout::compute_tests::focus_pane_by_number_indexes_in_visual_order`.
- [ ] New unit test `layout::compute_tests::focus_pane_by_number_handles_out_of_range`.
- [ ] `cargo test --features wgpu` passes.

---

## Phase 4: FocusPaneByDirection (spatial nearest neighbour)

**Dependencies: Phase 1.**

Implements US-2.2 — `Alt+Arrow` picks the geometrically closest pane in that direction relative to the current pane's centre.

**Tasks**:
- [ ] Add `ContextGrid::find_neighbour_in_direction(&self, dir: Direction) -> Option<NodeId>` in `layout/mod.rs`. Algorithm:
   1. Take current pane's centre `(cx, cy)`.
   2. Filter candidates whose centre is strictly on the requested side.
   3. Score each by `dx² + dy²` (Euclidean). Lowest score wins.
   4. Return `None` if no candidate exists (pane is at the edge — key has no effect).
- [ ] Add `ContextGrid::focus_neighbour_in_direction(&mut self, dir: Direction) -> bool`.
- [ ] Replace the Phase-1 stub `Act::FocusPaneByDirection(d) => { /* TODO */ }` with the call + redraw.

**Automated Verification**:
- [ ] New unit test `layout::compute_tests::focus_direction_picks_nearest_in_each_quadrant` (3-pane T-layout).
- [ ] New unit test `layout::compute_tests::focus_direction_returns_none_at_edge`.
- [ ] `cargo test --features wgpu` passes.

---

## Phase 5: Keyboard pane resize (10 px in direction)

**Dependencies: Phase 1.**

Implements US-3.1 — `Shift+Alt+Arrow` shifts the boundary between the current pane and its neighbour in the indicated direction by 10 px × DPI scale.

**Tasks**:
- [ ] Add `ContextGrid::resize_in_direction(&mut self, dir: Direction, sugarloaf: &mut Sugarloaf)` in `layout/mod.rs`.
   - For `Left` / `Right`: re-use `find_horizontal_neighbors`. Compute new sizes by ±10 × scale px, write via Taffy `set_style`, `compute_layout`, `apply_taffy_layout`.
   - For `Up` / `Down`: same with `find_vertical_neighbors`.
   - If no neighbour: noop.
- [ ] Replace the Phase-1 stub `Act::ResizePaneInDirection(d) => { /* TODO */ }` with the call.
- [ ] **Decision**: do NOT modify existing `MoveDividerUp/Down/Left/Right` actions — they remain for backwards compatibility and current Rio default bindings. The Tilix preset will map `Shift+Alt+Arrow` to `ResizePaneInDirection`.

**Automated Verification**:
- [ ] New unit test `layout::compute_tests::resize_in_direction_grows_current_pane`.
- [ ] New unit test `layout::compute_tests::resize_in_direction_at_edge_is_noop`.
- [ ] `cargo test --features wgpu` passes.

---

## Phase 6: MaximizePane + swap-while-maximized

**Dependencies: Phase 1.**

Implements US-4.1 through US-4.4. A maximized pane fills the session area; all other panes hide but their layout is preserved. Focus change while maximized swaps the maximized pane.

**Tasks**:
- [ ] Add `pub maximized: Option<NodeId>` field to `ContextGrid` in `layout/mod.rs`.
- [ ] Add `ContextGrid::toggle_maximize(&mut self, sugarloaf: &mut Sugarloaf)`:
   - If `maximized.is_none()` and `inner.len() > 1`: store `Some(self.current)`, set every other panel's Taffy style to `display: None`.
   - If `maximized.is_some()`: restore all panels' Taffy style to flex, set `maximized = None`.
   - Recompute layout via `apply_taffy_layout`.
- [ ] Update every focus-setting code path in `layout/mod.rs` (`select_next_split`, `select_prev_split`, `focus_pane_by_number`, `focus_neighbour_in_direction`, `select_current_based_on_mouse`) to call `swap_maximized_if_active(new_current)`, which:
   1. If `maximized.is_some()` and `new_current` differs from the maximized node: restore the previously-maximized pane to flex, set the newly-focused pane to fill, update `maximized = Some(new_current)`.
   2. Recompute layout.
- [ ] Replace the Phase-1 stub `Act::TogglePaneMaximized => { /* TODO */ }`.
- [ ] When `remove_current` runs and the removed node was the maximized one: clear `maximized` first, then proceed with normal removal (covers US-7.3).
- [ ] Override `Action::SplitRight` / `SplitDown` / `SplitAuto` dispatch to early-return when a pane is maximized (Tilix disallows split while maximized — log + ignore).

**Automated Verification**:
- [ ] New unit test `layout::compute_tests::maximize_hides_other_panes`.
- [ ] New unit test `layout::compute_tests::restore_brings_back_original_layout`.
- [ ] New unit test `layout::compute_tests::focus_change_while_maximized_swaps_maximized_node`.
- [ ] New unit test `layout::compute_tests::single_pane_maximize_is_noop`.
- [ ] `cargo test --features wgpu` passes.

---

## Phase 7: DistributeEvenly (double-click splitter + action)

**Dependencies: Phase 1.**

Implements US-3.3 — double-click splitter handle redistributes panes along that axis to equal size. Also expose as `Action::DistributePanesEvenly` (operates on all axes).

**Tasks**:
- [ ] Add `ContextGrid::distribute_evenly_along(&mut self, axis: BorderDirection, sugarloaf: &mut Sugarloaf)` in `layout/mod.rs`. Walk the Taffy tree; for every flex container whose `flex_direction` matches the requested axis, set every child's `flex_basis = length(0)`, `flex_grow = 1.0`. Recompute.
- [ ] Add `ContextGrid::distribute_evenly_all(&mut self, sugarloaf: &mut Sugarloaf)` — same for both axes.
- [ ] In `mouse/mod.rs` add `double_click_state: Option<{node_id: NodeId, last_t: Instant}>` (or extend existing double-click tracking if present).
- [ ] In `application.rs` mouse-down handler: when click hits a splitter (`find_border_at_position` returns `Some`), check if it's a double-click within 400 ms of the previous on the same border → call `distribute_evenly_along(border.direction, ...)`.
- [ ] Replace the Phase-1 stub `Act::DistributePanesEvenly => { /* TODO */ }`.

**Automated Verification**:
- [ ] New unit test `layout::compute_tests::distribute_evenly_along_horizontal_axis`.
- [ ] New unit test `layout::compute_tests::distribute_evenly_all_both_axes`.
- [ ] `cargo test --features wgpu` passes.

**Manual Verification**:
- [ ] Open Rio with `pane.titlebar = true`, split into 4 panes uneven sizes, double-click a vertical splitter — confirm left/right neighbours become equal width without affecting other axes.

---

## Phase 8: Per-pane titlebar (renderer + layout + buttons + hit-test)

**Dependencies: Phase 1.**

Implements US-6.1 through US-6.5 — the opt-in per-pane titlebar UI. Required by Phases 9 (drag), 12 (sync icon), 13 (menu).

**Tasks**:
- [ ] Add `frontends/rioterm/src/renderer/pane_titlebar.rs` with:
   - `pub const PANE_TITLEBAR_HEIGHT: f32 = 24.0`
   - `pub struct PaneTitlebar` with `render(sugarloaf, layout_rect, context, state)` drawing:
     - background `Rect` (1 px below pane)
     - sync-input icon (left) — colour reflects state
     - title text (centre-left, truncated with ellipsis to fit)
     - maximize/restore button (right, icon flips when maximized)
     - menu button `⋯` (right)
     - close button `✕` (right)
   - `pub struct TitlebarHitMap { drag_handle: Rect, sync_icon: Rect, max_button: Rect, menu_button: Rect, close_button: Rect }` returned from `render` for hit-testing.
- [ ] In `layout/mod.rs` `create_panel_style`: when `panel_config.titlebar_enabled`, add a top `padding` of `PANE_TITLEBAR_HEIGHT * scale` so the terminal grid doesn't draw under the titlebar.
- [ ] In `screen/mod.rs` render path: after the per-grid `apply_taffy_layout`, iterate `grid.contexts()`, render the titlebar above each pane if config opt-in and `grid.panel_count() > 1` (Tilix only shows titlebars when split). Single-pane sessions render no titlebar.
- [ ] In `mouse/mod.rs` mouse-down dispatch (`application.rs:1004`): when click Y is inside any pane's titlebar Y range, route to `screen.handle_pane_titlebar_click(node_id, hit, button)`:
   - left-click close → `CloseCurrentSplitOrTab`
   - left-click maximize → `TogglePaneMaximized`
   - left-click menu → open titlebar menu (Phase 14 stub for now; emit a no-op until that phase lands)
   - left-click sync icon → `TogglePaneSyncInputOverride` (stub until Phase 12)
   - middle-click anywhere on titlebar → no-op until Phase 15
   - left-click drag handle (rest of titlebar) → no-op until Phase 9
   - double-click on title text area → `TogglePaneMaximized` (US-4.1)
- [ ] Wire `pane.titlebar` config through `ContextManagerConfig` so `ContextGrid` knows whether to reserve layout space and render. Make titlebar enable/disable hot-reloadable via existing config-reload path.
- [ ] Add titlebar font + colour entries to existing colour scheme system (`rio-backend/src/config/colors`): `pane_titlebar_bg`, `pane_titlebar_fg`, `pane_titlebar_bg_inactive`, `pane_titlebar_fg_inactive`. Default values derived from existing tab-strip colours.

**Automated Verification**:
- [ ] New unit test `layout::compute_tests::titlebar_reserves_height_when_enabled`.
- [ ] New unit test `layout::compute_tests::titlebar_no_reserve_when_disabled`.
- [ ] New unit test `renderer::pane_titlebar::tests::hitmap_buttons_within_titlebar_rect`.
- [ ] `cargo test --features wgpu` passes.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.

**Manual Verification**:
- [ ] Set `pane.titlebar = true`, split, confirm thin bar appears above each pane with all four buttons visible.
- [ ] Click ✕ → pane closes.
- [ ] Click ⛶ → pane maximizes, click again → restores.
- [ ] With `pane.titlebar = false` (default), confirm Rio looks identical to today.

---

## Phase 9: Pane drag within session (state machine + quadrant overlay)

**Dependencies: Phase 8.**

Implements US-5.1, US-5.2, US-5.3, US-5.6, US-5.7 — intra-session drag, quadrant detection, drop-on-self ignored, drop while maximized ignored.

**Tasks**:
- [ ] Add `frontends/rioterm/src/renderer/pane_dnd.rs` with:
   ```rust
   pub enum PaneDropQuadrant { Left, Top, Right, Bottom }
   pub fn quadrant_at(target_rect: [f32;4], cursor: (f32,f32)) -> PaneDropQuadrant;
   pub struct PaneDragState {
       source_window: WindowId,
       source_tab: usize,
       source_node: NodeId,
       cursor: (f32, f32),
       thumbnail: ThumbnailHandle, // alpha-blended copy of source pane
   }
   ```
- [ ] Quadrant geometry: cursor in left triangle (below the `\` diagonal and above the `/` diagonal that bisect the target) → `Left`, etc. Pure function — easy to unit-test.
- [ ] In `mouse/mod.rs`: on mouse-down inside a titlebar drag-handle area (Phase 8 hit-test), set `Screen::pane_drag = Some(PaneDragState { ... })` and capture a thumbnail (snapshot of the pane's current rich-text grid frame).
- [ ] On mouse-move while `pane_drag.is_some()`: update cursor, find target NodeId via `find_context_at_position`, compute quadrant. Render in the next frame:
   - Translucent thumbnail at cursor.
   - 4 px highlight rect on the target's matching edge (cursor's quadrant).
   - Skip render if cursor is over the source pane (drop-on-self, US-5.6).
   - Skip render if grid is maximized (US-5.7).
- [ ] On mouse-up while `pane_drag.is_some()`:
   - Cursor over a non-source pane in the same session and not maximized: call `ContextGrid::move_pane(source: NodeId, target: NodeId, quadrant: PaneDropQuadrant)`. Internally: remove source from its location (preserving `Context`), split target in the quadrant direction, re-insert source as the new child. Update `current = source_node`.
   - Cursor over source pane: ignore.
   - Cursor outside any pane / over maximized grid: ignore for this phase (Phase 11 handles tear-off).
   - Clear `pane_drag`.
- [ ] Screen: add `Screen::pane_drag: Option<PaneDragState>` field + a small render-pass section that draws the overlay when set.

**Automated Verification**:
- [ ] New unit test `renderer::pane_dnd::tests::quadrant_at_left_triangle_returns_left` (and equivalents for top/right/bottom + boundary cases on the diagonals).
- [ ] New unit test `layout::compute_tests::move_pane_within_session_preserves_other_panes`.
- [ ] New unit test `layout::compute_tests::move_pane_to_self_is_noop`.
- [ ] New unit test `layout::compute_tests::move_pane_while_maximized_is_noop`.
- [ ] `cargo test --features wgpu` passes.

**Manual Verification**:
- [ ] With `pane.titlebar = true` and 3+ panes split into a non-trivial layout, grab a pane's titlebar and drop on another pane's left/right/top/bottom triangles — confirm the dragged pane lands on the correct edge each time.
- [ ] Drop on the source pane itself → pane snaps back to original position.
- [ ] Maximize a pane, attempt to drag from titlebar over the maximized pane → no drop occurs.

---

## Phase 10: Cross-session pane drag

**Dependencies: Phase 9.**

Implements US-5.4 — drag a pane onto a tab/sidebar entry or directly onto a pane in another session.

**Tasks**:
- [ ] Extend the mouse-up branch in Phase 9: if the cursor is over a different tab's clickable area (Island tab) or over the sidebar's session entry, OR over a pane that belongs to a different `ContextGrid`, emit `RioEvent::TransferPaneToSession { source_*, target_*, target_quadrant }`.
- [ ] In `application.rs` `RioEvent` handler: extract the source `Context` from the source `ContextGrid` (remove without dropping the PTY — split out `ContextGrid::take_pane(node) -> Option<Context<T>>` that detaches from Taffy but does NOT call `Drop`). If the source grid becomes empty, remove it from `ContextManager.contexts` (covers Tilix US-5.4 step "if original session empty, it closes").
- [ ] In the target grid, call `ContextGrid::insert_pane_at(target_node, quadrant, context)` which splits target in the indicated direction and uses the existing `Context` instead of creating a new one.
- [ ] If the source grid had a custom title or color, do NOT migrate it (Tilix treats sessions as having their own title).
- [ ] Update `current_index` / `current_route` in `ContextManager` to switch to the destination session.

**Automated Verification**:
- [ ] New unit test `context::tests::transfer_pane_across_sessions_preserves_pty` (uses mock event listener).
- [ ] New unit test `context::tests::transfer_last_pane_removes_source_session`.
- [ ] `cargo test --features wgpu` passes.

**Manual Verification**:
- [ ] Open 2 tabs (sessions), each with multiple panes. Drag a pane's titlebar in Tab A over Tab B's tab strip → confirm pane moves to Tab B and focus switches to Tab B.
- [ ] Drag the *last* pane of Tab A to Tab B → confirm Tab A closes.

---

## Phase 11: Tear-off pane to new window + Detach menu item

**Dependencies: Phase 10.**

Implements US-5.5 and US-5.8 — drop outside any window creates a new window holding only that pane; "Detach" menu entry triggers the same flow.

**Tasks**:
- [ ] Extend the mouse-up branch: if cursor is outside any window content area (no target NodeId found AND not over Island/sidebar), emit `RioEvent::DetachPaneToWindow { source_*, target_position: cursor_screen_pos }`.
- [ ] In `application.rs` `RioEvent` handler: call `ContextGrid::take_pane`, then `Router::create_window` with `WindowAttributes` positioned at `target_position` and a single-pane initial grid built from the detached `Context`.
- [ ] Add a new constructor `ContextGrid::from_existing_context(context: Context<T>, ...)` that bypasses PTY spawn.
- [ ] In the titlebar menu (Phase 14 will host the menu; stub the menu entry here): wire the "Detach" entry to `Action::DetachPaneToWindow`, which calls the same flow with `target_position = window_position + (24, 24)` offset.
- [ ] Cross-platform note: detect "outside window" using the OS cursor position from `WindowEvent::CursorLeft` + last known screen-space position. On Wayland, screen-space coordinates are not available — fall back to "drop on Wayland always becomes tear-off relative to current window position".

**Automated Verification**:
- [ ] New unit test `context::tests::detach_pane_creates_isolated_grid`.
- [ ] `cargo test --features wgpu` passes.

**Manual Verification**:
- [ ] Drag a pane's titlebar off the window and release on the desktop (X11/macOS) → new Rio window appears containing only that pane.
- [ ] Right-click pane titlebar → "Detach" → new window appears near the current one.
- [ ] On Wayland: drop outside → new window appears at default position with the pane.

---

## Phase 12: Synchronized input (session + per-pane override)

**Dependencies: Phase 8.**

Implements US-9.1, US-9.2, US-9.3.

**Tasks**:
- [ ] Add `pub sync_input: bool` field to `ContextGrid`.
- [ ] Add `pub sync_input_excluded: bool` field to `Context` (per-pane opt-out).
- [ ] In the keystroke pipeline (locate where the focused pane's `Messenger` is fed input — `bindings/mod.rs` action dispatch where text is sent to PTY): when `grid.sync_input` is true, iterate `grid.contexts_mut()`, sending the same key to every `Context` whose `sync_input_excluded == false`.
- [ ] Replace the Phase-1 stub `Act::ToggleSyncInputSession`:
   - Toggle `grid.sync_input`.
   - When toggling OFF: also clear all per-pane `sync_input_excluded` flags (US-9.3 "per-pane overrides are cleared").
- [ ] Replace the Phase-1 stub `Act::TogglePaneSyncInputOverride`:
   - Toggle `grid.contexts_mut()[current].val.sync_input_excluded`.
   - Only meaningful while `grid.sync_input == true`.
- [ ] In `pane_titlebar.rs` (Phase 8): use `sync_input` + `sync_input_excluded` to pick the icon variant (off / on / overridden).
- [ ] In titlebar hit-test handler (Phase 8): clicking the sync icon dispatches `Act::TogglePaneSyncInputOverride`.

**Automated Verification**:
- [ ] New unit test `context::tests::sync_input_broadcasts_to_all_panes`.
- [ ] New unit test `context::tests::sync_input_skips_overridden_pane`.
- [ ] New unit test `context::tests::disabling_sync_clears_overrides`.
- [ ] `cargo test --features wgpu` passes.

**Manual Verification**:
- [ ] Split into 3 panes, enable session-wide sync via configured keybinding, type — all 3 panes receive characters.
- [ ] Click the sync icon on pane 2's titlebar — pane 2 stops receiving, panes 1 and 3 still in sync.
- [ ] Disable session sync — all panes type independently; per-pane overrides reset.

---

## Phase 13: Per-pane read-only mode

**Dependencies: Phase 8.**

Implements the Tilix "Read-Only" toggle in the titlebar menu.

**Tasks**:
- [ ] Add `pub read_only: bool` to `Context` in `context/mod.rs`.
- [ ] In `Messenger::send_input` (or the closest equivalent — locate where keystrokes are turned into PTY writes): early-return when `context.read_only`.
- [ ] Replace the Phase-1 stub `Act::TogglePaneReadOnly` to flip the flag on the current pane.
- [ ] When `read_only == true`, render a small lock indicator on the titlebar's left edge (Phase 8 layout already allocates space — reuse the sync-icon slot when read-only is on, prioritising read-only display).

**Automated Verification**:
- [ ] New unit test `context::tests::read_only_blocks_pty_writes`.
- [ ] `cargo test --features wgpu` passes.

**Manual Verification**:
- [ ] Toggle read-only on a pane, type — nothing appears.
- [ ] Toggle off — typing resumes.

---

## Phase 14: Titlebar popover menu

**Dependencies: Phase 8, Phase 13 (for read-only entry).**

Implements US-6.1 — full menu popover when clicking the title-text/dropdown or right-clicking the titlebar.

**Tasks**:
- [ ] Add `frontends/rioterm/src/renderer/pane_titlebar_menu.rs` with `PaneTitlebarMenu` widget — uses sugarloaf primitives, mirrors the existing `command_palette` overlay pattern.
- [ ] Menu entries:
   - **Find…** → dispatches `Act::SearchForward` (already exists)
   - **Read-Only** (checkbox) → `Act::TogglePaneReadOnly`
   - **Assistants** → submenu stub (entries labelled "Password manager", "Bookmarks", marked "Not implemented" — clicks no-op)
   - **Profiles** → submenu stub ("Edit profile" → opens editor for `~/.config/rio/config.toml`)
   - **Other** → submenu:
     - "Show File Browser" — stub
     - "Save Output" — stub
     - "Reset" — dispatches existing `Act::ClearSelection` + emit `\\x1bc` (terminfo reset)
     - "Reset and Clear" — same + scrollback clear
     - "Encoding" — stub label "UTF-8" (Rio is always UTF-8)
     - "Layout Options" — stub
     - "Monitor Silence" — stub
   - **Detach** (Phase 11) → `Act::DetachPaneToWindow`
   - **Close** → `Act::CloseCurrentSplitOrTab`
- [ ] Trigger: replace Phase-8 stub for left-click on menu-button + right-click anywhere on titlebar → call `screen.open_pane_titlebar_menu(node_id, screen_xy)`.
- [ ] Dismiss: click outside menu, press `Escape`, or select an entry.

**Automated Verification**:
- [ ] New unit test `renderer::pane_titlebar_menu::tests::menu_layout_contains_all_required_entries`.
- [ ] `cargo test --features wgpu` passes.

**Manual Verification**:
- [ ] Right-click pane titlebar → menu appears at click position.
- [ ] Click "Read-Only" → entry shows checkbox; typing in the pane is blocked.
- [ ] Click "Detach" → pane moves to a new window.
- [ ] Press `Escape` while menu open → menu dismisses.

---

## Phase 15: Middle-click close + close-window-with-last-session pref

**Dependencies: Phase 8.**

Implements US-6.5, US-7.2. (US-7.3 restore-then-close is already covered by Phase 6.)

**Tasks**:
- [ ] Replace the Phase-8 stub for middle-click on titlebar: when `config.pane.close_on_middle_click` is `true`, dispatch `Act::CloseCurrentSplitOrTab` (close the pane the titlebar belongs to, not necessarily the focused one — use the `node_id` from the hit-test).
- [ ] Add `close_on_middle_click: bool` (default `false`) to the `Pane` struct in `rio-backend/src/config/layout.rs` (extension of Phase 1).
- [ ] In `ContextManager::close_current_grid_if_empty` (or equivalent — the path where removing the last pane removes the grid): consult `config.pane.close_window_with_last_session`. If `true`: close the window. If `false`: insert a fresh empty session.

**Automated Verification**:
- [ ] New unit test `context::tests::close_last_pane_with_pref_off_creates_empty_session`.
- [ ] New unit test `context::tests::close_last_pane_with_pref_on_closes_window`.
- [ ] `cargo test --features wgpu` passes.

**Manual Verification**:
- [ ] Set `pane.close_on_middle_click = true`, split into 2 panes, middle-click left pane's titlebar → pane closes.
- [ ] Default config (`close_window_with_last_session = false`), close last pane → fresh empty session appears.
- [ ] Set `pane.close_window_with_last_session = true`, close last pane → window closes.

---

## Phase 16: Session JSON save/restore

**Dependencies: Phase 1.**

Implements US-8.6 — `Shift+Ctrl+S` saves the current session's layout to JSON; `Shift+Ctrl+O` loads one.

**Tasks**:
- [ ] Add `frontends/rioterm/src/session_layout.rs` with serde structs:
   ```rust
   #[derive(Serialize, Deserialize)]
   pub struct SessionLayoutFile {
       pub version: u32, // 1
       pub session_title: Option<String>,
       pub tree: SessionNode,
   }
   #[derive(Serialize, Deserialize)]
   pub enum SessionNode {
       Pane { working_dir: Option<String>, profile: Option<String> },
       Split { direction: SplitDir, ratio: f32, left: Box<SessionNode>, right: Box<SessionNode> },
   }
   ```
- [ ] Add `ContextGrid::to_session_layout(&self) -> SessionLayoutFile` — walk the Taffy tree, emit one `SessionNode` per leaf/container. Encode current split ratios from Taffy's computed widths/heights.
- [ ] Add `ContextGrid::apply_session_layout(&mut self, layout: SessionLayoutFile, event_proxy, window_id, sugarloaf, config)` — rebuild the grid: recursively split, spawn PTYs in the recorded `working_dir`.
- [ ] Replace the Phase-1 stubs `Act::SaveSessionLayout` / `Act::OpenSessionLayout` to call a native file dialog (`rfd` crate or platform-specific) and serialise/deserialise. Default extension `.rio-session.json`.
- [ ] If `rfd` is not already in `Cargo.toml`, add it as a new workspace dependency.

**Automated Verification**:
- [ ] New unit test `session_layout::tests::round_trip_preserves_tree_structure`.
- [ ] New unit test `session_layout::tests::ratio_within_001_after_round_trip`.
- [ ] New unit test `session_layout::tests::version_mismatch_returns_error`.
- [ ] `cargo test --features wgpu` passes.

**Manual Verification**:
- [ ] Split 4 panes asymmetrically, save layout, close the window, reopen Rio, load layout → confirm layout matches (ratios within ~1%, working directories restored).

---

## Phase 17: Sidebar navigation mode

**Dependencies: Phase 1.**

Implements US-8.5.

**Tasks**:
- [ ] Add `frontends/rioterm/src/renderer/sidebar.rs` — collapsible left-side panel listing sessions. Width default 180 px; collapsed width 0.
- [ ] Add `Sidebar` widget mirroring `Island`'s style: each session row shows the session title and an optional ✕ close button on hover.
- [ ] Drag-reorder: re-use `TabDrag` pattern (`renderer/island.rs:64`) adapted to vertical orientation.
- [ ] Wire `NavigationMode::Sidebar` (added in Phase 1) — when active, render `Sidebar` instead of `Island`. Update `Screen::render` to reserve the sidebar's left margin.
- [ ] Replace the Phase-1 stub `Act::ToggleSessionSidebar` — animates the sidebar in/out (use a simple time-based 150 ms ease). State persists in `Screen`.
- [ ] Sidebar respects existing colours: `colors.sidebar_bg`, `colors.sidebar_fg` (add defaults derived from `tabs_active_bg` / `tabs_active_fg`).

**Automated Verification**:
- [ ] New unit test `renderer::sidebar::tests::layout_reserves_collapsed_zero_width`.
- [ ] New unit test `renderer::sidebar::tests::expanded_width_matches_config`.
- [ ] `cargo test --features wgpu` passes.

**Manual Verification**:
- [ ] Set `navigation.mode = "Sidebar"`, open Rio, create 3 sessions → sidebar shows 3 entries.
- [ ] Press `F12` (after Tilix preset applied, or whatever binding the user sets) → sidebar slides closed; press again → reopens.
- [ ] Drag the middle session's row up → order changes; tab strip is hidden in this mode.

---

## Phase 18: Tilix keybinding preset + documentation

**Dependencies: Phases 2–17.**

Implements US-11 — full Tilix-equivalent default bindings, opt-in via `keyboard.preset = "tilix"`.

**Tasks**:
- [ ] Add `frontends/rioterm/src/bindings/tilix_preset.rs` exposing `pub fn tilix_key_bindings() -> Vec<KeyBinding>` and `pub fn tilix_mouse_bindings() -> Vec<MouseBinding>` covering every spec §11 row:
   - Splits: `Ctrl+Alt+R`=`SplitRight`, `Ctrl+Alt+D`=`SplitDown`, `Ctrl+Alt+A`=`SplitAuto`
   - Focus by number: `Alt+1..Alt+9`=`FocusPaneByNumber(1..9)`, `Alt+0`=`FocusPaneByNumber(10)`
   - Focus by direction: `Alt+Up/Down/Left/Right`=`FocusPaneByDirection(...)`
   - Sequential cycle: `Ctrl+Tab`=`SelectNextSplit`, `Ctrl+Shift+Tab`=`SelectPrevSplit`
   - Resize: `Shift+Alt+Up/Down/Left/Right`=`ResizePaneInDirection(...)`
   - Maximize: `Shift+Ctrl+X`=`TogglePaneMaximized`
   - Close pane: `Shift+Ctrl+W`=`CloseCurrentSplitOrTab`
   - Close session: `Shift+Ctrl+Q`=`TabCloseCurrent`
   - New session: `Shift+Ctrl+T`=`TabCreateNew`
   - Switch session: `Ctrl+Alt+1..9`=`SelectTab(0..8)`, `Ctrl+Alt+0`=`SelectTab(9)`
   - Next/prev session: `Ctrl+PageDown`=`SelectNextTab`, `Ctrl+PageUp`=`SelectPrevTab`
   - Reorder session: `Ctrl+Shift+PageDown`=`MoveCurrentTabToNext`, `Ctrl+Shift+PageUp`=`MoveCurrentTabToPrev`
   - Sidebar: `F12`=`ToggleSessionSidebar`
   - Save/load: `Shift+Ctrl+S`=`SaveSessionLayout`, `Shift+Ctrl+O`=`OpenSessionLayout`
   - Full-screen: `F11`=`ToggleFullscreen`
- [ ] In `bindings/mod.rs::default_key_bindings()` (or its call site in `Screen::new`/binding setup): when `config.keyboard.preset == "tilix"`, replace the default keybinding set with `tilix_preset::tilix_key_bindings()`.
- [ ] User-defined `keyboard.bindings` in config continues to take priority over the preset (existing behaviour).
- [ ] Add `docs/features/tilix-preset.md` — full table of preset bindings + how to enable.
- [ ] Add `docs/features/tiling.md` — overview of all new features (pane numbering, titlebar opt-in, sync input, read-only, session save/load, sidebar mode, DnD).
- [ ] Update `docs/agents/plans/2026-06-19-tilix-tiling.md` — mark `status: complete` at the end.

**Automated Verification**:
- [ ] New unit test `bindings::tilix_preset::tests::preset_contains_every_spec_row` (asserts the bindings match the §11 table exactly).
- [ ] `cargo test --features wgpu` passes.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.

**Manual Verification**:
- [ ] Set `keyboard.preset = "tilix"`, restart Rio, exercise every binding from spec §11 — confirm each performs its documented action.
- [ ] Without `keyboard.preset` set, confirm Rio's existing default bindings still work (regression check).

---

## References

- Tilix source for behavioural reference: <https://github.com/gnunn1/tilix>
- Rio existing split engine: `frontends/rioterm/src/layout/mod.rs:108` (`ContextGrid`)
- Rio Action enum: `frontends/rioterm/src/bindings/mod.rs:323`
- Rio action dispatch: `frontends/rioterm/src/screen/mod.rs:881`
- Rio Island (tab strip) precedent for drag: `frontends/rioterm/src/renderer/island.rs:64` (`TabDrag`)
- Rio multi-window: `frontends/rioterm/src/router/mod.rs:501` (`Router::create_window`)
- Sugarloaf primitive draw API: `sugarloaf/src/sugarloaf.rs:603+`
- This plan: `docs/agents/plans/2026-06-19-tilix-tiling.md`
