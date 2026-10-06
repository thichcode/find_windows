use crate::{is_binary_buffer, ContentMatcher, SearchMode, SearchOptions, SearchResult, SearchSummary};
use ignore::WalkBuilder;
use memmap2::MmapOptions;
use std::error::Error;
use std::fs::File;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub struct SearchEngine;

impl SearchEngine {
    pub fn search<F>(options: SearchOptions, on_result: F) -> Result<SearchSummary, Box<dyn Error>>
    where
        F: Fn(SearchResult) + Send + Sync + 'static,
    {
        let start_time = Instant::now();
        let matcher = ContentMatcher::new(&options)?;
        
        let mut walker_builder = WalkBuilder::new(&options.root_path);
        walker_builder
            .hidden(!options.include_hidden)
            .ignore(!options.include_hidden)
            .git_ignore(!options.include_hidden)
            .git_exclude(!options.include_hidden);

        let files_searched = Arc::new(AtomicUsize::new(0));
        let files_with_matches = Arc::new(AtomicUsize::new(0));
        let total_matches = Arc::new(AtomicUsize::new(0));

        let on_result = Arc::new(on_result);
        
        walker_builder.build_parallel().run(|| {
            let options = options.clone();
            let matcher = &matcher;
            let files_searched = Arc::clone(&files_searched);
            let files_with_matches = Arc::clone(&files_with_matches);
            let total_matches = Arc::clone(&total_matches);
            let on_result = Arc::clone(&on_result);

            Box::new(move |result| {
                if let Ok(entry) = result {
                    if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                        return ignore::WalkState::Continue;
                    }

                    files_searched.fetch_add(1, Ordering::Relaxed);
                    let path = entry.path();
                    let path_str = path.to_string_lossy();
                    
                    let is_file_name_match = if options.mode == SearchMode::FileName || options.mode == SearchMode::Both {
                        matcher.matches_filename(&path_str)
                    } else {
                        false
                    };

                    let mut matches = Vec::new();

                    if options.mode == SearchMode::FileContent || options.mode == SearchMode::Both {
                        if let Ok(file) = File::open(path) {
                            if let Ok(mmap) = unsafe { MmapOptions::new().map(&file) } {
                                if !is_binary_buffer(&mmap) {
                                    matches = matcher.find_matches_in_bytes(&mmap);
                                }
                            }
                        }
                    }

                    if is_file_name_match || !matches.is_empty() {
                        let match_count = matches.len();
                        
                        on_result(SearchResult {
                            file_path: path.to_path_buf(),
                            is_file_name_match,
                            matches,
                        });

                        files_with_matches.fetch_add(1, Ordering::Relaxed);
                        total_matches.fetch_add(match_count, Ordering::Relaxed);
                    }
                }
                
                ignore::WalkState::Continue
            })
        });

        Ok(SearchSummary {
            total_matches: total_matches.load(Ordering::Relaxed),
            files_with_matches: files_with_matches.load(Ordering::Relaxed),
            files_searched: files_searched.load(Ordering::Relaxed),
            duration_ms: start_time.elapsed().as_millis(),
        })
    }
}
