use crate::tile::Tile;
use anyhow::Result;
use std::path::Path;

pub fn export_traffic_json(tile: &Tile, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(&tile.traffic_graph)?;
    std::fs::write(path, json)?;
    log::info!("Wrote traffic graph: {}", path.display());
    Ok(())
}
