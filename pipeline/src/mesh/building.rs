use crate::parse::types::*;
use super::MeshData;
use anyhow::Result;

/// Extrude a building footprint to its height, creating walls and a flat roof.
pub fn extrude_building(building: &Building, origin: &Coordinate) -> Result<MeshData> {
    if building.footprint.len() < 3 {
        return Ok(MeshData::default());
    }

    let points: Vec<(f32, f32)> = building
        .footprint
        .iter()
        .map(|c| {
            let (x, z) = c.to_local_meters(origin);
            (x as f32, z as f32)
        })
        .collect();

    let height = building.height;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    let n = points.len();

    // Walls: for each edge of the footprint, create a quad
    for i in 0..n {
        let j = (i + 1) % n;
        let (x0, z0) = points[i];
        let (x1, z1) = points[j];

        // Wall normal (outward facing)
        let dx = x1 - x0;
        let dz = z1 - z0;
        let len = (dx * dx + dz * dz).sqrt();
        let nx = -dz / len;
        let nz = dx / len;

        let base = positions.len() as u32;

        // Bottom-left, bottom-right, top-right, top-left
        positions.push([x0, 0.0, z0]);
        positions.push([x1, 0.0, z1]);
        positions.push([x1, height, z1]);
        positions.push([x0, height, z0]);

        normals.push([nx, 0.0, nz]);
        normals.push([nx, 0.0, nz]);
        normals.push([nx, 0.0, nz]);
        normals.push([nx, 0.0, nz]);

        // Two triangles for the quad
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);
        indices.push(base);
        indices.push(base + 2);
        indices.push(base + 3);
    }

    // Roof: simple fan triangulation from first vertex
    let roof_base = positions.len() as u32;
    for &(x, z) in &points {
        positions.push([x, height, z]);
        normals.push([0.0, 1.0, 0.0]);
    }
    for i in 1..(n as u32 - 1) {
        indices.push(roof_base);
        indices.push(roof_base + i);
        indices.push(roof_base + i + 1);
    }

    Ok(MeshData {
        positions,
        normals,
        indices,
        surface_type: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_square_building() -> Building {
        Building {
            id: 1,
            footprint: vec![
                Coordinate { lat: 22.530, lon: 114.050 },
                Coordinate { lat: 22.530, lon: 114.0505 },
                Coordinate { lat: 22.5305, lon: 114.0505 },
                Coordinate { lat: 22.5305, lon: 114.050 },
            ],
            height: 30.0,
            building_type: BuildingType::Commercial,
        }
    }

    #[test]
    fn square_building_produces_walls_and_roof() {
        let origin = Coordinate { lat: 22.530, lon: 114.050 };
        let mesh = extrude_building(&make_square_building(), &origin).unwrap();

        // 4 walls * 4 vertices + 4 roof vertices = 20
        assert_eq!(mesh.positions.len(), 20);
        // 4 walls * 6 indices + 2 roof triangles * 3 = 30
        assert_eq!(mesh.indices.len(), 30);
    }

    #[test]
    fn roof_is_at_correct_height() {
        let origin = Coordinate { lat: 22.530, lon: 114.050 };
        let mesh = extrude_building(&make_square_building(), &origin).unwrap();

        // Last 4 vertices are the roof
        let roof_start = mesh.positions.len() - 4;
        for i in roof_start..mesh.positions.len() {
            assert_eq!(mesh.positions[i][1], 30.0);
        }
    }

    #[test]
    fn degenerate_footprint_returns_empty() {
        let origin = Coordinate { lat: 22.530, lon: 114.050 };
        let building = Building {
            id: 1, footprint: vec![Coordinate { lat: 22.53, lon: 114.05 }],
            height: 10.0, building_type: BuildingType::Other,
        };
        let mesh = extrude_building(&building, &origin).unwrap();
        assert!(mesh.positions.is_empty());
    }
}
