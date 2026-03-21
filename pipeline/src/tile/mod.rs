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

pub fn chunk_into_tiles(
    _meshes: &GeneratedMeshes,
    _traffic_graph: &TrafficGraph,
    _parsed: &ParsedOsmData,
    _config: &PipelineConfig,
) -> Result<Vec<Tile>> {
    Ok(vec![])
}
