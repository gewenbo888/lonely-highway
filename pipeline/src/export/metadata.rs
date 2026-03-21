use crate::tile::Tile;
use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct TileMetadata {
    coord_x: i32,
    coord_y: i32,
    bounds: TileBoundsJson,
    road_count: usize,
    building_count: usize,
    traffic_nodes: usize,
    traffic_edges: usize,
    signal_positions: Vec<[f32; 2]>,
}

#[derive(Serialize)]
struct TileBoundsJson {
    min_x: f32,
    min_z: f32,
    max_x: f32,
    max_z: f32,
}

pub fn export_metadata(tile: &Tile, path: &Path) -> Result<()> {
    let meta = TileMetadata {
        coord_x: tile.coord.x,
        coord_y: tile.coord.y,
        bounds: TileBoundsJson {
            min_x: tile.bounds.min_x,
            min_z: tile.bounds.min_z,
            max_x: tile.bounds.max_x,
            max_z: tile.bounds.max_z,
        },
        road_count: tile.road_meshes.len(),
        building_count: tile.building_meshes.len(),
        traffic_nodes: tile.traffic_graph.nodes.len(),
        traffic_edges: tile.traffic_graph.edges.len(),
        signal_positions: tile.signal_positions.iter().map(|(x, z)| [*x, *z]).collect(),
    };
    let json = serde_json::to_string_pretty(&meta)?;
    std::fs::write(path, json)?;
    log::info!("Wrote metadata: {}", path.display());
    Ok(())
}
