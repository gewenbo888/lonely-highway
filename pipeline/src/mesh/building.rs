use crate::parse::types::*;
use super::MeshData;
use anyhow::Result;

pub fn extrude_building(_building: &Building, _origin: &Coordinate) -> Result<MeshData> {
    Ok(MeshData::default())
}
