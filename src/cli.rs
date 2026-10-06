use crate::{SearchEngine, SearchMode, SearchOptions};
use clap::Parser;
use std::path::PathBuf;
use colored::*;

#[derive(Parser, Debug)]
#[command(name = "fsearch")]
#[command(about = "Lightning-fast file and content search for Windows", long_about = None)]
pub struct CliArgs {
    /// The search query
    pub query: String,

    /// The root path to search in (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    pub path: String,

    /// Search mode: 'name', 'content', or 'both'
    #[arg(short, long, default_value = "both")]
    pub mode: String,

    /// Ignore case
    #[arg(short, long)]
    pub ignore_case: bool,

    /// Treat query as a regular expression
    #[arg(short, long)]
    pub regex: bool,

    /// Include hidden files and directories
    #[arg(long)]
    pub hidden: bool,

    /// Output in NDJSON format for AI agents
    #[arg(long)]
    pub json: bool,

    /// Maximum number of results per file
    #[arg(long)]
    pub max_results: Option<usize>,
}

impl CliArgs {
    pub fn into_search_options(self) -> SearchOptions {
        let mode = match self.mode.to_lowercase().as_str() {
            "name" => SearchMode::FileName,
            "content" => SearchMode::FileContent,
            _ => SearchMode::Both,
        };

        SearchOptions {
            query: self.query,
            root_path: PathBuf::from(self.path),
            mode,
            ignore_case: self.ignore_case,
            is_regex: self.regex,
            include_hidden: self.hidden,
            max_results: self.max_results,
        }
    }
}

pub fn run_cli(args: CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    let json_output = args.json;
    let opts = args.into_search_options();
    
    let summary = SearchEngine::search(opts, move |result| {
        if json_output {
            if let Ok(json) = serde_json::to_string(&result) {
                println!("{}", json);
            }
        } else {
            // Colored human-readable output
            if result.is_file_name_match {
                println!("{} {}", "[FILE]".green().bold(), result.file_path.display());
            } else {
                println!("{}", result.file_path.display().to_string().cyan().bold());
            }

            for mat in &result.matches {
                let line_num = mat.line_number.to_string().yellow();
                
                // Highlight the match in the line content
                let before = &mat.line_content[..mat.match_start];
                let matched = &mat.line_content[mat.match_start..mat.match_end];
                let after = &mat.line_content[mat.match_end..];
                
                println!("  {}: {}{}{}", line_num, before, matched.red().bold(), after);
            }
            if !result.matches.is_empty() {
                println!();
            }
        }
    })?;

    if !json_output {
        println!(
            "\n{} {} matches in {} files (searched {} files) in {}ms",
            "Done!".green().bold(),
            summary.total_matches,
            summary.files_with_matches,
            summary.files_searched,
            summary.duration_ms
        );
    }

    Ok(())
}
