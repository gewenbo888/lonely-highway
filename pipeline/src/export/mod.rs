pub mod gltf;
pub mod traffic_json;
pub mod metadata;
pub mod minimap;

use crate::tile::Tile;
use anyhow::Result;
use std::path::Path;

pub fn export_tile(tile: &Tile, output_dir: &str) -> Result<()> {
    let dir = Path::new(output_dir);
    let prefix = format!("tile_{}_{}", tile.coord.x, tile.coord.y);

    gltf::export_tile_gltf(tile, &dir.join(format!("{}.glb", prefix)))?;
    traffic_json::export_traffic_json(tile, &dir.join(format!("{}_traffic.json", prefix)))?;
    metadata::export_metadata(tile, &dir.join(format!("{}_meta.json", prefix)))?;
    minimap::export_minimap(tile, &dir.join(format!("{}_minimap.png", prefix)))?;

    Ok(())
}
