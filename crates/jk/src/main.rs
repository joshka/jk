use color_eyre::Result;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {}

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    let _args = Args::parse();
    tracing::info!("starting reset scaffold");
    println!("jk reset scaffold");
    Ok(())
}
