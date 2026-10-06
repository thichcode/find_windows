#[cfg(test)]
mod tests {
    use fsearch::{ContentMatcher, SearchMode, SearchOptions};
    use std::path::PathBuf;

    fn make_opts(query: &str, is_regex: bool, ignore_case: bool) -> SearchOptions {
        SearchOptions {
            query: query.to_string(),
            root_path: PathBuf::from("."),
            mode: SearchMode::Both,
            ignore_case,
            is_regex,
            include_hidden: false,
            max_results: None,
        }
    }

    #[test]
    fn test_binary_detection() {
        let text_buf = b"Hello, world!\nLine 2\n";
        assert!(!fsearch::is_binary_buffer(text_buf));

        let bin_buf = b"Hello\x00World";
        assert!(fsearch::is_binary_buffer(bin_buf));
    }

    #[test]
    fn test_filename_match() {
        let opts = make_opts("hello", false, false);
        let matcher = ContentMatcher::new(&opts).unwrap();
        assert!(matcher.matches_filename("hello_world.txt"));
        assert!(!matcher.matches_filename("HELLO_world.txt"));

        let opts_case = make_opts("hello", false, true);
        let matcher_case = ContentMatcher::new(&opts_case).unwrap();
        assert!(matcher_case.matches_filename("HELLO_world.txt"));
    }

    #[test]
    fn test_content_match_exact() {
        let opts = make_opts("rust", false, false);
        let matcher = ContentMatcher::new(&opts).unwrap();
        
        let data = b"Hello world\nThis is rust code\nEnd of file";
        let matches = matcher.find_matches_in_bytes(data);
        
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line_number, 2);
        assert_eq!(matches[0].line_content, "This is rust code");
    }

    #[test]
    fn test_content_match_regex() {
        let opts = make_opts("r[a-z]{3}", true, false);
        let matcher = ContentMatcher::new(&opts).unwrap();
        
        let data = b"Hello rust world\nThis is a test";
        let matches = matcher.find_matches_in_bytes(data);
        
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line_content, "Hello rust world");
    }
}
