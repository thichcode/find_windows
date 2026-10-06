use clap::Parser;
use fsearch::cli::{CliArgs, run_cli};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    run_cli(args)
}

