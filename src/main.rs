use clap::CommandFactory;
use clap::{Parser, Subcommand};
use tessera::tileset::writer::write_tileset;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use tessera::calculate_geometric_error_with_cache_size;
use tessera::error::TesseraError;

#[derive(Parser)]
#[command(
    name = "tessera",
    about = "A command line interface for calculating geometric error for 3D Tiles tilesets",
    version,
    long_about = "Tessera is a CLI tool to traverse a 3D Tiles tileset and calculate a correct geometric error for each tile."
)]
struct Cli {
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
        #[arg(short('i'), long("input"), value_name = "PATH")]
        tileset: String,

        /// Path to output tileset.json
        #[arg(short('o'), long("output"), value_name = "PATH")]
        output: String,

        /// Whether to pretty print the output tileset.json
        #[arg(short('p'), long("pretty"), action = clap::ArgAction::SetTrue)]
        pretty: Option<bool>,

        /// Maximum number of decoded tile geometries to keep in memory. Use 0 to disable caching.
        #[arg(
            long("cache-tiles"),
            visible_alias("tile-load-limit"),
            value_name = "LIMIT",
            default_value_t = tessera::DEFAULT_GEOMETRY_CACHE_TILES
        )]
        cache_tiles: usize,
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

    match cli.command {
        Some(Commands::Recalculate {
            tileset,
            output,
            pretty,
            cache_tiles,
        }) => {
            use std::path::PathBuf;
            use tessera::tileset::loader::load_tileset;

            let tileset_path = PathBuf::from(&tileset);
            let base_dir = tileset_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));

            let mut doc = load_tileset(&tileset_path)?;

            calculate_geometric_error_with_cache_size(&mut doc, base_dir, cache_tiles)?;

            write_tileset(&doc, &PathBuf::from(&output), pretty.unwrap_or(false))?;
        }
        // No subcommand and no tileset path: show help
        None => {
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
