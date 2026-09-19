use std::fs;
use std::path::PathBuf;

use clap::Parser;

use structdiff::diff::diff;
use structdiff::render::render;

#[derive(Parser)]
#[command(
    name = "structdiff",
    about = "Semantic JSON diff — reports added/removed/changed by path, not a line-based text diff"
)]
struct Cli {
    old: PathBuf,
    new: PathBuf,
}

fn load(path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    let text =
        fs::read_to_string(path).map_err(|e| anyhow::anyhow!("reading {}: {e}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|e| anyhow::anyhow!("{}: invalid JSON: {e}", path.display()))
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let old = load(&cli.old)?;
    let new = load(&cli.new)?;

    let findings = diff(&old, &new);
    if findings.is_empty() {
        println!("no differences");
        return Ok(());
    }

    println!("{}", render(&findings));
    std::process::exit(1);
}
