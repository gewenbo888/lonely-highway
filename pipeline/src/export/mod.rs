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

    // Export road + building meshes as glTF
    gltf::export_tile_gltf(tile, &dir.join(format!("{}.glb", prefix)))?;

    // Export traffic graph as JSON
    traffic_json::export_traffic_json(tile, &dir.join(format!("{}_traffic.json", prefix)))?;

    // Export tile metadata
    metadata::export_metadata(tile, &dir.join(format!("{}_meta.json", prefix)))?;

    // Export minimap tile
    minimap::export_minimap(tile, &dir.join(format!("{}_minimap.png", prefix)))?;

    Ok(())
}
