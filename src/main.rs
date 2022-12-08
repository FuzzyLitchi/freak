use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use clap::Parser;
use color_eyre::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    file: PathBuf,
    // /// Sets a custom config file
    // #[arg(short, long, value_name = "FILE")]
    // config: Option<PathBuf>,

    // /// Turn debugging information on
    // #[arg(short, long, action = clap::ArgAction::Count)]
    // debug: u8,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    let file = File::open(cli.file)?;

    let mut count: HashMap<u8, u64> = HashMap::new();

    for byte in file.bytes() {
        let byte = byte?;
        *count.entry(byte).or_insert(0) += 1;
    }

    let mut count: Vec<(u8, u64)> = count.into_iter().collect();
    count.sort_by(|(_, n1), (_, n2)| n1.cmp(n2));

    for (byte, n) in count {
        println!("{byte:02x}: {n}");
    }

    Ok(())
}
