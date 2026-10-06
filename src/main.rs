#![windows_subsystem = "windows"]

use clap::Parser;
use fsearch::cli::{run_cli, CliArgs};
use fsearch::gui::run_gui;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] != "--gui" {
        // Run CLI if arguments provided
        let cli_args = CliArgs::parse();
        run_cli(cli_args)
    } else {
        // Run GUI mode
        run_gui();
        Ok(())
    }
}
