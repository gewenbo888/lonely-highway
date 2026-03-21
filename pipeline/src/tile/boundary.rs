// Boundary duplication and shared edge IDs for tile seam stitching.
// Features crossing tile boundaries get duplicated into both tiles.
// Each duplicate carries a shared edge ID for runtime stitching.

use super::TileBounds;

/// Unique ID for a shared boundary feature.
pub type SharedEdgeId = u64;

/// Check if a line segment crosses a tile boundary.
pub fn crosses_boundary(x0: f32, z0: f32, x1: f32, z1: f32, bounds: &TileBounds) -> bool {
    let in0 = x0 >= bounds.min_x && x0 <= bounds.max_x && z0 >= bounds.min_z && z0 <= bounds.max_z;
    let in1 = x1 >= bounds.min_x && x1 <= bounds.max_x && z1 >= bounds.min_z && z1 <= bounds.max_z;
    in0 != in1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_inside_tile_does_not_cross() {
        let bounds = TileBounds { min_x: 0.0, min_z: 0.0, max_x: 512.0, max_z: 512.0 };
        assert!(!crosses_boundary(100.0, 100.0, 200.0, 200.0, &bounds));
    }

    #[test]
    fn segment_crossing_east_boundary() {
        let bounds = TileBounds { min_x: 0.0, min_z: 0.0, max_x: 512.0, max_z: 512.0 };
        assert!(crosses_boundary(500.0, 256.0, 520.0, 256.0, &bounds));
    }
}
