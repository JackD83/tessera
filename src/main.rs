use clap::CommandFactory;
use clap::{Parser, Subcommand};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use tessera::calculate_geometric_error;
use tessera::error::TesseraError;

#[derive(Parser)]
#[command(
    name = "tessera",
    about = "A command line interface for calculating geometric error for 3D Tiles tilesets",
    version,
    long_about = "Tessera is a CLI tool to traverse a 3D Tiles tileset and calculate a correct geometric error for each tile."
)]
struct Cli {
    /// Optional: path to tileset.json (equivalent to `recalculate` subcommand)
    #[arg(value_name = "PATH")]
    tileset: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable debug logging
    #[arg(short, long, global = true)]
    debug: bool,

    // Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Recalculate the geometric error of the tiles in a 3D Tiles tileset.json
    Recalculate {
        /// Path to tileset.json
        #[arg(value_name = "PATH")]
        tileset: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), TesseraError> {
    let cli = Cli::parse();

    let log_level = if cli.debug {
        Level::DEBUG
    } else if cli.verbose {
        Level::INFO
    } else {
        Level::WARN
    };

    FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .init();

    info!("Starting Tessera CLI");

    match (cli.command, cli.tileset) {
        (Some(Commands::Recalculate { tileset }), _) | (None, Some(tileset)) => {
            use std::path::PathBuf;
            use tessera::tileset::loader::load_tileset;

            let tileset_path = PathBuf::from(&tileset);
            let base_dir = tileset_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));

            let doc = load_tileset(&tileset_path)?;

            calculate_geometric_error(&doc, base_dir)?;
        }
        // No subcommand and no tileset path: show help
        (None, None) => {
            // Print help and exit with code 2 to indicate usage
            let mut cmd = Cli::command();
            let _ = cmd.print_help();
            eprintln!();
            std::process::exit(2);
        }
    }

    info!("Tessera CLI completed successfully");

    return Ok(());
}
