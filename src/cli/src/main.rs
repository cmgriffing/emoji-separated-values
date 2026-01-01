use anyhow::Result;
use clap::Parser;
use esv_cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run()
}
