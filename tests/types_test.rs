#[cfg(test)]
mod tests {
    use fsearch::{MatchLocation, SearchMode, SearchOptions, SearchResult};
    use std::path::PathBuf;

    #[test]
    fn test_search_options_construction() {
        let opts = SearchOptions {
            query: "hello".to_string(),
            root_path: PathBuf::from("D:\\project"),
            mode: SearchMode::Both,
            ignore_case: true,
            is_regex: false,
            include_hidden: false,
            max_results: Some(100),
        };
        assert_eq!(opts.query, "hello");
        assert!(opts.ignore_case);
        assert!(!opts.is_regex);
        assert_eq!(opts.max_results, Some(100));
    }

    #[test]
    fn test_search_mode_variants() {
        let _file = SearchMode::FileName;
        let _content = SearchMode::FileContent;
        let _both = SearchMode::Both;
    }

    #[test]
    fn test_match_location_construction() {
        let loc = MatchLocation {
            line_number: 42,
            line_content: "fn main() {}".to_string(),
            match_start: 3,
            match_end: 7,
        };
        assert_eq!(loc.line_number, 42);
        assert_eq!(loc.match_start, 3);
        assert_eq!(loc.match_end, 7);
    }

    #[test]
    fn test_search_result_construction() {
        let result = SearchResult {
            file_path: PathBuf::from("src/main.rs"),
            is_file_name_match: true,
            matches: vec![MatchLocation {
                line_number: 1,
                line_content: "fn main() {}".to_string(),
                match_start: 0,
                match_end: 4,
            }],
        };
        assert!(result.is_file_name_match);
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.file_path, PathBuf::from("src/main.rs"));
    }

    #[test]
    fn test_search_result_json_serialization() {
        let result = SearchResult {
            file_path: PathBuf::from("test.txt"),
            is_file_name_match: false,
            matches: vec![MatchLocation {
                line_number: 10,
                line_content: "hello world".to_string(),
                match_start: 0,
                match_end: 5,
            }],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test.txt"));
        assert!(json.contains("hello world"));
    }
}
