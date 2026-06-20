use taffy::NodeId;

/// Which triangle of a drop-target pane the cursor is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneDropQuadrant {
    Left,
    Top,
    Right,
    Bottom,
}

/// Active pane drag state — set when the user begins dragging a pane titlebar.
#[derive(Debug, Clone)]
pub struct PaneDragState {
    /// The NodeId of the pane being dragged.
    pub source_node: NodeId,
    /// Current cursor position in logical (unscaled) coordinates.
    pub cursor: (f32, f32),
}

/// Pure function: given a target pane's rect [x, y, w, h] in logical coordinates
/// and a cursor position, return which quadrant the cursor falls in.
///
/// The pane is split into 4 triangles by its diagonals:
///   - If |dx/w| > |dy/h|  → Left or Right
///   - Else                  → Top or Bottom
pub fn quadrant_at(rect: [f32; 4], cursor: (f32, f32)) -> PaneDropQuadrant {
    let (x, y, w, h) = (rect[0], rect[1], rect[2], rect[3]);
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    let dx = cursor.0 - cx;
    let dy = cursor.1 - cy;
    // Normalise by half-dimensions to make the triangles aspect-ratio-correct
    let nx = if w > 0.0 { dx / (w / 2.0) } else { dx };
    let ny = if h > 0.0 { dy / (h / 2.0) } else { dy };
    if nx.abs() >= ny.abs() {
        if nx >= 0.0 {
            PaneDropQuadrant::Right
        } else {
            PaneDropQuadrant::Left
        }
    } else if ny >= 0.0 {
        PaneDropQuadrant::Bottom
    } else {
        PaneDropQuadrant::Top
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECT: [f32; 4] = [0.0, 0.0, 100.0, 60.0];

    #[test]
    fn quadrant_at_right_triangle_returns_right() {
        // Well into the right triangle
        assert_eq!(quadrant_at(RECT, (90.0, 30.0)), PaneDropQuadrant::Right);
    }

    #[test]
    fn quadrant_at_left_triangle_returns_left() {
        assert_eq!(quadrant_at(RECT, (5.0, 30.0)), PaneDropQuadrant::Left);
    }

    #[test]
    fn quadrant_at_top_triangle_returns_top() {
        assert_eq!(quadrant_at(RECT, (50.0, 5.0)), PaneDropQuadrant::Top);
    }

    #[test]
    fn quadrant_at_bottom_triangle_returns_bottom() {
        assert_eq!(quadrant_at(RECT, (50.0, 55.0)), PaneDropQuadrant::Bottom);
    }

    #[test]
    fn quadrant_at_center_left_of_center_returns_left() {
        // Cursor exactly at left edge
        assert_eq!(quadrant_at(RECT, (0.0, 30.0)), PaneDropQuadrant::Left);
    }
}
