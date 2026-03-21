pub mod boundary;

use crate::config::PipelineConfig;
use crate::mesh::{GeneratedMeshes, MeshData};
use crate::traffic::lane_graph::TrafficGraph;
use crate::parse::types::*;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileBounds {
    pub min_x: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_z: f32,
}

#[derive(Debug)]
pub struct Tile {
    pub coord: TileCoord,
    pub bounds: TileBounds,
    pub road_meshes: Vec<MeshData>,
    pub building_meshes: Vec<MeshData>,
    pub traffic_graph: TrafficGraph,
    pub signal_positions: Vec<(f32, f32)>,
}

/// Determine which tile a world-space point belongs to.
pub fn world_to_tile(x: f32, z: f32, tile_size: f64) -> TileCoord {
    TileCoord {
        x: (x as f64 / tile_size).floor() as i32,
        y: (z as f64 / tile_size).floor() as i32,
    }
}

/// Check if a mesh has any vertex within the given tile bounds.
fn mesh_overlaps_tile(mesh: &MeshData, bounds: &TileBounds) -> bool {
    mesh.positions.iter().any(|p| {
        p[0] >= bounds.min_x && p[0] <= bounds.max_x &&
        p[2] >= bounds.min_z && p[2] <= bounds.max_z
    })
}

/// Chunk all generated data into tiles.
pub fn chunk_into_tiles(
    meshes: &GeneratedMeshes,
    traffic_graph: &TrafficGraph,
    _parsed: &ParsedOsmData,
    config: &PipelineConfig,
) -> Result<Vec<Tile>> {
    let tile_size = config.tile_size as f32;

    // Determine tile range from all mesh positions
    let all_positions: Vec<&[f32; 3]> = meshes.road_meshes.iter().flat_map(|(_, m)| m.positions.iter())
        .chain(meshes.building_meshes.iter().flat_map(|(_, m)| m.positions.iter()))
        .collect();

    if all_positions.is_empty() {
        return Ok(vec![]);
    }

    let min_x = all_positions.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
    let max_x = all_positions.iter().map(|p| p[0]).fold(f32::NEG_INFINITY, f32::max);
    let min_z = all_positions.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);
    let max_z = all_positions.iter().map(|p| p[2]).fold(f32::NEG_INFINITY, f32::max);

    let tile_min = world_to_tile(min_x, min_z, config.tile_size);
    let tile_max = world_to_tile(max_x, max_z, config.tile_size);

    let mut tiles = Vec::new();

    for tx in tile_min.x..=tile_max.x {
        for ty in tile_min.y..=tile_max.y {
            let bounds = TileBounds {
                min_x: tx as f32 * tile_size,
                min_z: ty as f32 * tile_size,
                max_x: (tx + 1) as f32 * tile_size,
                max_z: (ty + 1) as f32 * tile_size,
            };

            // Collect road meshes that overlap this tile
            let tile_roads: Vec<MeshData> = meshes.road_meshes
                .iter()
                .filter(|(_, m)| mesh_overlaps_tile(m, &bounds))
                .map(|(_, m)| m.clone())
                .collect();

            // Collect building meshes that overlap this tile
            let tile_buildings: Vec<MeshData> = meshes.building_meshes
                .iter()
                .filter(|(_, m)| mesh_overlaps_tile(m, &bounds))
                .map(|(_, m)| m.clone())
                .collect();

            // Filter traffic graph nodes within this tile
            let tile_traffic = filter_traffic_graph(traffic_graph, &bounds);

            // Collect signal positions
            let tile_signals: Vec<(f32, f32)> = traffic_graph.signals
                .iter()
                .filter(|s| s.x >= bounds.min_x && s.x <= bounds.max_x && s.z >= bounds.min_z && s.z <= bounds.max_z)
                .map(|s| (s.x, s.z))
                .collect();

            if tile_roads.is_empty() && tile_buildings.is_empty() {
                continue; // Skip empty tiles
            }

            tiles.push(Tile {
                coord: TileCoord { x: tx, y: ty },
                bounds,
                road_meshes: tile_roads,
                building_meshes: tile_buildings,
                traffic_graph: tile_traffic,
                signal_positions: tile_signals,
            });
        }
    }

    Ok(tiles)
}

fn filter_traffic_graph(graph: &TrafficGraph, bounds: &TileBounds) -> TrafficGraph {
    let mut result = TrafficGraph::default();

    // Include nodes within bounds (with margin for boundary features)
    let margin = 50.0; // meters
    let node_in_tile: Vec<bool> = graph.nodes.iter().map(|n| {
        n.x >= bounds.min_x - margin && n.x <= bounds.max_x + margin &&
        n.z >= bounds.min_z - margin && n.z <= bounds.max_z + margin
    }).collect();

    // Re-map node IDs
    let mut node_map = std::collections::HashMap::new();
    for (i, node) in graph.nodes.iter().enumerate() {
        if node_in_tile[i] {
            let new_id = result.add_node(node.x, node.z, node.y);
            node_map.insert(node.id, new_id);
        }
    }

    // Include edges where both endpoints are in the tile
    for edge in &graph.edges {
        if let (Some(&from), Some(&to)) = (node_map.get(&edge.from), node_map.get(&edge.to)) {
            result.add_edge(from, to, edge.speed_limit_kmh, edge.lane_index, edge.road_id, edge.layer);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_to_tile_positive() {
        let t = world_to_tile(100.0, 200.0, 512.0);
        assert_eq!(t.x, 0);
        assert_eq!(t.y, 0);
    }

    #[test]
    fn world_to_tile_crosses_boundary() {
        let t = world_to_tile(600.0, 600.0, 512.0);
        assert_eq!(t.x, 1);
        assert_eq!(t.y, 1);
    }

    #[test]
    fn world_to_tile_negative() {
        let t = world_to_tile(-100.0, -200.0, 512.0);
        assert_eq!(t.x, -1);
        assert_eq!(t.y, -1);
    }
}
