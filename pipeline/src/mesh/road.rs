use crate::parse::types::*;
use super::{MeshData, elevation};
use anyhow::Result;

const LANE_WIDTH: f32 = 3.5; // meters
const CURB_WIDTH: f32 = 0.3;

/// Extrude a road centerline into a flat mesh strip.
/// Each segment between two nodes becomes a quad (2 triangles).
pub fn extrude_road(road: &Road, origin: &Coordinate) -> Result<MeshData> {
    if road.nodes.len() < 2 {
        return Ok(MeshData::default());
    }

    let total_lanes = road.lanes_forward + road.lanes_backward;
    let road_half_width = (total_lanes as f32 * LANE_WIDTH + CURB_WIDTH * 2.0) / 2.0;
    let y_offset = elevation::vertical_offset(road.layer, road.is_bridge, road.is_tunnel);

    // Convert nodes to local meters
    let points: Vec<(f32, f32)> = road
        .nodes
        .iter()
        .map(|c| {
            let (x, z) = c.to_local_meters(origin);
            (x as f32, z as f32)
        })
        .collect();

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    for i in 0..points.len() {
        let (x, z) = points[i];

        // Calculate perpendicular direction
        let (dx, dz) = if i < points.len() - 1 {
            let next = points[i + 1];
            (next.0 - x, next.1 - z)
        } else {
            let prev = points[i - 1];
            (x - prev.0, z - prev.1)
        };

        let len = (dx * dx + dz * dz).sqrt();
        if len < 0.001 {
            continue;
        }

        // Perpendicular (rotate 90 degrees)
        let perp_x = -dz / len;
        let perp_z = dx / len;

        // Left and right edges
        let left_x = x + perp_x * road_half_width;
        let left_z = z + perp_z * road_half_width;
        let right_x = x - perp_x * road_half_width;
        let right_z = z - perp_z * road_half_width;

        let base_idx = positions.len() as u32;

        // Left vertex
        positions.push([left_x, y_offset, left_z]);
        normals.push([0.0, 1.0, 0.0]);

        // Right vertex
        positions.push([right_x, y_offset, right_z]);
        normals.push([0.0, 1.0, 0.0]);

        // Add triangles for the quad between this segment and the previous
        if i > 0 {
            let prev_left = base_idx - 2;
            let prev_right = base_idx - 1;
            let curr_left = base_idx;
            let curr_right = base_idx + 1;

            // Triangle 1
            indices.push(prev_left);
            indices.push(curr_left);
            indices.push(prev_right);

            // Triangle 2
            indices.push(prev_right);
            indices.push(curr_left);
            indices.push(curr_right);
        }
    }

    Ok(MeshData {
        positions,
        normals,
        indices,
        surface_type: Some(road.surface),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_straight_road() -> Road {
        Road {
            id: 1,
            nodes: vec![
                Coordinate { lat: 22.530, lon: 114.050 },
                Coordinate { lat: 22.531, lon: 114.050 },
            ],
            highway_class: HighwayClass::Primary,
            lanes_forward: 2,
            lanes_backward: 2,
            speed_limit_kmh: 60.0,
            surface: SurfaceType::Asphalt,
            is_oneway: false,
            layer: 0,
            is_bridge: false,
            is_tunnel: false,
            name: None,
        }
    }

    #[test]
    fn straight_road_produces_quad() {
        let origin = Coordinate { lat: 22.530, lon: 114.050 };
        let road = make_straight_road();
        let mesh = extrude_road(&road, &origin).unwrap();

        assert_eq!(mesh.positions.len(), 4); // 2 nodes * 2 edges
        assert_eq!(mesh.indices.len(), 6);   // 1 quad = 2 triangles = 6 indices
    }

    #[test]
    fn road_width_matches_lanes() {
        let origin = Coordinate { lat: 22.530, lon: 114.050 };
        let road = make_straight_road();
        let mesh = extrude_road(&road, &origin).unwrap();

        // 4 lanes * 3.5m + 0.6m curb = 14.6m total, half = 7.3m
        let left_x = mesh.positions[0][0];
        let right_x = mesh.positions[1][0];
        let width = (left_x - right_x).abs();
        assert!((width - 14.6).abs() < 0.5, "Road width should be ~14.6m, got {}", width);
    }

    #[test]
    fn bridge_road_is_elevated() {
        let origin = Coordinate { lat: 22.530, lon: 114.050 };
        let mut road = make_straight_road();
        road.is_bridge = true;
        road.layer = 1;
        let mesh = extrude_road(&road, &origin).unwrap();

        assert_eq!(mesh.positions[0][1], 8.0, "Bridge should be at 8m height");
    }

    #[test]
    fn empty_road_returns_empty_mesh() {
        let origin = Coordinate { lat: 22.530, lon: 114.050 };
        let road = Road {
            id: 1, nodes: vec![], highway_class: HighwayClass::Primary,
            lanes_forward: 2, lanes_backward: 2, speed_limit_kmh: 60.0,
            surface: SurfaceType::Asphalt, is_oneway: false, layer: 0,
            is_bridge: false, is_tunnel: false, name: None,
        };
        let mesh = extrude_road(&road, &origin).unwrap();
        assert!(mesh.positions.is_empty());
    }
}
