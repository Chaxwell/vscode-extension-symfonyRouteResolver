use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Index {
        id: u64,
        workspace_path: String,
        symfony_binary: String,
    },
    Search {
        id: u64,
        content: String,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    Indexed {
        id: u64,
        count: usize,
    },
    Searched {
        id: u64,
        matches: Vec<TextMatch>,
    },
    Error {
        id: u64,
        message: String,
    },
}

/// Location of a route controller declaration in a PHP file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    /// Absolute path to the PHP file.
    pub file: String,
    /// 1-indexed line number of the class or method declaration.
    pub line: u32,
}

/// A single occurrence of a route name found in a document.
#[derive(Debug, Serialize)]
pub struct TextMatch {
    pub route: String,
    pub position: MatchPosition,
    pub file: RouteInfo,
}

#[derive(Debug, Serialize)]
pub struct MatchPosition {
    pub start: LineCol,
    pub end: LineCol,
}

/// 0-indexed line and column (compatible with VSCode's `vscode.Position`).
#[derive(Debug, Serialize)]
pub struct LineCol {
    pub line: u32,
    pub col: u32,
}
