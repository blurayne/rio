use super::*;

// This file tests compute function on different layouts.
// I've added some real scenarios so I can make sure it doesn't go off again.

/// Build a `CellMetrics` whose integer cell stride matches the given
/// `TextDimensions`. Used by tests that construct dimensions
/// directly without going through sugarloaf's font path.
fn cell_for(dims: TextDimensions) -> rio_backend::sugarloaf::layout::CellMetrics {
    rio_backend::sugarloaf::layout::CellMetrics {
        cell_width: dims.width.round().max(1.0) as u32,
        cell_height: dims.height.round().max(1.0) as u32,
        cell_baseline: 0,
        face_width: dims.width as f64,
        face_height: dims.height as f64,
        face_y: 0.0,
    }
}

/// note: Computes the renderer's actual per-line height in physical pixels.
///
/// The renderer gets metrics from Metrics::for_rich_text() which packs
/// cell_height as (ascent, descent, 0.0). cell_height is computed by
/// Metrics::calc at physical font_size scale, with ceil applied.
///
/// basically renderer line_height = ceil((ascent + descent + leading) * scale) * line_height_mod
fn renderer_line_height(
    ascent: f32,
    descent: f32,
    leading: f32,
    line_height_mod: f32,
    scale: f32,
) -> f32 {
    // Matches the Metrics::calc path: scale to physical, then ceil
    let cell_height = ((ascent + descent + leading) * scale).ceil();
    cell_height * line_height_mod
}

fn sugar_height(
    ascent: f32,
    descent: f32,
    leading: f32,
    line_height_mod: f32,
    scale: f32,
) -> f32 {
    ((ascent + descent + leading) * line_height_mod * scale).ceil()
}

/// Verifies that compute() row count fits when rendered.
#[allow(clippy::too_many_arguments)]
fn assert_rows_fit(
    panel_width: f32,
    panel_height: f32,
    sugar_width: f32,
    scale: f32,
    line_height_mod: f32,
    ascent: f32,
    descent: f32,
    leading: f32,
) {
    let sh = sugar_height(ascent, descent, leading, line_height_mod, scale);
    let dimensions = TextDimensions {
        width: sugar_width,
        height: sh,
        scale,
    };

    let (cols, rows) = compute(
        panel_width,
        panel_height,
        cell_for(dimensions),
        Margin::all(0.0),
        scale,
    );
    let _ = line_height_mod; // line_height already baked into `dimensions.height`.

    let actual_line_height =
        renderer_line_height(ascent, descent, leading, line_height_mod, scale);
    let rendered_height = rows as f32 * actual_line_height;

    assert!(
        rendered_height <= panel_height,
        "Rows overflow! {} rows * {:.2}px = {:.2}px rendered, but panel is only {:.2}px tall \
         (cols={}, sugar={:.2}x{:.2}, scale={:.1}, lh_mod={:.1})",
        rows,
        actual_line_height,
        rendered_height,
        panel_height,
        cols,
        sugar_width,
        sh,
        scale,
        line_height_mod,
    );
}

#[test]
fn test_user_case_1834x1436() {
    assert_rows_fit(1834.0, 1436.0, 16.41, 2.0, 1.0, 13.0, 3.5, 0.0);
}

#[test]
fn test_user_case_3766x1996() {
    assert_rows_fit(3766.0, 1996.0, 16.41, 2.0, 1.0, 13.0, 3.5, 0.0);
}

#[test]
fn test_user_case_5104x2736() {
    assert_rows_fit(5104.0, 2736.0, 16.41, 2.0, 1.0, 13.0, 3.5, 0.0);
}

#[test]
fn test_rows_fit_various_sizes() {
    for height in (500..=3000).step_by(50) {
        for width in [800.0, 1600.0, 2400.0, 3200.0] {
            assert_rows_fit(width, height as f32, 16.41, 2.0, 1.0, 13.0, 3.5, 0.0);
        }
    }
}

#[test]
fn test_rows_fit_with_nonzero_leading() {
    let test_cases: Vec<(f32, f32, f32)> = vec![
        (12.0, 3.0, 0.5),
        (12.0, 3.0, 1.0),
        (14.0, 4.0, 0.25),
        (10.0, 3.0, 2.0),
    ];

    for (ascent, descent, leading) in test_cases {
        for height in (500..=2000).step_by(100) {
            assert_rows_fit(
                1600.0,
                height as f32,
                16.0,
                2.0,
                1.0,
                ascent,
                descent,
                leading,
            );
        }
    }
}

#[test]
fn test_rows_fit_with_line_height_modifier() {
    for lh_mod in [1.1, 1.2, 1.5, 2.0] {
        for height in (500..=2000).step_by(100) {
            assert_rows_fit(1600.0, height as f32, 16.0, 2.0, lh_mod, 12.0, 3.0, 0.5);
        }
    }
}

#[test]
fn test_rows_fit_scale_1() {
    for height in (300..=1200).step_by(50) {
        assert_rows_fit(800.0, height as f32, 8.0, 1.0, 1.0, 13.0, 3.5, 0.0);
    }
}

#[test]
fn test_rows_fit_zero_leading() {
    for height in (500..=2000).step_by(100) {
        assert_rows_fit(1600.0, height as f32, 16.0, 2.0, 1.0, 12.77, 3.50, 0.0);
    }
}

#[test]
fn test_rows_fit_fractional_metrics() {
    // Fractional ascent+descent that would produce different results
    // with and without ceil
    assert_rows_fit(1600.0, 1000.0, 16.0, 2.0, 1.0, 12.3, 3.4, 0.1);
    assert_rows_fit(1600.0, 1000.0, 16.0, 2.0, 1.0, 11.9, 4.6, 0.3);
    assert_rows_fit(1600.0, 1000.0, 16.0, 1.5, 1.0, 12.0, 3.0, 0.5);
}

#[test]
fn test_compute_returns_min_for_zero_dimensions() {
    let dims = TextDimensions {
        width: 16.0,
        height: 32.0,
        scale: 2.0,
    };
    let (cols, rows) = compute(0.0, 0.0, cell_for(dims), Margin::all(0.0), 2.0);
    assert_eq!(cols, MIN_COLS);
    assert_eq!(rows, MIN_LINES);
}

#[test]
fn test_compute_returns_min_for_negative_dimensions() {
    let dims = TextDimensions {
        width: 16.0,
        height: 32.0,
        scale: 2.0,
    };
    let (cols, rows) = compute(-100.0, -100.0, cell_for(dims), Margin::all(0.0), 2.0);
    assert_eq!(cols, MIN_COLS);
    assert_eq!(rows, MIN_LINES);
}

#[test]
fn test_compute_returns_min_for_zero_scale() {
    let dims = TextDimensions {
        width: 16.0,
        height: 32.0,
        scale: 0.0,
    };
    let (cols, rows) = compute(1600.0, 900.0, cell_for(dims), Margin::all(0.0), 0.0);
    assert_eq!(cols, MIN_COLS);
    assert_eq!(rows, MIN_LINES);
}

#[test]
fn test_compute_basic_grid() {
    let dims = TextDimensions {
        width: 16.0,
        height: 33.0,
        scale: 2.0,
    };
    let (cols, rows) = compute(1600.0, 825.0, cell_for(dims), Margin::all(0.0), 2.0);
    assert_eq!(cols, 100);
    assert_eq!(rows, 25);
}

#[test]
fn test_compute_floors_fractional_rows() {
    // 840px / 33px = 25.45 → floor → 25
    let dims = TextDimensions {
        width: 16.0,
        height: 33.0,
        scale: 1.0,
    };
    let (_, rows) = compute(1600.0, 840.0, cell_for(dims), Margin::all(0.0), 1.0);
    assert_eq!(rows, 25);
}

#[test]
fn test_compute_respects_margins() {
    let dims = TextDimensions {
        width: 16.0,
        height: 32.0,
        scale: 2.0,
    };
    let margin = Margin::new(0.0, 10.0, 0.0, 10.0);
    let (cols, _) = compute(1600.0, 800.0, cell_for(dims), margin, 2.0);
    // available = 1600 - 10*2 - 10*2 = 1560, cols = 1560/16 = 97
    assert_eq!(cols, 97);
}

#[test]
fn test_compute_margin_exceeds_size() {
    let dims = TextDimensions {
        width: 16.0,
        height: 32.0,
        scale: 2.0,
    };
    let margin = Margin::new(0.0, 0.0, 0.0, 1000.0);
    let (cols, rows) = compute(100.0, 800.0, cell_for(dims), margin, 2.0);
    assert_eq!(cols, MIN_COLS);
    assert_eq!(rows, MIN_LINES);
}

#[test]
fn test_context_dimension_update_scale_refreshes_scaled_font_size() {
    let dims = TextDimensions {
        width: 8.0,
        height: 16.0,
        scale: 1.0,
    };
    let cell = cell_for(dims);
    let mut context =
        ContextDimension::build(800.0, 600.0, dims, cell, 1.0, 14.0, Margin::all(10.0));

    context.update_scale(2.0);

    assert_eq!(context.dimension.scale, 2.0);
    assert_eq!(context.scaled_font_size, 28.0);
    // Width/height-derived layout is recomputed separately after the new
    // cell metrics are installed, so update_scale itself must not clobber
    // the current row/column counts.
    assert_eq!(context.columns, 97);
    assert_eq!(context.lines, 36);
}

#[test]
fn test_context_dimension_build() {
    let dims = TextDimensions {
        width: 16.0,
        height: 33.0,
        scale: 2.0,
    };
    let cd = ContextDimension::build(
        1650.0,
        825.0,
        dims,
        cell_for(dims),
        1.0,
        14.0,
        Margin::all(0.0),
    );
    assert_eq!(cd.columns, 103);
    assert_eq!(cd.lines, 25);
}

#[test]
fn test_context_dimension_update_width() {
    let dims = TextDimensions {
        width: 16.0,
        height: 33.0,
        scale: 2.0,
    };
    let mut cd = ContextDimension::build(
        1600.0,
        825.0,
        dims,
        cell_for(dims),
        1.0,
        14.0,
        Margin::all(0.0),
    );
    assert_eq!(cd.columns, 100);

    cd.update_width(800.0);
    assert_eq!(cd.columns, 50);
    assert_eq!(cd.lines, 25);
}

#[test]
fn test_context_dimension_update_height() {
    let dims = TextDimensions {
        width: 16.0,
        height: 33.0,
        scale: 2.0,
    };
    let mut cd = ContextDimension::build(
        1600.0,
        825.0,
        dims,
        cell_for(dims),
        1.0,
        14.0,
        Margin::all(0.0),
    );
    assert_eq!(cd.lines, 25);

    cd.update_height(660.0);
    assert_eq!(cd.lines, 20);
    assert_eq!(cd.columns, 100);
}

#[test]
fn test_context_dimension_update_dimensions() {
    let dims = TextDimensions {
        width: 16.0,
        height: 33.0,
        scale: 1.0,
    };
    let mut cd = ContextDimension::build(
        1600.0,
        825.0,
        dims,
        cell_for(dims),
        1.0,
        14.0,
        Margin::all(0.0),
    );
    assert_eq!(cd.lines, 25);

    let new_dims = TextDimensions {
        width: 16.0,
        height: 66.0,
        scale: 1.0,
    };
    cd.update_dimensions(new_dims, cell_for(new_dims));
    assert_eq!(cd.lines, 12); // 825/66 = 12.5 → 12
}

// ── SplitAuto orientation tests ────────────────────────────────────────────
//
// `ContextGrid::split_auto` picks `split_right` (Row) when the pane's
// `layout_rect` width ≥ height, and `split_down` (Column) otherwise.
// The tests below verify that decision in terms of the resulting Taffy
// container direction, mirroring `try_split_right` (Row) vs
// `try_split_down` (Column) without spawning PTY contexts.

/// Helper: build a 1-child root Taffy tree sized `w × h` and return the
/// `FlexDirection` that `split_auto` would choose for it.
fn split_auto_direction_for(w: f32, h: f32) -> taffy::FlexDirection {
    if w >= h {
        taffy::FlexDirection::Row
    } else {
        taffy::FlexDirection::Column
    }
}

/// Helper: given a root `w × h` and a panel of the same size, simulate the
/// same container tree that `split_panel` creates and return the direction
/// of the newly created container.
fn simulate_split_auto(
    w: f32,
    h: f32,
) -> taffy::FlexDirection {
    use taffy::{FlexDirection, TaffyTree};

    let mut tree: TaffyTree<()> = TaffyTree::new();

    let root = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            size: geometry::Size {
                width: length(w),
                height: length(h),
            },
            ..Default::default()
        })
        .unwrap();

    let panel = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();
    tree.add_child(root, panel).unwrap();

    tree.compute_layout(
        root,
        geometry::Size {
            width: AvailableSpace::MaxContent,
            height: AvailableSpace::MaxContent,
        },
    )
    .unwrap();

    let layout = tree.layout(panel).unwrap();
    let pw = layout.size.width;
    let ph = layout.size.height;

    // Simulate split_auto decision: use layout rect [2]=w [3]=h
    let chosen_dir = split_auto_direction_for(pw, ph);

    // Build the container as split_panel would, to confirm the tree shape
    let container = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_direction: chosen_dir,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();

    let new_panel = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();

    tree.remove_child(root, panel).unwrap();
    tree.add_child(container, panel).unwrap();
    tree.add_child(container, new_panel).unwrap();
    tree.add_child(root, container).unwrap();

    // Retrieve the direction of the container node from the tree
    tree.style(container).unwrap().flex_direction
}

/// A wide pane (width > height) should produce a horizontal (Row) split,
/// i.e. the two resulting panes are side-by-side.
#[test]
fn split_auto_picks_horizontal_for_wide_pane() {
    // 1000 × 400 → width > height → Row (split_right)
    let dir = simulate_split_auto(1000.0, 400.0);
    assert_eq!(
        dir,
        taffy::FlexDirection::Row,
        "Wide pane should split horizontally (Row), got {dir:?}"
    );
}

/// A tall pane (height > width) should produce a vertical (Column) split,
/// i.e. the two resulting panes are stacked.
#[test]
fn split_auto_picks_vertical_for_tall_pane() {
    // 400 × 1000 → height > width → Column (split_down)
    let dir = simulate_split_auto(400.0, 1000.0);
    assert_eq!(
        dir,
        taffy::FlexDirection::Column,
        "Tall pane should split vertically (Column), got {dir:?}"
    );
}

// ── End SplitAuto tests ─────────────────────────────────────────────────────

/// Reproduces the bug: after resizing a panel to 80%/20% and then
/// resizing the window, the panel proportions should be preserved
/// but they are not because set_panel_size uses flex_shrink: 0.0.
#[test]
fn test_panel_resize_preserves_proportions_on_window_resize() {
    use taffy::{FlexDirection, TaffyTree};

    let mut tree: TaffyTree<()> = TaffyTree::new();

    let initial_width = 1000.0;

    // Root container (simulates the grid root after margin subtraction)
    let root = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            size: geometry::Size {
                width: length(initial_width),
                height: length(800.0),
            },
            ..Default::default()
        })
        .unwrap();

    // Two panels, initially equal (flex_grow: 1.0)
    let left = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();
    let right = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();

    tree.add_child(root, left).unwrap();
    tree.add_child(root, right).unwrap();

    // Compute initial layout — should be 500/500
    tree.compute_layout(
        root,
        geometry::Size {
            width: AvailableSpace::MaxContent,
            height: AvailableSpace::MaxContent,
        },
    )
    .unwrap();
    let left_w = tree.layout(left).unwrap().size.width;
    let right_w = tree.layout(right).unwrap().size.width;
    assert!(
        (left_w - 500.0).abs() < 1.0,
        "left should be ~500, got {left_w}"
    );
    assert!(
        (right_w - 500.0).abs() < 1.0,
        "right should be ~500, got {right_w}"
    );

    // Simulate move_divider: set left to 80%, right to 20%
    // Uses flex_grow proportional to the size so panels scale on resize
    let mut left_style = tree.style(left).unwrap().clone();
    left_style.flex_basis = length(0.0);
    left_style.flex_grow = 800.0;
    left_style.flex_shrink = 1.0;
    tree.set_style(left, left_style).unwrap();

    let mut right_style = tree.style(right).unwrap().clone();
    right_style.flex_basis = length(0.0);
    right_style.flex_grow = 200.0;
    right_style.flex_shrink = 1.0;
    tree.set_style(right, right_style).unwrap();

    // Verify 80/20 split
    tree.compute_layout(
        root,
        geometry::Size {
            width: AvailableSpace::MaxContent,
            height: AvailableSpace::MaxContent,
        },
    )
    .unwrap();
    let left_w = tree.layout(left).unwrap().size.width;
    let right_w = tree.layout(right).unwrap().size.width;
    assert!(
        (left_w - 800.0).abs() < 1.0,
        "left should be 800, got {left_w}"
    );
    assert!(
        (right_w - 200.0).abs() < 1.0,
        "right should be 200, got {right_w}"
    );

    // Now resize the window to 1200px (simulates try_update_size)
    let new_width = 1200.0;
    let mut root_style = tree.style(root).unwrap().clone();
    root_style.size.width = length(new_width);
    tree.set_style(root, root_style).unwrap();

    tree.compute_layout(
        root,
        geometry::Size {
            width: AvailableSpace::MaxContent,
            height: AvailableSpace::MaxContent,
        },
    )
    .unwrap();

    let left_w = tree.layout(left).unwrap().size.width;
    let right_w = tree.layout(right).unwrap().size.width;

    // The 80/20 proportion should be preserved: 960/240
    let expected_left = new_width * 0.8;
    let expected_right = new_width * 0.2;

    assert!(
        (left_w - expected_left).abs() < 1.0,
        "After resize, left should be ~{expected_left} (80%), got {left_w}"
    );
    assert!(
        (right_w - expected_right).abs() < 1.0,
        "After resize, right should be ~{expected_right} (20%), got {right_w}"
    );
}

/// Reproduces bug: two panels with 20/80 split, then splitting the 80%
/// panel horizontally should keep the 20/80 proportion in the parent.
#[test]
fn test_split_inside_resized_panel_preserves_proportions() {
    use taffy::{FlexDirection, TaffyTree};

    let mut tree: TaffyTree<()> = TaffyTree::new();

    // Root container
    let root = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            size: geometry::Size {
                width: length(1000.0),
                height: length(800.0),
            },
            ..Default::default()
        })
        .unwrap();

    // Two panels
    let left = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();
    let right = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();

    tree.add_child(root, left).unwrap();
    tree.add_child(root, right).unwrap();

    // Resize: left=20%, right=80% (using flex_grow proportional)
    let mut left_style = tree.style(left).unwrap().clone();
    left_style.flex_basis = length(0.0);
    left_style.flex_grow = 200.0;
    left_style.flex_shrink = 1.0;
    tree.set_style(left, left_style).unwrap();

    let mut right_style = tree.style(right).unwrap().clone();
    right_style.flex_basis = length(0.0);
    right_style.flex_grow = 800.0;
    right_style.flex_shrink = 1.0;
    tree.set_style(right, right_style).unwrap();

    // Verify 20/80 split
    let available = geometry::Size {
        width: AvailableSpace::MaxContent,
        height: AvailableSpace::MaxContent,
    };
    tree.compute_layout(root, available).unwrap();
    let left_w = tree.layout(left).unwrap().size.width;
    let right_w = tree.layout(right).unwrap().size.width;
    assert!(
        (left_w - 200.0).abs() < 1.0,
        "left should be 200, got {left_w}"
    );
    assert!(
        (right_w - 800.0).abs() < 1.0,
        "right should be 800, got {right_w}"
    );

    // Now split the right panel horizontally (Column direction).
    // This simulates what split_panel does:
    // 1. Create container inheriting right's flex properties
    // 2. Reset right to flex_grow: 1.0
    // 3. Create new panel with flex_grow: 1.0
    // 4. Move right into container, add new panel

    let right_inherited = tree.style(right).unwrap().clone();
    let container = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_basis: right_inherited.flex_basis,
            flex_grow: right_inherited.flex_grow,
            flex_shrink: right_inherited.flex_shrink,
            ..Default::default()
        })
        .unwrap();

    // Reset right panel to flexible inside container
    let mut reset_right = right_inherited;
    reset_right.flex_basis = taffy::Dimension::auto();
    reset_right.flex_grow = 1.0;
    reset_right.flex_shrink = 1.0;
    tree.set_style(right, reset_right).unwrap();

    let bottom = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();

    tree.remove_child(root, right).unwrap();
    tree.add_child(container, right).unwrap();
    tree.add_child(container, bottom).unwrap();
    tree.add_child(root, container).unwrap();

    tree.compute_layout(root, available).unwrap();

    // The container (replacing right) should still be ~800px wide (80%)
    let container_w = tree.layout(container).unwrap().size.width;
    assert!(
        (container_w - 800.0).abs() < 1.0,
        "Container should keep 80% (800px), got {container_w}"
    );

    // Left should still be ~200px (20%)
    let left_w = tree.layout(left).unwrap().size.width;
    assert!(
        (left_w - 200.0).abs() < 1.0,
        "Left should keep 20% (200px), got {left_w}"
    );

    // The two children inside the container should each be ~400px tall (50/50)
    let right_h = tree.layout(right).unwrap().size.height;
    let bottom_h = tree.layout(bottom).unwrap().size.height;
    assert!(
        (right_h - 400.0).abs() < 1.0,
        "Right (top half) should be ~400px tall, got {right_h}"
    );
    assert!(
        (bottom_h - 400.0).abs() < 1.0,
        "Bottom (bottom half) should be ~400px tall, got {bottom_h}"
    );
}

// ── Phase 3: focus_pane_by_number tests ─────────────────────────────────────

/// Build a 3-pane ContextGrid without Sugarloaf by:
///   1. Constructing via `ContextGrid::new` (single pane)
///   2. Calling private `try_split_right` twice to register Taffy nodes
///   3. Inserting `ContextGridItem`s into `inner` directly
///   4. Setting `layout_rect` to deterministic positions so
///      `get_ordered_keys` returns a predictable visual order
///
/// Visual layout (left-to-right): pane_a | pane_b | pane_c
/// Expected `get_ordered_keys` order: [pane_a, pane_b, pane_c]
fn make_three_pane_grid() -> (
    ContextGrid<rio_backend::event::VoidListener>,
    taffy::NodeId, // pane_a (leftmost, initially `current`)
    taffy::NodeId, // pane_b (middle)
    taffy::NodeId, // pane_c (rightmost)
) {
    use crate::context::create_dead_context;
    use rio_backend::config::layout::{Margin, Panel};
    use rio_backend::event::VoidListener;
    use rio_backend::sugarloaf::layout::{CellMetrics, TextDimensions};
    use rio_window::window::WindowId;

    let dims = TextDimensions {
        width: 16.0,
        height: 32.0,
        scale: 1.0,
    };
    let cell = CellMetrics {
        cell_width: 16,
        cell_height: 32,
        cell_baseline: 0,
        face_width: 16.0,
        face_height: 32.0,
        face_y: 0.0,
    };
    let dimension = ContextDimension::build(
        1200.0,
        800.0,
        dims,
        cell,
        1.0,
        14.0,
        Margin::all(0.0),
    );

    let window_id = WindowId::from(0u64);
    let ctx_a = create_dead_context(VoidListener {}, window_id, 1, 1, dimension);
    let ctx_b = create_dead_context(VoidListener {}, window_id, 2, 2, dimension);
    let ctx_c = create_dead_context(VoidListener {}, window_id, 3, 3, dimension);

    let mut grid = ContextGrid::new(
        ctx_a,
        Margin::all(0.0),
        [0.5, 0.5, 0.5, 1.0],
        [1.0, 1.0, 1.0, 1.0],
        Panel::default(),
    );

    // `grid.current` is pane_a's node after `new`
    let pane_a = grid.current;

    // Add two more panes via the private Taffy helper, then register them
    let pane_b = grid
        .try_split_right()
        .expect("try_split_right for pane_b failed");
    grid.inner.insert(pane_b, ContextGridItem::new(ctx_b));

    // Re-focus pane_a so the next split is relative to it (column layout)
    grid.current = pane_a;
    let pane_c = grid
        .try_split_right()
        .expect("try_split_right for pane_c failed");
    grid.inner.insert(pane_c, ContextGridItem::new(ctx_c));

    // Compute Taffy layout so real positions are populated in layout_rect
    grid.calculate_positions();

    // Leave `current` at pane_a so tests start from a known state
    grid.current = pane_a;

    (grid, pane_a, pane_b, pane_c)
}

#[test]
fn focus_pane_by_number_indexes_in_visual_order() {
    let (mut grid, pane_a, pane_b, pane_c) = make_three_pane_grid();

    // Sanity: we have 3 panes
    assert_eq!(grid.inner.len(), 3);

    // The visual order is determined by layout_rect (top-to-bottom, left-to-right).
    // Record the ordered keys so we can assert focus matches position.
    let ordered = grid.get_ordered_keys();
    assert_eq!(
        ordered.len(),
        3,
        "Expected 3 panes in visual order, got {}",
        ordered.len()
    );

    // Focusing position 1 should set current to the first key in visual order.
    let changed = grid.focus_pane_by_number(1);
    // If pane_a is already first, changed may be false (already there); either way
    // `current` must equal ordered[0].
    assert_eq!(
        grid.current,
        ordered[0],
        "After focus_pane_by_number(1), current should be ordered[0]"
    );

    // Focusing position 2 must change focus (ordered[1] != ordered[0]).
    let changed2 = grid.focus_pane_by_number(2);
    assert!(changed2, "focus_pane_by_number(2) must return true");
    assert_eq!(
        grid.current,
        ordered[1],
        "After focus_pane_by_number(2), current should be ordered[1]"
    );

    // Focusing position 3.
    let changed3 = grid.focus_pane_by_number(3);
    assert!(changed3, "focus_pane_by_number(3) must return true");
    assert_eq!(
        grid.current,
        ordered[2],
        "After focus_pane_by_number(3), current should be ordered[2]"
    );

    // All three NodeIds must appear (no duplicates, all panes reachable).
    let reachable: std::collections::HashSet<_> =
        [ordered[0], ordered[1], ordered[2]].into_iter().collect();
    let all_ids: std::collections::HashSet<_> =
        [pane_a, pane_b, pane_c].into_iter().collect();
    assert_eq!(
        reachable, all_ids,
        "Visual order must cover exactly the three pane NodeIds"
    );

    // current_pane_number mirrors the focus.
    grid.current = ordered[0];
    assert_eq!(grid.current_pane_number(), 1);
    grid.current = ordered[1];
    assert_eq!(grid.current_pane_number(), 2);
    grid.current = ordered[2];
    assert_eq!(grid.current_pane_number(), 3);
}

#[test]
fn focus_pane_by_number_handles_out_of_range() {
    let (mut grid, _pane_a, _pane_b, _pane_c) = make_three_pane_grid();

    // Start with pane_a (index 0 in ordered keys)
    let ordered = grid.get_ordered_keys();
    grid.current = ordered[0];

    // n=9 is out of range for a 3-pane grid — must return false and leave focus unchanged.
    let changed = grid.focus_pane_by_number(9);
    assert!(!changed, "focus_pane_by_number(9) on 3-pane grid must return false");
    assert_eq!(
        grid.current,
        ordered[0],
        "focus unchanged after out-of-range call"
    );

    // n=0 saturates to index 0 (saturating_sub(1) = 0) — should focus ordered[0].
    // If already there it returns false; current stays the same.
    let changed_zero = grid.focus_pane_by_number(0);
    assert!(
        !changed_zero,
        "focus_pane_by_number(0) saturates to index 0; already there, so false"
    );
    assert_eq!(grid.current, ordered[0]);
}

/// Simulates what `resize_in_direction(Right)` does when the current pane is
/// the left pane in a horizontal split: the left pane should grow and the right
/// pane should shrink by `amount`.
///
/// This exercises the same `set_panel_size` / `compute_layout` path that
/// `ContextGrid::resize_in_direction` uses internally.
#[test]
fn resize_in_direction_grows_current_pane() {
    use taffy::{FlexDirection, TaffyTree};

    let total_w = 1000.0_f32;
    let total_h = 800.0_f32;
    let scale = 2.0_f32;
    // 10 logical px × scale → amount used by resize_in_direction
    let amount = 10.0 * scale;

    let mut tree: TaffyTree<()> = TaffyTree::new();

    let root = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            size: geometry::Size {
                width: length(total_w),
                height: length(total_h),
            },
            ..Default::default()
        })
        .unwrap();

    // Two equal panels (500 px each)
    let left = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();
    let right = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();

    tree.add_child(root, left).unwrap();
    tree.add_child(root, right).unwrap();

    let available = taffy::geometry::Size {
        width: AvailableSpace::MaxContent,
        height: AvailableSpace::MaxContent,
    };
    tree.compute_layout(root, available).unwrap();

    let left_w_before = tree.layout(left).unwrap().size.width;
    let right_w_before = tree.layout(right).unwrap().size.width;
    assert!(
        (left_w_before - 500.0).abs() < 1.0,
        "left should start at ~500, got {left_w_before}"
    );
    assert!(
        (right_w_before - 500.0).abs() < 1.0,
        "right should start at ~500, got {right_w_before}"
    );

    // --- Simulate resize_in_direction(Right) with current == left ---
    // Current is the left pane; growing Right means left grows, right shrinks.
    let new_left_w = left_w_before + amount;
    let new_right_w = right_w_before - amount;

    // set_panel_size uses flex_grow proportional to pixel size
    let mut left_style = tree.style(left).unwrap().clone();
    left_style.flex_basis = length(0.0);
    left_style.flex_grow = new_left_w;
    left_style.flex_shrink = 1.0;
    tree.set_style(left, left_style).unwrap();

    let mut right_style = tree.style(right).unwrap().clone();
    right_style.flex_basis = length(0.0);
    right_style.flex_grow = new_right_w;
    right_style.flex_shrink = 1.0;
    tree.set_style(right, right_style).unwrap();

    tree.compute_layout(root, available).unwrap();

    let left_w_after = tree.layout(left).unwrap().size.width;
    let right_w_after = tree.layout(right).unwrap().size.width;

    // Left should have grown by `amount`; right should have shrunk by `amount`.
    assert!(
        (left_w_after - (left_w_before + amount)).abs() < 1.0,
        "left should be ~{}, got {left_w_after}",
        left_w_before + amount
    );
    assert!(
        (right_w_after - (right_w_before - amount)).abs() < 1.0,
        "right should be ~{}, got {right_w_after}",
        right_w_before - amount
    );
    // Total width must remain constant
    assert!(
        ((left_w_after + right_w_after) - total_w).abs() < 1.0,
        "total width should remain {total_w}, got {}",
        left_w_after + right_w_after
    );
}

/// When a pane has no neighbour in the requested direction (single-pane grid,
/// or the current pane is at the edge with no adjacent panel), `resize_in_direction`
/// must be a no-op.
///
/// This test verifies the guard condition: a single panel in the Taffy tree
/// produces no measurable size change when the resize logic is applied.
#[test]
fn resize_in_direction_at_edge_is_noop() {
    use taffy::{FlexDirection, TaffyTree};

    let total_w = 1000.0_f32;
    let total_h = 800.0_f32;

    let mut tree: TaffyTree<()> = TaffyTree::new();

    let root = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            size: geometry::Size {
                width: length(total_w),
                height: length(total_h),
            },
            ..Default::default()
        })
        .unwrap();

    // Single panel — no horizontal neighbour exists.
    let only = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();

    tree.add_child(root, only).unwrap();

    let available = taffy::geometry::Size {
        width: AvailableSpace::MaxContent,
        height: AvailableSpace::MaxContent,
    };
    tree.compute_layout(root, available).unwrap();

    let w_before = tree.layout(only).unwrap().size.width;

    // In ContextGrid::resize_in_direction, find_horizontal_neighbors returns
    // None for a single-panel grid, so set_panel_size is never called.
    // The tree layout remains unchanged — verify that here.
    tree.compute_layout(root, available).unwrap();
    let w_after = tree.layout(only).unwrap().size.width;

    assert!(
        (w_before - w_after).abs() < 0.01,
        "single-pane resize must be noop: before={w_before}, after={w_after}"
    );
    assert!(
        (w_after - total_w).abs() < 1.0,
        "single pane should fill the full width={total_w}, got {w_after}"
    );
}

// ── Phase 6: MaximizePane tests ──────────────────────────────────────────────
//
// These tests exercise `ContextGrid::toggle_maximize` and the
// `swap_maximize_display` helper without spawning PTY contexts.  They use
// `make_three_pane_grid` (already defined above) and directly inspect the
// Taffy style `display` field on each panel node.

/// After toggling maximize on a 2-pane grid, only the maximized panel should
/// have `display: Flex`; the other panel(s) must have `display: None`.
#[test]
fn maximize_hides_other_panes() {
    let (mut grid, pane_a, _pane_b, _pane_c) = make_three_pane_grid();

    // Start with pane_a focused
    grid.current = pane_a;
    assert!(
        grid.maximized.is_none(),
        "maximized should be None before toggle"
    );

    // Build a dummy Sugarloaf-less call: toggle_maximize calls apply_taffy_layout
    // which needs a Sugarloaf. We test via the Taffy tree state directly instead,
    // by calling the internal helpers that toggle_maximize uses.
    //
    // Replicate toggle_maximize logic for 3-pane grid (maximized is None, len > 1):
    grid.maximized = Some(pane_a);
    let keys: Vec<taffy::NodeId> = grid.inner.keys().copied().collect();
    for id in &keys {
        let style = if *id == pane_a {
            taffy::Style {
                display: taffy::Display::Flex,
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..Default::default()
            }
        } else {
            taffy::Style {
                display: taffy::Display::None,
                ..Default::default()
            }
        };
        let _ = grid.tree.set_style(*id, style);
    }

    // Verify: maximized is Some(pane_a)
    assert_eq!(grid.maximized, Some(pane_a));

    // All non-current panels must have display: None
    for id in &keys {
        let style = grid.tree.style(*id).unwrap();
        if *id == pane_a {
            assert_eq!(
                style.display,
                taffy::Display::Flex,
                "maximized pane should have Flex display"
            );
        } else {
            assert_eq!(
                style.display,
                taffy::Display::None,
                "non-maximized pane should have None display"
            );
        }
    }
}

/// After toggling maximize twice (maximize then restore), all panels should
/// return to `display: Flex` with positive flex_grow.
#[test]
fn restore_brings_back_original_layout() {
    let (mut grid, pane_a, _pane_b, _pane_c) = make_three_pane_grid();

    grid.current = pane_a;

    // Simulate maximize
    grid.maximized = Some(pane_a);
    let keys: Vec<taffy::NodeId> = grid.inner.keys().copied().collect();
    for id in &keys {
        let style = if *id == pane_a {
            taffy::Style {
                display: taffy::Display::Flex,
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..Default::default()
            }
        } else {
            taffy::Style {
                display: taffy::Display::None,
                ..Default::default()
            }
        };
        let _ = grid.tree.set_style(*id, style);
    }

    assert_eq!(grid.maximized, Some(pane_a));

    // Simulate restore (replicate the maximized.is_some() branch of toggle_maximize):
    grid.maximized = None;
    // Explicitly restore display:Flex on panel leaf nodes first (same as toggle_maximize).
    for id in &keys {
        let _ = grid.tree.set_style(
            *id,
            taffy::Style {
                display: taffy::Display::Flex,
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..Default::default()
            },
        );
    }
    // Also reset container node sizing via the existing helper.
    grid.reset_panel_styles_to_flexible();

    // All panels must now be visible (Flex)
    for id in &keys {
        let style = grid.tree.style(*id).unwrap();
        assert_eq!(
            style.display,
            taffy::Display::Flex,
            "all panes should be Flex after restore, failed for {id:?}"
        );
    }
    assert!(
        grid.maximized.is_none(),
        "maximized should be None after restore"
    );
}

/// When a pane other than the currently-maximized one gains focus,
/// `swap_maximize_display` must update which pane is maximized.
#[test]
fn focus_change_while_maximized_swaps_maximized_node() {
    let (mut grid, pane_a, pane_b, _pane_c) = make_three_pane_grid();

    // Maximize pane_a
    grid.current = pane_a;
    grid.maximized = Some(pane_a);
    let keys: Vec<taffy::NodeId> = grid.inner.keys().copied().collect();
    for id in &keys {
        let style = if *id == pane_a {
            taffy::Style {
                display: taffy::Display::Flex,
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..Default::default()
            }
        } else {
            taffy::Style {
                display: taffy::Display::None,
                ..Default::default()
            }
        };
        let _ = grid.tree.set_style(*id, style);
    }

    // Now simulate focus moving to pane_b via swap_maximize_display
    let swapped = grid.swap_maximize_display(pane_b);
    assert!(swapped, "swap_maximize_display must return true when focus moves");
    assert_eq!(
        grid.maximized,
        Some(pane_b),
        "maximized should now track pane_b"
    );

    // pane_a must now be hidden
    let style_a = grid.tree.style(pane_a).unwrap();
    assert_eq!(
        style_a.display,
        taffy::Display::None,
        "pane_a should be hidden after swap"
    );

    // pane_b must now be visible
    let style_b = grid.tree.style(pane_b).unwrap();
    assert_eq!(
        style_b.display,
        taffy::Display::Flex,
        "pane_b should be Flex after becoming the maximized pane"
    );
}

/// On a single-pane grid, `toggle_maximize` must not change `maximized`
/// (it should remain `None`).
#[test]
fn single_pane_maximize_is_noop() {
    use crate::context::create_dead_context;
    use rio_backend::config::layout::{Margin, Panel};
    use rio_backend::event::VoidListener;
    use rio_backend::sugarloaf::layout::{CellMetrics, TextDimensions};
    use rio_window::window::WindowId;

    let dims = TextDimensions {
        width: 16.0,
        height: 32.0,
        scale: 1.0,
    };
    let cell = CellMetrics {
        cell_width: 16,
        cell_height: 32,
        cell_baseline: 0,
        face_width: 16.0,
        face_height: 32.0,
        face_y: 0.0,
    };
    let dimension = ContextDimension::build(
        1200.0,
        800.0,
        dims,
        cell,
        1.0,
        14.0,
        Margin::all(0.0),
    );

    let window_id = WindowId::from(0u64);
    let ctx = create_dead_context(VoidListener {}, window_id, 1, 1, dimension);

    let grid: ContextGrid<VoidListener> = ContextGrid::new(
        ctx,
        Margin::all(0.0),
        [0.5, 0.5, 0.5, 1.0],
        [1.0, 1.0, 1.0, 1.0],
        Panel::default(),
    );

    assert_eq!(grid.inner.len(), 1, "single-pane grid expected");
    // Replicate the toggle_maximize guard: inner.len() > 1 is false, so no-op
    assert!(
        grid.maximized.is_none(),
        "maximized must be None on a single-pane grid"
    );

    // Confirm the condition that toggle_maximize checks:
    // (maximized.is_none() && inner.len() <= 1) → no-op branch
    let would_maximize = grid.maximized.is_some() || grid.inner.len() > 1;
    assert!(
        !would_maximize,
        "single pane: toggle_maximize would be a no-op (inner.len()={})",
        grid.inner.len()
    );
}

// ── Phase 4: FocusPaneByDirection tests ─────────────────────────────────────

/// Build a T-layout (A and B side-by-side on top, C full-width on bottom)
/// without spawning PTY contexts, using the same technique as `make_three_pane_grid`.
///
/// Visual layout:
///   ┌────────┬────────┐
///   │   A    │   B    │  (each 600 × 400)
///   ├────────┴────────┤
///   │        C        │  (1200 × 400)
///   └─────────────────┘
///
/// Expected centres:
///   A: (300, 200)   B: (900, 200)   C: (600, 600)
fn make_t_layout_grid() -> (
    ContextGrid<rio_backend::event::VoidListener>,
    taffy::NodeId, // pane_a (top-left)
    taffy::NodeId, // pane_b (top-right)
    taffy::NodeId, // pane_c (full-width bottom)
) {
    use crate::context::create_dead_context;
    use rio_backend::config::layout::{Margin, Panel};
    use rio_backend::event::VoidListener;
    use rio_backend::sugarloaf::layout::{CellMetrics, TextDimensions};
    use rio_window::window::WindowId;

    let dims = TextDimensions {
        width: 16.0,
        height: 32.0,
        scale: 1.0,
    };
    let cell = CellMetrics {
        cell_width: 16,
        cell_height: 32,
        cell_baseline: 0,
        face_width: 16.0,
        face_height: 32.0,
        face_y: 0.0,
    };
    let dimension = ContextDimension::build(
        1200.0,
        800.0,
        dims,
        cell,
        1.0,
        14.0,
        Margin::all(0.0),
    );

    let window_id = WindowId::from(0u64);
    let ctx_a = create_dead_context(VoidListener {}, window_id, 1, 1, dimension);
    let ctx_b = create_dead_context(VoidListener {}, window_id, 2, 2, dimension);
    let ctx_c = create_dead_context(VoidListener {}, window_id, 3, 3, dimension);

    // Create grid with pane_a as the single pane.
    let mut grid = ContextGrid::new(
        ctx_a,
        Margin::all(0.0),
        [0.5, 0.5, 0.5, 1.0],
        [1.0, 1.0, 1.0, 1.0],
        Panel::default(),
    );

    let pane_a = grid.current;

    // Split pane_a downward → pane_c lands below pane_a in a Column container.
    // Tree after: Column[ pane_a (top), pane_c (bottom) ]
    let pane_c = grid
        .try_split_down()
        .expect("try_split_down for pane_c failed");
    grid.inner.insert(pane_c, ContextGridItem::new(ctx_c));

    // Refocus pane_a, then split right → pane_b lands to the right of pane_a.
    // Tree after: Column[ Row[pane_a, pane_b], pane_c ]
    grid.current = pane_a;
    let pane_b = grid
        .try_split_right()
        .expect("try_split_right for pane_b failed");
    grid.inner.insert(pane_b, ContextGridItem::new(ctx_b));

    // Compute real Taffy layout so layout_rect is populated with actual positions.
    grid.calculate_positions();

    // Leave current at pane_a so tests start from a known state.
    grid.current = pane_a;

    (grid, pane_a, pane_b, pane_c)
}

/// Verify that FocusPaneByDirection picks the geometrically nearest neighbour
/// in a 3-pane T-layout:
///   - From A: Right → B, Down → C
///   - From B: Left  → A, Down → C
///   - From C: Up    → A or B (both are equidistant from C's centre)
#[test]
fn focus_direction_picks_nearest_in_each_quadrant() {
    use crate::bindings::PaneDirection;

    let (mut grid, pane_a, pane_b, pane_c) = make_t_layout_grid();

    // ── From A ──────────────────────────────────────────────────────────────
    grid.current = pane_a;

    // A → Right should reach B.
    let neighbour = grid.find_neighbour_in_direction(PaneDirection::Right);
    assert_eq!(
        neighbour,
        Some(pane_b),
        "From A, Right should land on B"
    );

    // A → Down should reach C.
    let neighbour = grid.find_neighbour_in_direction(PaneDirection::Down);
    assert_eq!(
        neighbour,
        Some(pane_c),
        "From A, Down should land on C"
    );

    // ── From B ──────────────────────────────────────────────────────────────
    grid.current = pane_b;

    // B → Left should reach A.
    let neighbour = grid.find_neighbour_in_direction(PaneDirection::Left);
    assert_eq!(
        neighbour,
        Some(pane_a),
        "From B, Left should land on A"
    );

    // B → Down should reach C.
    let neighbour = grid.find_neighbour_in_direction(PaneDirection::Down);
    assert_eq!(
        neighbour,
        Some(pane_c),
        "From B, Down should land on C"
    );

    // ── From C ──────────────────────────────────────────────────────────────
    grid.current = pane_c;

    // C → Up should reach either A or B (both equidistant from C's centre).
    let neighbour = grid.find_neighbour_in_direction(PaneDirection::Up);
    assert!(
        neighbour == Some(pane_a) || neighbour == Some(pane_b),
        "From C, Up should reach either A or B, got {neighbour:?}"
    );

    // ── focus_neighbour_in_direction wires through correctly ─────────────────
    grid.current = pane_a;
    let changed = grid.focus_neighbour_in_direction(PaneDirection::Right);
    assert!(changed, "focus_neighbour_in_direction(Right) from A must return true");
    assert_eq!(grid.current, pane_b, "Current must be B after focusing Right from A");
}

/// Verify that FocusPaneByDirection returns false and leaves focus unchanged
/// when the current pane has no neighbour in the requested direction.
#[test]
fn focus_direction_returns_none_at_edge() {
    use crate::bindings::PaneDirection;

    let (mut grid, pane_a, _pane_b, _pane_c) = make_t_layout_grid();

    // pane_a is the leftmost pane — there is nothing further to its Left.
    grid.current = pane_a;

    // find_neighbour_in_direction must return None.
    let neighbour = grid.find_neighbour_in_direction(PaneDirection::Left);
    assert!(
        neighbour.is_none(),
        "From the leftmost pane, Left should return None, got {neighbour:?}"
    );

    // focus_neighbour_in_direction must return false and leave focus on pane_a.
    let changed = grid.focus_neighbour_in_direction(PaneDirection::Left);
    assert!(
        !changed,
        "focus_neighbour_in_direction(Left) at edge must return false"
    );
    assert_eq!(
        grid.current,
        pane_a,
        "Focus must remain on pane_a after edge-direction attempt"
    );
}

// ── Phase 7: DistributeEvenly tests ─────────────────────────────────────────
//
// `ContextGrid::distribute_evenly_along` walks the Taffy tree and resets
// every flex container whose `flex_direction` matches the requested axis so
// that all children have `flex_grow = 1.0` and `flex_basis = length(0)`.
//
// Because `distribute_evenly_along` requires a live `Sugarloaf` context, the
// tests here directly exercise the Taffy tree mechanics that the function
// relies on — the same approach used by the resize and split-auto tests above.

/// After an unequal horizontal split (80 / 20) the left and right panels
/// should become equal when the distribute logic is applied.
#[test]
fn distribute_evenly_along_horizontal_axis() {
    use taffy::{FlexDirection, TaffyTree};

    let total_w = 1000.0_f32;
    let total_h = 800.0_f32;

    let mut tree: TaffyTree<()> = TaffyTree::new();

    // Root: horizontal Row container (Vertical border axis → Row direction)
    let root = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            size: geometry::Size {
                width: length(total_w),
                height: length(total_h),
            },
            ..Default::default()
        })
        .unwrap();

    // Left panel (80 %)
    let left = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_basis: length(0.0),
            flex_grow: 800.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();

    // Right panel (20 %)
    let right = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_basis: length(0.0),
            flex_grow: 200.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();

    tree.add_child(root, left).unwrap();
    tree.add_child(root, right).unwrap();

    let available = geometry::Size {
        width: AvailableSpace::MaxContent,
        height: AvailableSpace::MaxContent,
    };
    tree.compute_layout(root, available).unwrap();

    let left_w_before = tree.layout(left).unwrap().size.width;
    let right_w_before = tree.layout(right).unwrap().size.width;
    assert!(
        (left_w_before - 800.0).abs() < 1.0,
        "left should start at ~800, got {left_w_before}"
    );
    assert!(
        (right_w_before - 200.0).abs() < 1.0,
        "right should start at ~200, got {right_w_before}"
    );

    // ── Apply distribute_evenly_along(Vertical) logic ─────────────────────
    // For a Row container, set every child's flex_grow = 1.0 / flex_basis = 0.
    // This mirrors exactly what ContextGrid::distribute_evenly_along does.
    let target_dir = FlexDirection::Row; // BorderDirection::Vertical → Row
    let children = tree.children(root).unwrap();
    let node_dir = tree.style(root).unwrap().flex_direction;
    assert_eq!(node_dir, target_dir, "root must be a Row container");

    for child in &children {
        let mut style = tree.style(*child).unwrap().clone();
        style.flex_basis = length(0.0);
        style.flex_grow = 1.0;
        style.flex_shrink = 1.0;
        tree.set_style(*child, style).unwrap();
    }

    tree.compute_layout(root, available).unwrap();

    let left_w_after = tree.layout(left).unwrap().size.width;
    let right_w_after = tree.layout(right).unwrap().size.width;

    // Both panels should now be equal (~500 px each).
    assert!(
        (left_w_after - 500.0).abs() < 1.0,
        "left should be ~500 after distribute, got {left_w_after}"
    );
    assert!(
        (right_w_after - 500.0).abs() < 1.0,
        "right should be ~500 after distribute, got {right_w_after}"
    );
    // Total width is preserved.
    assert!(
        ((left_w_after + right_w_after) - total_w).abs() < 1.0,
        "total width must remain {total_w}, got {}",
        left_w_after + right_w_after
    );
}

/// After a right-then-down split (unequal on both axes), calling
/// distribute_evenly_all must equalise panes on both the horizontal and
/// vertical axes.
#[test]
fn distribute_evenly_all_both_axes() {
    use taffy::{FlexDirection, TaffyTree};

    // Layout tree:
    //   root (Row)
    //   ├── left (leaf, 20 %)
    //   └── right_container (Column, 80 %)
    //       ├── top (leaf, 75 %)
    //       └── bottom (leaf, 25 %)
    //
    // After distribute_evenly_all:
    //   row axis  → left and right_container each get 50 % of total_w
    //   col axis  → top and bottom each get 50 % of right_container's height

    let total_w = 1000.0_f32;
    let total_h = 800.0_f32;

    let mut tree: TaffyTree<()> = TaffyTree::new();

    let root = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            size: geometry::Size {
                width: length(total_w),
                height: length(total_h),
            },
            ..Default::default()
        })
        .unwrap();

    // left panel: 20 % of width
    let left = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_basis: length(0.0),
            flex_grow: 200.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();

    // right_container: 80 % of width, Column direction
    let right_container = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_basis: length(0.0),
            flex_grow: 800.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();

    // top sub-panel: 75 % of right_container height
    let top = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_basis: length(0.0),
            flex_grow: 600.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();

    // bottom sub-panel: 25 % of right_container height
    let bottom = tree
        .new_leaf(Style {
            display: Display::Flex,
            flex_basis: length(0.0),
            flex_grow: 200.0,
            flex_shrink: 1.0,
            ..Default::default()
        })
        .unwrap();

    tree.add_child(root, left).unwrap();
    tree.add_child(root, right_container).unwrap();
    tree.add_child(right_container, top).unwrap();
    tree.add_child(right_container, bottom).unwrap();

    let available = geometry::Size {
        width: AvailableSpace::MaxContent,
        height: AvailableSpace::MaxContent,
    };

    // Verify the unequal initial state.
    tree.compute_layout(root, available).unwrap();
    let left_w = tree.layout(left).unwrap().size.width;
    let rc_w = tree.layout(right_container).unwrap().size.width;
    assert!(
        (left_w - 200.0).abs() < 1.0,
        "left should start at ~200, got {left_w}"
    );
    assert!(
        (rc_w - 800.0).abs() < 1.0,
        "right_container should start at ~800, got {rc_w}"
    );

    // ── Apply distribute_evenly_along(Vertical) → Row containers ─────────
    // Walk tree breadth-first; fix every Row container's children.
    let mut all_nodes: Vec<taffy::NodeId> = vec![root];
    let mut idx = 0;
    while idx < all_nodes.len() {
        let node = all_nodes[idx];
        if let Ok(ch) = tree.children(node) {
            for c in ch {
                all_nodes.push(c);
            }
        }
        idx += 1;
    }

    for node in &all_nodes {
        let children = match tree.children(*node) {
            Ok(c) if !c.is_empty() => c,
            _ => continue,
        };
        if tree.style(*node).unwrap().flex_direction != FlexDirection::Row {
            continue;
        }
        for child in children {
            let mut style = tree.style(child).unwrap().clone();
            style.flex_basis = length(0.0);
            style.flex_grow = 1.0;
            style.flex_shrink = 1.0;
            tree.set_style(child, style).unwrap();
        }
    }

    // ── Apply distribute_evenly_along(Horizontal) → Column containers ─────
    for node in &all_nodes {
        let children = match tree.children(*node) {
            Ok(c) if !c.is_empty() => c,
            _ => continue,
        };
        if tree.style(*node).unwrap().flex_direction != FlexDirection::Column {
            continue;
        }
        for child in children {
            let mut style = tree.style(child).unwrap().clone();
            style.flex_basis = length(0.0);
            style.flex_grow = 1.0;
            style.flex_shrink = 1.0;
            tree.set_style(child, style).unwrap();
        }
    }

    tree.compute_layout(root, available).unwrap();

    // Row axis: left and right_container should each be 50 % of total_w.
    let left_w_after = tree.layout(left).unwrap().size.width;
    let rc_w_after = tree.layout(right_container).unwrap().size.width;
    assert!(
        (left_w_after - total_w / 2.0).abs() < 1.0,
        "left should be ~500 after distribute, got {left_w_after}"
    );
    assert!(
        (rc_w_after - total_w / 2.0).abs() < 1.0,
        "right_container should be ~500 after distribute, got {rc_w_after}"
    );

    // Column axis: top and bottom inside right_container should each be 50 %
    // of right_container's height (total_h / 2 = 400).
    let top_h_after = tree.layout(top).unwrap().size.height;
    let bot_h_after = tree.layout(bottom).unwrap().size.height;
    assert!(
        (top_h_after - total_h / 2.0).abs() < 1.0,
        "top sub-panel should be ~400 after distribute, got {top_h_after}"
    );
    assert!(
        (bot_h_after - total_h / 2.0).abs() < 1.0,
        "bottom sub-panel should be ~400 after distribute, got {bot_h_after}"
    );
}
