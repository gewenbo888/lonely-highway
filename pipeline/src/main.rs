mod config;
mod fetch;
mod parse;
mod mesh;
mod traffic;
mod tile;
mod export;

use clap::Parser;
use config::{CliArgs, PipelineConfig};

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = CliArgs::parse();
    let config = PipelineConfig::from_args(&args)?;

    log::info!("Lonely Highway Pipeline");
    log::info!("Bounding box: {:?}", config.bbox);
    log::info!("Tile size: {}m", config.tile_size);
    log::info!("Output: {}", config.output_dir);

    // Stage 1: Fetch
    let osm_data = if let Some(cached_path) = &args.cached {
        log::info!("Loading cached OSM data from {}", cached_path);
        std::fs::read_to_string(cached_path)?
    } else {
        log::info!("Fetching OSM data from Overpass API...");
        fetch::fetch_osm_data(&config.bbox)?
    };

    // Stage 2: Parse
    log::info!("Parsing OSM data...");
    let parsed = parse::parse_osm(&osm_data, &config.fallbacks)?;
    log::info!(
        "Parsed: {} roads, {} buildings, {} signals",
        parsed.roads.len(),
        parsed.buildings.len(),
        parsed.signals.len()
    );

    // Stage 3: Generate meshes
    log::info!("Generating meshes...");
    let meshes = mesh::generate_meshes(&parsed)?;

    // Stage 4: Build traffic graph
    log::info!("Building traffic graph...");
    let traffic_graph = traffic::build_traffic_graph(&parsed)?;

    // Stage 5: Chunk into tiles
    log::info!("Chunking into {}m tiles...", config.tile_size);
    let tiles = tile::chunk_into_tiles(&meshes, &traffic_graph, &parsed, &config)?;
    log::info!("Generated {} tiles", tiles.len());

    // Stage 6: Export
    log::info!("Exporting tiles to {}...", config.output_dir);
    std::fs::create_dir_all(&config.output_dir)?;
    for tile in &tiles {
        export::export_tile(tile, &config.output_dir)?;
    }

    log::info!("Pipeline complete!");
    Ok(())
}
