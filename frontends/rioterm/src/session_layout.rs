use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Returns the path used to save the current session layout.
///
/// Uses `$XDG_DATA_HOME/rio/session.rio-layout.json` on Linux/macOS,
/// or `%APPDATA%\rio\session.rio-layout.json` on Windows.
/// Falls back to `~/.rio/session.rio-layout.json` when the standard
/// directory cannot be determined.
pub fn get_session_layout_save_path() -> PathBuf {
    get_session_layout_default_path()
}

/// Returns the path used to open a session layout (currently the same as
/// the save path — no interactive file picker is implemented yet).
pub fn get_session_layout_open_path() -> PathBuf {
    get_session_layout_default_path()
}

fn get_session_layout_default_path() -> PathBuf {
    // Honour XDG / platform conventions when possible.
    if let Some(data_dir) = dirs::data_dir() {
        return data_dir.join("rio").join("session.rio-layout.json");
    }
    // Fallback: ~/.rio/session.rio-layout.json
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".rio").join("session.rio-layout.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLayoutFile {
    pub version: u32,
    pub session_title: Option<String>,
    pub tree: SessionNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionNode {
    Pane {
        working_dir: Option<String>,
        profile: Option<String>,
    },
    Split {
        direction: SplitDir,
        /// Fraction [0,1] of the total dimension allocated to `left`.
        ratio: f32,
        left: Box<SessionNode>,
        right: Box<SessionNode>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitDir {
    Horizontal, // FlexDirection::Row (side-by-side)
    Vertical,   // FlexDirection::Column (top-bottom)
}

impl SessionLayoutFile {
    pub const CURRENT_VERSION: u32 = 1;

    pub fn new(tree: SessionNode, title: Option<String>) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            session_title: title,
            tree,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_preserves_tree_structure() {
        let layout = SessionLayoutFile {
            version: 1,
            session_title: Some("test".into()),
            tree: SessionNode::Split {
                direction: SplitDir::Horizontal,
                ratio: 0.5,
                left: Box::new(SessionNode::Pane {
                    working_dir: Some("/tmp".into()),
                    profile: None,
                }),
                right: Box::new(SessionNode::Pane {
                    working_dir: None,
                    profile: None,
                }),
            },
        };
        let json = serde_json::to_string(&layout).unwrap();
        let back: SessionLayoutFile = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version, 1);
        assert!(matches!(back.tree, SessionNode::Split { .. }));
    }

    #[test]
    fn version_mismatch_is_detectable() {
        let json = r#"{"version":99,"session_title":null,"tree":{"Pane":{"working_dir":null,"profile":null}}}"#;
        let layout: SessionLayoutFile = serde_json::from_str(json).unwrap();
        assert_ne!(layout.version, SessionLayoutFile::CURRENT_VERSION);
    }
}
