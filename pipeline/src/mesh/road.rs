use crate::parse::types::*;
use super::{MeshData, elevation};
use anyhow::Result;

pub fn extrude_road(_road: &Road, _origin: &Coordinate) -> Result<MeshData> {
    Ok(MeshData::default())
}
