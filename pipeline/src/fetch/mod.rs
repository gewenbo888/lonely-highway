use crate::config::BoundingBox;
use anyhow::Result;

pub fn build_overpass_query(_bbox: &BoundingBox) -> String {
    String::new()
}

pub fn fetch_osm_data(_bbox: &BoundingBox) -> Result<String> {
    Ok(String::new())
}
