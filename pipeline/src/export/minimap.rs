use crate::tile::Tile;
use anyhow::Result;
use std::path::Path;

/// Generate a top-down minimap image for this tile.
/// Roads are drawn as white lines on a dark background.
pub fn export_minimap(tile: &Tile, path: &Path) -> Result<()> {
    let size = 256u32; // pixels
    let tile_size = tile.bounds.max_x - tile.bounds.min_x;
    let scale = size as f32 / tile_size;

    let mut img = image::RgbaImage::new(size, size);

    // Dark background
    for pixel in img.pixels_mut() {
        *pixel = image::Rgba([30, 30, 40, 255]);
    }

    // Draw road center lines as white pixels
    for mesh in &tile.road_meshes {
        for pos in &mesh.positions {
            let px = ((pos[0] - tile.bounds.min_x) * scale) as i32;
            let py = size as i32 - ((pos[2] - tile.bounds.min_z) * scale) as i32;
            if px >= 0 && px < size as i32 && py >= 0 && py < size as i32 {
                img.put_pixel(px as u32, py as u32, image::Rgba([220, 220, 220, 255]));
            }
        }
    }

    img.save(path)?;
    log::info!("Wrote minimap: {}", path.display());
    Ok(())
}
