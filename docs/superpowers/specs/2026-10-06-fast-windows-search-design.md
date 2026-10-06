# Fast Windows Search Tool (`fsearch`) — Technical Design Specification

- **Date:** 2026-10-06
- **Status:** Approved
- **Target OS:** Windows 10 / Windows 11 (x64)
- **Primary Language:** Rust 2021 Edition

---

## 1. Overview & Objectives

### 1.1 Goal
Develop a lightning-fast file and file-content search engine for Windows that achieves throughput comparable to ripgrep (`rg`) and `fd`, consumable either as a command-line tool (`CLI`) for developers and AI agents, or as an instant desktop interface (`GUI`).

### 1.2 Key Requirements
- **Maximum Performance:** Multi-threaded filesystem traversal, memory-mapped I/O (`memmap2`), SIMD-accelerated string scanning (`memchr`), and linear-time regex execution (`regex-automata`).
- **Dual-Mode Single Binary:** Packaged as a standalone `fsearch.exe` with zero external runtime dependencies.
  - Invoked with arguments: executes in CLI mode and outputs to `stdout` (plain text with color or JSON streaming).
  - Invoked without arguments or with `--gui`: opens an ultra-lightweight desktop window powered by Microsoft Edge WebView2.
- **Smart Filtering:** Automatically respects `.gitignore`, `.ignore`, hidden file conventions, and rejects binary files within 1KB examination.
- **Low Footprint:** Memory usage under 35 MB in GUI mode, under 15 MB in CLI mode.

---

## 2. Architecture & Modules

```
                    +---------------------------+
                    |        fsearch.exe        |
                    +-------------+-------------+
                                  |
               +------------------+------------------+
               | (CLI Mode)                          | (GUI Mode)
               v                                     v
       +---------------+                     +---------------+
       |  cli::runner  |                     |  gui::window  |
       +-------+-------+                     +-------+-------+
               |                                     |
               | (Stream stdout/JSON)                | (IPC / Webview)
               +------------------+------------------+
                                  |
                                  v
                      +-----------------------+
                      |   core::SearchEngine  |
                      +-----------+-----------+
                                  |
         +------------------------+------------------------+
         |                                                 |
         v                                                 v
+------------------+                              +------------------+
|   core::walker   |                              |  core::matcher   |
| (ignore / rayon) |                              | (mmap / memchr)  |
+------------------+                              +------------------+
```

### 2.1 `core::types`
Data models shared between CLI, GUI, and search workers:
```rust
pub enum SearchMode {
    FileName,
    FileContent,
    Both,
}

pub struct SearchOptions {
    pub query: String,
    pub root_path: PathBuf,
    pub mode: SearchMode,
    pub ignore_case: bool,
    pub is_regex: bool,
    pub include_hidden: bool,
    pub max_results: Option<usize>,
}

pub struct MatchLocation {
    pub line_number: usize,
    pub line_content: String,
    pub match_start: usize,
    pub match_end: usize,
}

pub struct SearchResult {
    pub file_path: PathBuf,
    pub is_file_name_match: bool,
    pub matches: Vec<MatchLocation>,
}
```

### 2.2 `core::walker`
- Uses `ignore::WalkBuilder` for parallel directory traversal.
- Filters out non-matching files early.
- Handles symlinks safely and suppresses I/O permission errors gracefully.

### 2.3 `core::matcher`
- **File Name Matching:** Case-insensitive substring search or regex matching on file paths.
- **Binary Check:** Inspects the first 1024 bytes of each file. If null bytes (`0x00`) or disproportionate control bytes are detected, the file is classified as binary and content matching is skipped.
- **Content Matching with Memory Mapping:**
  - Files under 16KB: read directly into small stack buffer.
  - Files $\ge$ 16KB: memory mapped using `memmap2::Mmap::map(&file)`.
  - Employs `memchr::memmem::Finder` for substring search and `regex::bytes::Regex` for regular expressions.
  - Extracts line numbers and snippet lines efficiently using `memchr::memchr_iter(b'\n', &data)`.

### 2.4 `cli::runner`
- Uses `clap` for command-line parsing.
- Outputs human-readable terminal formatting with ANSI colors (green paths, yellow line numbers, bold red highlights).
- Supports `--json` flag: emits newline-delimited JSON (NDJSON) for direct consumption by AI agents:
  ```json
  {"type":"match","path":"D:\\repo\\src\\main.rs","line":42,"content":"fn main() { ... }","start":3,"end":7}
  ```

### 2.5 `gui::window` & Frontend
- Uses `wry` or Windows WebView2 native bindings to spawn a borderless or modern themed window.
- The UI (HTML/CSS/JS) is compiled directly into the binary via `include_str!("../assets/index.html")`.
- Communication via two-way IPC:
  - Frontend sends `{"action": "search", "options": {...}}` or `{"action": "open_file", "path": "...", "line": 42}`.
  - Backend streams batches of `SearchResult` objects back to the webview.
- **UI Layout:**
  - Fast search input with 80ms debounce.
  - Target directory selector.
  - Two-pane split view: left side is virtualized scroll list of matches; right side is syntax-highlighted preview pane.

---

## 3. Performance & Memory Targets

| Metric | Target |
|---|---|
| Directory Traversal Speed | > 50,000 files / sec |
| Content Scanning Throughput | > 1.2 GB / sec (SSD bound) |
| GUI Startup Time | < 250 ms |
| Peak RAM (CLI mode) | < 15 MB |
| Peak RAM (GUI mode) | < 35 MB |

---

## 4. Verification & Testing Plan

1. **Unit Tests:**
   - Case-sensitive and case-insensitive matching.
   - Regex matching correctness.
   - Binary file rejection test.
   - Line number calculation edge cases (CRLF vs LF).
2. **Integration Tests:**
   - Searching mock directory tree with `.gitignore` exclusions.
   - JSON output schema conformance for AI agents.
3. **End-to-End Test:**
   - CLI execution on project folder.
   - GUI launch and query execution.
