# Fast Windows Search (`fsearch`) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a high-performance Windows file and content search application (`fsearch.exe`) in Rust with dual-mode CLI (colored / streaming NDJSON) and embedded Edge WebView2 desktop GUI.

**Architecture:** Multi-threaded parallel directory traversal via `ignore` and `rayon`, memory-mapped file reading with `memmap2`, SIMD substring and regex scanning with `memchr`, streaming results via `crossbeam-channel`, and single-binary packaging with embedded HTML/CSS/JS frontend.

**Tech Stack:** Rust 2021, `ignore`, `memmap2`, `memchr`, `regex`, `clap`, `serde`, `serde_json`, `colored`, `rayon`, `crossbeam-channel`, `wry` (or Windows native WebView2).

**Spec:** [docs/superpowers/specs/2026-10-06-fast-windows-search-design.md](file:///d:/pupeteer/find_windows/docs/superpowers/specs/2026-10-06-fast-windows-search-design.md)

## Global Constraints

- Standalone single `.exe` binary without external runtime or DLL dependencies beyond Windows Edge WebView2.
- Respect `.gitignore`, `.ignore`, and hidden files by default unless explicitly disabled with `--hidden`.
- Binary file detection within first 1024 bytes (skip scanning binary content).
- Fast memory mapped I/O for files $\ge$ 16KB, direct buffered read for smaller files.
- NDJSON output format strictly compliant for AI Agent streaming.

## Review Focus

1. **Very large files (e.g. > 1GB log files):** Must not crash with Out-Of-Memory; memory-mapping handles page eviction cleanly.
2. **Binary files disguised with `.txt` or `.log` extensions:** Null bytes in first 1024 bytes must stop content search immediately.
3. **Windows line ending variations (`\r\n` vs `\n`):** Line numbers and column offsets must be calculated accurately across both CRLF and LF.
4. **Locked or permission-denied files:** I/O errors must be gracefully skipped without terminating the search walk.
5. **No matches found / Empty directories:** Must return clean 0 exit code or empty result set without errors.

---

### Task 1: Project Setup & Core Types Definition

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/types.rs`
- Test: `tests/types_test.rs`

**Interfaces:**
- Produces:
  ```rust
  pub enum SearchMode { FileName, FileContent, Both }
  pub struct SearchOptions { pub query: String, pub root_path: PathBuf, pub mode: SearchMode, pub ignore_case: bool, pub is_regex: bool, pub include_hidden: bool, pub max_results: Option<usize> }
  pub struct MatchLocation { pub line_number: usize, pub line_content: String, pub match_start: usize, pub match_end: usize }
  pub struct SearchResult { pub file_path: PathBuf, pub is_file_name_match: bool, pub matches: Vec<MatchLocation> }
  ```

- [ ] **Step 1: Write the failing test for Core Types**
  In `tests/types_test.rs`, test construction and serialization of `SearchOptions`, `MatchLocation`, and `SearchResult`.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test types_test`
  Expected: FAIL with compilation error (missing types).

- [ ] **Step 3: Implement `Cargo.toml`, `src/types.rs`, and `src/lib.rs`**
  Configure dependencies in `Cargo.toml` and define the data structures in `src/types.rs`.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test types_test`
  Expected: PASS.

- [ ] **Step 5: Commit**
  Run: `git add Cargo.toml src/ tests/ && git commit -m "feat: initialize project and core data types"`

---

### Task 2: Core Matcher & Binary Content Detection

**Files:**
- Create: `src/matcher.rs`
- Modify: `src/lib.rs`
- Test: `tests/matcher_test.rs`

**Interfaces:**
- Consumes: `SearchOptions`, `MatchLocation` from Task 1
- Produces:
  ```rust
  pub fn is_binary_buffer(buf: &[u8]) -> bool;
  pub struct ContentMatcher { ... }
  impl ContentMatcher {
      pub fn new(options: &SearchOptions) -> Result<Self, Box<dyn std::error::Error>>;
      pub fn matches_filename(&self, path_str: &str) -> bool;
      pub fn find_matches_in_bytes(&self, data: &[u8]) -> Vec<MatchLocation>;
  }
  ```

- [ ] **Step 1: Write the failing test for Matcher**
  In `tests/matcher_test.rs`, test:
  1. Substring search with case-sensitivity and case-insensitivity.
  2. Regex pattern matching.
  3. Binary detection on buffers containing null bytes.
  4. CRLF vs LF line number calculation.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test matcher_test`
  Expected: FAIL with missing `ContentMatcher`.

- [ ] **Step 3: Implement `src/matcher.rs`**
  Implement SIMD substring finding via `memchr::memmem::Finder`, regex matching via `regex::bytes::Regex`, and line slicing via `memchr::memchr_iter`.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test matcher_test`
  Expected: PASS.

- [ ] **Step 5: Commit**
  Run: `git add src/matcher.rs src/lib.rs tests/matcher_test.rs && git commit -m "feat: implement SIMD content matcher and binary file detection"`

---

### Task 3: Filesystem Walker & Memory-Mapped Scanning Engine

**Files:**
- Create: `src/engine.rs`
- Modify: `src/lib.rs`
- Test: `tests/engine_test.rs`

**Interfaces:**
- Consumes: `SearchOptions`, `SearchResult`, `ContentMatcher` from Tasks 1 & 2
- Produces:
  ```rust
  pub struct SearchEngine;
  impl SearchEngine {
      pub fn search<F>(options: SearchOptions, on_result: F) -> Result<SearchSummary, Box<dyn std::error::Error>>
      where F: Fn(SearchResult) + Send + Sync + 'static;
  }
  ```

- [ ] **Step 1: Write the failing test for SearchEngine**
  In `tests/engine_test.rs`, create a temporary directory tree with `.gitignore`, binary files (`.png`), and nested source files, and test search execution.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test engine_test`
  Expected: FAIL.

- [ ] **Step 3: Implement `src/engine.rs`**
  Implement `ignore::WalkBuilder` parallel walker with memory mapped files (`memmap2::Mmap`) and result dispatching.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test engine_test`
  Expected: PASS.

- [ ] **Step 5: Commit**
  Run: `git add src/engine.rs src/lib.rs tests/engine_test.rs && git commit -m "feat: implement parallel walker and mmap search engine"`

---

### Task 4: CLI Application & AI Agent Streaming (NDJSON)

**Files:**
- Create: `src/cli.rs`
- Create: `src/main.rs`
- Test: `tests/cli_test.rs`

**Interfaces:**
- Consumes: `SearchEngine`, `SearchOptions` from Tasks 1-3
- Produces:
  ```rust
  pub fn run_cli(args: CliArgs) -> Result<(), Box<dyn std::error::Error>>;
  ```

- [ ] **Step 1: Write the failing test for CLI and JSON streaming**
  In `tests/cli_test.rs`, test CLI argument parsing and NDJSON serialization format.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test cli_test`
  Expected: FAIL.

- [ ] **Step 3: Implement `src/cli.rs` and `src/main.rs`**
  Parse flags via `clap`, print colored output or stream NDJSON to stdout, with summary footer.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test cli_test`
  Expected: PASS.

- [ ] **Step 5: Commit**
  Run: `git add src/cli.rs src/main.rs tests/cli_test.rs && git commit -m "feat: implement CLI runner and AI agent NDJSON streaming"`

---

### Task 5: Desktop GUI with Embedded WebView2 & Web Assets

**Files:**
- Create: `src/gui.rs`
- Create: `assets/index.html`
- Create: `assets/style.css`
- Create: `assets/app.js`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `SearchEngine`, `SearchOptions` from Task 3
- Produces:
  ```rust
  pub fn run_gui() -> Result<(), Box<dyn std::error::Error>>;
  ```

- [ ] **Step 1: Create embedded frontend assets**
  Create modern dark-theme responsive UI in `assets/index.html`, `assets/style.css`, and `assets/app.js` with live search, split view, result list, and preview pane.

- [ ] **Step 2: Implement `src/gui.rs`**
  Build WebView2 / wry native window runner and connect IPC commands:
  - `"search"`: runs search in background thread and streams results back to UI.
  - `"open_file"`: opens file in default editor or VS Code.
  - `"open_folder"`: opens Windows Explorer with file selected (`explorer.exe /select,...`).

- [ ] **Step 3: Update `src/main.rs` for Dual-Mode Dispatch**
  If CLI arguments are passed, execute `run_cli(args)`. If no arguments or `--gui`, execute `run_gui()`.

- [ ] **Step 4: Verify Compilation & Integration**
  Run: `cargo check` & `cargo build`
  Expected: PASS with compiled binary `target/debug/fsearch.exe`.

- [ ] **Step 5: Commit**
  Run: `git add assets/ src/gui.rs src/main.rs && git commit -m "feat: implement embedded WebView2 desktop GUI"`

---

### Task 6: End-to-End Verification & Benchmarks

**Files:**
- Test: Run verification on local workspace

- [ ] **Step 1: Test CLI search on current repository**
  Run: `cargo run -- "fsearch" -c`
  Verify colored matches and fast response time (< 50ms).

- [ ] **Step 2: Test NDJSON streaming for AI Agent**
  Run: `cargo run -- "SearchMode" --json`
  Verify valid JSON output per line.

- [ ] **Step 3: Test GUI launch**
  Verify GUI launches cleanly with embedded assets.

- [ ] **Step 4: Build release binary**
  Run: `cargo build --release`
  Verify standalone `target/release/fsearch.exe` is created.

- [ ] **Step 5: Final Commit**
  Run: `git add . && git commit -m "feat: complete fast windows search tool"`
