pub mod road;
pub mod building;
pub mod elevation;

use crate::parse::types::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub surface_type: Option<SurfaceType>,
}

#[derive(Debug, Default)]
pub struct GeneratedMeshes {
    pub road_meshes: Vec<(Road, MeshData)>,
    pub building_meshes: Vec<(Building, MeshData)>,
}

pub fn generate_meshes(parsed: &ParsedOsmData) -> Result<GeneratedMeshes> {
    let origin = parsed.origin.unwrap_or(Coordinate { lat: 22.5, lon: 114.0 });
    let mut result = GeneratedMeshes::default();

    for r in &parsed.roads {
        let mesh = road::extrude_road(r, &origin)?;
        result.road_meshes.push((r.clone(), mesh));
    }

    for b in &parsed.buildings {
        let mesh = building::extrude_building(b, &origin)?;
        result.building_meshes.push((b.clone(), mesh));
    }

    Ok(result)
}
