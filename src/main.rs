use clap::CommandFactory;
use clap::{Parser, Subcommand};
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

mod error;
mod tiles;
mod utils;

use error::TesseraError;

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
            use crate::tiles::{
                collect_content_uris, load_gltf_assets, load_tileset, summarize_gltf,
            };
            use std::path::PathBuf;

            let tileset_path = PathBuf::from(&tileset);
            let base_dir = tileset_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));

            let doc = load_tileset(&tileset_path)?;
            let uris = collect_content_uris(&doc);
            info!("Found {} content URIs in tileset", uris.len());

            let loaded = load_gltf_assets(base_dir, &uris);
            let mut ok_count = 0usize;
            let mut fail_count = 0usize;
            for res in loaded {
                match res {
                    Ok(asset) => {
                        ok_count += 1;
                        let (m, n, p) = summarize_gltf(&asset.document);
                        info!(
                            "Loaded {:?}: meshes={}, nodes={}, primitives={}",
                            asset.source_path, m, n, p
                        );
                    }
                    Err(e) => {
                        fail_count += 1;
                        error!("{}", e);
                    }
                }
            }
            info!("GLTF load results: ok={}, failed={}", ok_count, fail_count);
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
