use clap::CommandFactory;
use clap::{Parser, Subcommand};
use std::time::Instant;
use tessera::tileset::compare::{DEFAULT_GEOMETRIC_ERROR_TOLERANCE, compare_tileset_json_files};
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

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable high-level timing logs
    #[arg(long, global = true)]
    timings: bool,
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

        /// Number of Rayon worker threads to use for recalculation.
        /// Defaults to Rayon/RAYON_NUM_THREADS behavior when omitted.
        #[arg(long("threads"), value_name = "COUNT")]
        threads: Option<usize>,
    },

    /// Compare two tileset JSON files while allowing tiny geometricError differences
    Compare {
        /// Path to the expected/baseline tileset.json
        #[arg(short('e'), long("expected"), value_name = "PATH")]
        expected: String,

        /// Path to the actual/new tileset.json
        #[arg(short('a'), long("actual"), value_name = "PATH")]
        actual: String,

        /// Absolute tolerance for geometricError comparisons
        #[arg(
            short('t'),
            long("tolerance"),
            value_name = "FLOAT",
            default_value_t = DEFAULT_GEOMETRIC_ERROR_TOLERANCE
        )]
        tolerance: f64,
    },
}

#[tokio::main]
async fn main() -> Result<(), TesseraError> {
    let cli = Cli::parse();

    let log_level = if cli.debug {
        Level::DEBUG
    } else if cli.verbose || cli.timings {
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
            threads,
        }) => {
            use std::path::PathBuf;
            use tessera::tileset::loader::load_tileset;

            if let Some(threads) = threads {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build_global()
                    .map_err(|e| {
                        TesseraError::Processing(format!(
                            "Failed to configure Rayon thread pool: {}",
                            e
                        ))
                    })?;
                info!(threads, "Configured Rayon thread pool");
            }

            let total_start = Instant::now();
            let tileset_path = PathBuf::from(&tileset);
            let base_dir = tileset_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));

            let load_start = Instant::now();
            let mut doc = load_tileset(&tileset_path)?;
            info!(
                elapsed_ms = load_start.elapsed().as_millis(),
                input = tileset,
                "Loaded tileset"
            );

            let calculation_start = Instant::now();
            calculate_geometric_error_with_cache_size(&mut doc, base_dir, cache_tiles)?;
            info!(
                elapsed_ms = calculation_start.elapsed().as_millis(),
                "Recalculated geometric errors"
            );

            let write_start = Instant::now();
            write_tileset(&doc, &PathBuf::from(&output), pretty.unwrap_or(false))?;
            info!(
                elapsed_ms = write_start.elapsed().as_millis(),
                output = output,
                "Wrote tileset"
            );

            info!(
                elapsed_ms = total_start.elapsed().as_millis(),
                "Completed recalculate command"
            );
        }
        Some(Commands::Compare {
            expected,
            actual,
            tolerance,
        }) => {
            use std::path::PathBuf;

            let compare_start = Instant::now();
            compare_tileset_json_files(
                &PathBuf::from(&expected),
                &PathBuf::from(&actual),
                tolerance,
            )?;
            info!(
                elapsed_ms = compare_start.elapsed().as_millis(),
                expected, actual, tolerance, "Tileset JSON comparison passed"
            );
            println!("Tileset JSON comparison passed");
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
