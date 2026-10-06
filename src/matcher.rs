use crate::{MatchLocation, SearchOptions};
use memchr::memchr;
use memchr::memmem::Finder;
use regex::bytes::Regex;
use std::error::Error;

const BINARY_CHECK_BYTES: usize = 1024;

/// Detects if a buffer contains binary data by checking for null bytes
/// in the first 1024 bytes.
pub fn is_binary_buffer(buf: &[u8]) -> bool {
    let check_len = buf.len().min(BINARY_CHECK_BYTES);
    memchr(0, &buf[..check_len]).is_some()
}

pub struct ContentMatcher {
    opts: SearchOptions,
    regex: Option<Regex>,
    finder: Option<Finder<'static>>,
    finder_case_insensitive: Option<Finder<'static>>, // For lowercase conversion fallback if needed, but wait: memchr Finder doesn't do case-insensitive natively.
    query_bytes: Vec<u8>,
    query_bytes_lower: Vec<u8>,
}

impl ContentMatcher {
    pub fn new(options: &SearchOptions) -> Result<Self, Box<dyn Error>> {
        let mut regex = None;
        let mut finder = None;
        let query_bytes = options.query.as_bytes().to_vec();
        let query_bytes_lower = options.query.to_lowercase().as_bytes().to_vec();

        if options.is_regex {
            let mut builder = regex::bytes::RegexBuilder::new(&options.query);
            builder.case_insensitive(options.ignore_case);
            regex = Some(builder.build()?);
        } else if !options.ignore_case {
            finder = Some(Finder::new(&query_bytes).into_owned());
        }

        Ok(Self {
            opts: options.clone(),
            regex,
            finder,
            finder_case_insensitive: None,
            query_bytes,
            query_bytes_lower,
        })
    }

    pub fn matches_filename(&self, path_str: &str) -> bool {
        if self.opts.is_regex {
            if let Some(re) = &self.regex {
                return re.is_match(path_str.as_bytes());
            }
        }
        
        if self.opts.ignore_case {
            path_str.to_lowercase().contains(&self.opts.query.to_lowercase())
        } else {
            path_str.contains(&self.opts.query)
        }
    }

    pub fn find_matches_in_bytes(&self, data: &[u8]) -> Vec<MatchLocation> {
        let mut results = Vec::new();

        if self.opts.is_regex {
            if let Some(re) = &self.regex {
                for cap in re.captures_iter(data) {
                    if let Some(mat) = cap.get(0) {
                        let (line_num, line_start, line_end) = Self::find_line_bounds(data, mat.start(), mat.end());
                        let line_content = String::from_utf8_lossy(&data[line_start..line_end]).into_owned();
                        results.push(MatchLocation {
                            line_number: line_num,
                            line_content,
                            match_start: mat.start() - line_start,
                            match_end: mat.end() - line_start,
                        });
                        
                        if let Some(max) = self.opts.max_results {
                            if results.len() >= max {
                                break;
                            }
                        }
                    }
                }
            }
        } else if self.opts.ignore_case {
            // Case-insensitive exact match: fallback to regex for simplicity in this engine,
            // or naive iteration. Let's use regex for case-insensitive exact match for now 
            // since we already have regex crate and building a custom case-insensitive SIMD is hard.
            // Wait, we can construct a literal regex.
            let re = regex::bytes::RegexBuilder::new(&regex::escape(&self.opts.query))
                .case_insensitive(true)
                .build()
                .unwrap();
            
            for cap in re.captures_iter(data) {
                if let Some(mat) = cap.get(0) {
                    let (line_num, line_start, line_end) = Self::find_line_bounds(data, mat.start(), mat.end());
                    let line_content = String::from_utf8_lossy(&data[line_start..line_end]).into_owned();
                    results.push(MatchLocation {
                        line_number: line_num,
                        line_content,
                        match_start: mat.start() - line_start,
                        match_end: mat.end() - line_start,
                    });
                    
                    if let Some(max) = self.opts.max_results {
                        if results.len() >= max {
                            break;
                        }
                    }
                }
            }
        } else {
            // Case-sensitive exact match using memchr Finder
            if let Some(finder) = &self.finder {
                for match_idx in finder.find_iter(data) {
                    let match_end = match_idx + self.query_bytes.len();
                    let (line_num, line_start, line_end) = Self::find_line_bounds(data, match_idx, match_end);
                    let line_content = String::from_utf8_lossy(&data[line_start..line_end]).into_owned();
                    results.push(MatchLocation {
                        line_number: line_num,
                        line_content,
                        match_start: match_idx - line_start,
                        match_end: match_end - line_start,
                    });
                    
                    if let Some(max) = self.opts.max_results {
                        if results.len() >= max {
                            break;
                        }
                    }
                }
            }
        }

        results
    }

    fn find_line_bounds(data: &[u8], match_start: usize, match_end: usize) -> (usize, usize, usize) {
        // Count newlines before match_start to get line number (1-indexed)
        let mut line_number = 1;
        let mut line_start = 0;
        
        for (i, &b) in data[..match_start].iter().enumerate() {
            if b == b'\n' {
                line_number += 1;
                line_start = i + 1;
            }
        }

        // Find line end
        let mut line_end = data.len();
        for (i, &b) in data[match_end..].iter().enumerate() {
            if b == b'\n' {
                line_end = match_end + i;
                break;
            }
        }
        
        // Trim trailing CR if CRLF
        if line_end > line_start && data[line_end - 1] == b'\r' {
            line_end -= 1;
        }

        (line_number, line_start, line_end)
    }
}
