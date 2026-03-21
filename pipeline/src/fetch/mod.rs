use crate::config::BoundingBox;
use anyhow::Result;

const OVERPASS_URL: &str = "https://overpass-api.de/api/interpreter";

pub fn build_overpass_query(bbox: &BoundingBox) -> String {
    format!(
        r#"[out:xml][timeout:300];
(
  way["highway"]({south},{west},{north},{east});
  way["building"]({south},{west},{north},{east});
  node["highway"="traffic_signals"]({south},{west},{north},{east});
  way["footway"="crossing"]({south},{west},{north},{east});
);
(._;>;);
out body;"#,
        south = bbox.south,
        west = bbox.west,
        north = bbox.north,
        east = bbox.east,
    )
}

pub fn fetch_osm_data(bbox: &BoundingBox) -> Result<String> {
    let query = build_overpass_query(bbox);
    log::info!("Sending Overpass query ({} bytes)...", query.len());

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()?;

    let response = client
        .post(OVERPASS_URL)
        .body(query)
        .send()?;

    if !response.status().is_success() {
        anyhow::bail!("Overpass API returned status: {}", response.status());
    }

    let body = response.text()?;
    log::info!("Received {} bytes of OSM data", body.len());
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BoundingBox;

    #[test]
    fn overpass_query_contains_bbox() {
        let bbox = BoundingBox {
            south: 22.52, west: 114.05, north: 22.56, east: 114.10,
        };
        let query = build_overpass_query(&bbox);
        assert!(query.contains("22.52"));
        assert!(query.contains("114.1"));
        assert!(query.contains("highway"));
        assert!(query.contains("building"));
    }
}
