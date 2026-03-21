pub mod types;
pub mod fallbacks;

use crate::config::Fallbacks;
use types::*;
use anyhow::Result;
use std::collections::HashMap;

/// Parse OSM XML data into structured game-ready types.
pub fn parse_osm(xml: &str, fallbacks: &Fallbacks) -> Result<ParsedOsmData> {
    let mut nodes: HashMap<u64, Coordinate> = HashMap::new();
    let mut roads = Vec::new();
    let mut buildings = Vec::new();
    let mut signals = Vec::new();
    let mut crosswalks = Vec::new();

    let mut current_way_id: Option<u64> = None;
    let mut current_way_nodes: Vec<u64> = Vec::new();
    let mut current_tags: HashMap<String, String> = HashMap::new();
    let mut in_way = false;

    for line in xml.lines() {
        let line = line.trim();

        // Parse nodes
        if line.starts_with("<node") && !line.contains("</node>") {
            if let (Some(id), Some(lat), Some(lon)) = (
                extract_attr(line, "id"),
                extract_attr(line, "lat"),
                extract_attr(line, "lon"),
            ) {
                let id: u64 = id.parse().unwrap_or(0);
                let lat: f64 = lat.parse().unwrap_or(0.0);
                let lon: f64 = lon.parse().unwrap_or(0.0);
                nodes.insert(id, Coordinate { lat, lon });
            }
        }

        // Node with child tags (traffic signals)
        if line.starts_with("<node") && !line.ends_with("/>") {
            if let Some(id) = extract_attr(line, "id") {
                current_way_id = Some(id.parse().unwrap_or(0));
                current_tags.clear();
            }
        }

        if line == "</node>" {
            if let Some(id) = current_way_id {
                if current_tags.get("highway").map(|v| v.as_str()) == Some("traffic_signals") {
                    if let Some(coord) = nodes.get(&id) {
                        signals.push(TrafficSignal {
                            id,
                            position: *coord,
                            cycle_time: fallbacks.signal_timing.four_way,
                            connected_road_ids: vec![],
                        });
                    }
                }
            }
            current_way_id = None;
        }

        // Parse ways
        if line.starts_with("<way") {
            in_way = true;
            current_way_nodes.clear();
            current_tags.clear();
            if let Some(id) = extract_attr(line, "id") {
                current_way_id = Some(id.parse().unwrap_or(0));
            }
        }

        if in_way && line.starts_with("<nd") {
            if let Some(ref_id) = extract_attr(line, "ref") {
                if let Ok(id) = ref_id.parse::<u64>() {
                    current_way_nodes.push(id);
                }
            }
        }

        if line.starts_with("<tag") {
            if let (Some(k), Some(v)) = (extract_attr(line, "k"), extract_attr(line, "v")) {
                current_tags.insert(k.to_string(), v.to_string());
            }
        }

        if line == "</way>" {
            in_way = false;
            let id = current_way_id.unwrap_or(0);

            // Resolve node coordinates
            let way_coords: Vec<Coordinate> = current_way_nodes
                .iter()
                .filter_map(|nid| nodes.get(nid).copied())
                .collect();

            if way_coords.is_empty() {
                continue;
            }

            // Is this a highway?
            if let Some(highway_val) = current_tags.get("highway") {
                if let Some(highway_class) = HighwayClass::from_str(highway_val) {
                    let lanes_total: u8 = current_tags
                        .get("lanes")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(fallbacks.lane_count_for(&highway_class) * 2);

                    let is_oneway = current_tags
                        .get("oneway")
                        .map(|v| v == "yes" || v == "1")
                        .unwrap_or(matches!(highway_class, HighwayClass::Motorway | HighwayClass::MotorwayLink));

                    let (lanes_fwd, lanes_bwd) = if is_oneway {
                        (lanes_total, 0)
                    } else {
                        (lanes_total / 2, lanes_total - lanes_total / 2)
                    };

                    let speed_limit = current_tags
                        .get("maxspeed")
                        .and_then(|v| v.trim_end_matches(" km/h").parse().ok())
                        .unwrap_or(fallbacks.speed_limit_for(&highway_class));

                    let surface = match current_tags.get("surface").map(|s| s.as_str()) {
                        Some("concrete") => SurfaceType::Concrete,
                        Some("gravel") => SurfaceType::Gravel,
                        Some("grass") => SurfaceType::Grass,
                        _ => SurfaceType::Asphalt,
                    };

                    let layer: i8 = current_tags
                        .get("layer")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);

                    let is_bridge = current_tags.get("bridge").map(|v| v == "yes").unwrap_or(false);
                    let is_tunnel = current_tags.get("tunnel").map(|v| v == "yes").unwrap_or(false);

                    roads.push(Road {
                        id,
                        nodes: way_coords.clone(),
                        highway_class,
                        lanes_forward: lanes_fwd,
                        lanes_backward: lanes_bwd,
                        speed_limit_kmh: speed_limit,
                        surface,
                        is_oneway,
                        layer,
                        is_bridge,
                        is_tunnel,
                        name: current_tags.get("name").cloned(),
                    });
                }
            }

            // Is this a building?
            if current_tags.contains_key("building") {
                let building_type = match current_tags.get("building").map(|s| s.as_str()) {
                    Some("commercial" | "retail" | "office") => BuildingType::Commercial,
                    Some("residential" | "apartments" | "house") => BuildingType::Residential,
                    Some("industrial" | "warehouse") => BuildingType::Industrial,
                    _ => BuildingType::Other,
                };

                let height: f32 = current_tags
                    .get("height")
                    .and_then(|v| v.trim_end_matches('m').trim().parse().ok())
                    .or_else(|| {
                        current_tags
                            .get("building:levels")
                            .and_then(|v| v.parse::<f32>().ok())
                            .map(|levels| levels * 3.0)
                    })
                    .unwrap_or(fallbacks.building_height_for(&building_type));

                buildings.push(Building {
                    id,
                    footprint: way_coords.clone(),
                    height,
                    building_type,
                });
            }

            // Is this a crosswalk?
            if current_tags.get("footway").map(|v| v.as_str()) == Some("crossing") {
                crosswalks.push(Crosswalk {
                    id,
                    nodes: way_coords,
                    signal_id: None,
                });
            }
        }
    }

    // Compute origin (center of bounding box from data)
    let origin = if !roads.is_empty() {
        let all_coords: Vec<&Coordinate> = roads.iter().flat_map(|r| r.nodes.iter()).collect();
        let avg_lat = all_coords.iter().map(|c| c.lat).sum::<f64>() / all_coords.len() as f64;
        let avg_lon = all_coords.iter().map(|c| c.lon).sum::<f64>() / all_coords.len() as f64;
        Some(Coordinate { lat: avg_lat, lon: avg_lon })
    } else {
        None
    };

    Ok(ParsedOsmData {
        roads,
        buildings,
        signals,
        crosswalks,
        origin,
    })
}

fn extract_attr<'a>(line: &'a str, attr: &str) -> Option<&'a str> {
    let pattern = format!("{}=\"", attr);
    let start = line.find(&pattern)? + pattern.len();
    let end = line[start..].find('"')? + start;
    Some(&line[start..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Fallbacks;

    fn test_xml() -> &'static str {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<osm version="0.6">
  <node id="1" lat="22.530" lon="114.050"/>
  <node id="2" lat="22.531" lon="114.051"/>
  <node id="3" lat="22.532" lon="114.050"/>
  <node id="4" lat="22.530" lon="114.049"/>
  <node id="5" lat="22.530" lon="114.051"/>
  <node id="6" lat="22.531" lon="114.051"/>
  <node id="7" lat="22.531" lon="114.049"/>
  <node id="8" lat="22.530" lon="114.049"/>
  <node id="100" lat="22.5305" lon="114.0505">
    <tag k="highway" v="traffic_signals"/>
  </node>
  <way id="10">
    <nd ref="1"/>
    <nd ref="2"/>
    <nd ref="3"/>
    <tag k="highway" v="primary"/>
    <tag k="lanes" v="4"/>
    <tag k="maxspeed" v="60"/>
    <tag k="name" v="Shennan Road"/>
  </way>
  <way id="11">
    <nd ref="4"/>
    <nd ref="5"/>
    <tag k="highway" v="residential"/>
    <tag k="oneway" v="yes"/>
  </way>
  <way id="20">
    <nd ref="5"/>
    <nd ref="6"/>
    <nd ref="7"/>
    <nd ref="8"/>
    <nd ref="5"/>
    <tag k="building" v="commercial"/>
    <tag k="height" v="45"/>
  </way>
</osm>"#
    }

    #[test]
    fn parses_roads() {
        let data = parse_osm(test_xml(), &Fallbacks::default()).unwrap();
        assert_eq!(data.roads.len(), 2);

        let primary = &data.roads[0];
        assert_eq!(primary.highway_class, HighwayClass::Primary);
        assert_eq!(primary.lanes_forward, 2);
        assert_eq!(primary.lanes_backward, 2);
        assert_eq!(primary.speed_limit_kmh, 60.0);
        assert_eq!(primary.name, Some("Shennan Road".to_string()));

        let residential = &data.roads[1];
        assert!(residential.is_oneway);
    }

    #[test]
    fn parses_buildings() {
        let data = parse_osm(test_xml(), &Fallbacks::default()).unwrap();
        assert_eq!(data.buildings.len(), 1);
        assert_eq!(data.buildings[0].height, 45.0);
        assert_eq!(data.buildings[0].building_type, BuildingType::Commercial);
    }

    #[test]
    fn parses_signals() {
        let data = parse_osm(test_xml(), &Fallbacks::default()).unwrap();
        assert_eq!(data.signals.len(), 1);
    }

    #[test]
    fn applies_fallbacks_when_tags_missing() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<osm version="0.6">
  <node id="1" lat="22.530" lon="114.050"/>
  <node id="2" lat="22.531" lon="114.051"/>
  <way id="10">
    <nd ref="1"/>
    <nd ref="2"/>
    <tag k="highway" v="secondary"/>
  </way>
</osm>"#;
        let data = parse_osm(xml, &Fallbacks::default()).unwrap();
        assert_eq!(data.roads[0].lanes_forward, 2);
        assert_eq!(data.roads[0].lanes_backward, 2);
        assert_eq!(data.roads[0].speed_limit_kmh, 40.0);
    }

    #[test]
    fn computes_origin() {
        let data = parse_osm(test_xml(), &Fallbacks::default()).unwrap();
        let origin = data.origin.unwrap();
        assert!(origin.lat > 22.52 && origin.lat < 22.54);
        assert!(origin.lon > 114.04 && origin.lon < 114.06);
    }
}
