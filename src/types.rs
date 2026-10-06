use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Determines what to search: file names, file content, or both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchMode {
    FileName,
    FileContent,
    Both,
}

/// Configuration for a single search operation.
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub query: String,
    pub root_path: PathBuf,
    pub mode: SearchMode,
    pub ignore_case: bool,
    pub is_regex: bool,
    pub include_hidden: bool,
    pub max_results: Option<usize>,
}

/// A single match location within a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchLocation {
    pub line_number: usize,
    pub line_content: String,
    pub match_start: usize,
    pub match_end: usize,
}

/// A search result for one file, containing zero or more content matches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_path: PathBuf,
    pub is_file_name_match: bool,
    pub matches: Vec<MatchLocation>,
}

/// Summary statistics returned after a search completes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchSummary {
    pub total_matches: usize,
    pub files_with_matches: usize,
    pub files_searched: usize,
    pub duration_ms: u128,
}
