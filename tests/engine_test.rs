#[cfg(test)]
mod tests {
    use fsearch::{SearchEngine, SearchMode, SearchOptions, SearchResult};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    fn setup_test_dir() -> PathBuf {
        let dir = std::env::temp_dir().join("fsearch_engine_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("test.txt"), "hello world\nthis is a rust test\n").unwrap();
        fs::write(dir.join("ignored.txt"), "hello world in ignored file").unwrap();
        fs::write(dir.join(".gitignore"), "ignored.txt").unwrap();
        fs::write(dir.join("binary.bin"), b"hello\x00world").unwrap();
        
        let subdir = dir.join("subdir");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join("another.rs"), "fn main() {\n    println!(\"hello from rust\");\n}").unwrap();

        dir
    }

    #[test]
    fn test_search_engine_basic() {
        let dir = setup_test_dir();

        let opts = SearchOptions {
            query: "rust".to_string(),
            root_path: dir.clone(),
            mode: SearchMode::Both,
            ignore_case: false,
            is_regex: false,
            include_hidden: false,
            max_results: None,
        };

        let results = Arc::new(Mutex::new(Vec::new()));
        let results_clone = results.clone();

        let summary = SearchEngine::search(opts, move |res: SearchResult| {
            results_clone.lock().unwrap().push(res);
        }).unwrap();

        let res = results.lock().unwrap();
        
        // test.txt and subdir/another.rs should match
        assert_eq!(res.len(), 2);
        
        let mut files: Vec<_> = res.iter().map(|r| r.file_path.file_name().unwrap().to_str().unwrap()).collect();
        files.sort();
        assert_eq!(files, vec!["another.rs", "test.txt"]);

        assert_eq!(summary.files_with_matches, 2);
        // It should have searched test.txt, binary.bin (rejected), .gitignore, another.rs
        assert!(summary.files_searched >= 3); 
    }
}
