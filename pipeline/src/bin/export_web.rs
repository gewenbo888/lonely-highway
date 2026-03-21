/// Export pipeline data as a single JSON file for the Three.js prototype.
/// Usage: cargo run --bin export_web -- --bbox 22.530,114.050,22.545,114.070
use std::io::Write;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    let cached = args.iter().position(|a| a == "--cached").map(|i| args[i + 1].clone());
    let bbox_str = args.iter().position(|a| a == "--bbox").map(|i| args[i + 1].clone())
        .unwrap_or_else(|| "22.530,114.050,22.545,114.070".to_string());
    let output = args.iter().position(|a| a == "--output").map(|i| args[i + 1].clone())
        .unwrap_or_else(|| "city.json".to_string());

    let parts: Vec<f64> = bbox_str.split(',').map(|s| s.parse().unwrap()).collect();
    let bbox = lonely_highway_pipeline::config::BoundingBox {
        south: parts[0], west: parts[1], north: parts[2], east: parts[3],
    };

    let osm_data = if let Some(path) = cached {
        std::fs::read_to_string(path)?
    } else {
        lonely_highway_pipeline::fetch::fetch_osm_data(&bbox)?
    };

    let fallbacks = lonely_highway_pipeline::config::Fallbacks::default();
    let parsed = lonely_highway_pipeline::parse::parse_osm(&osm_data, &fallbacks)?;
    let origin = parsed.origin.unwrap_or(lonely_highway_pipeline::parse::types::Coordinate { lat: 22.5375, lon: 114.06 });

    eprintln!("Parsed: {} roads, {} buildings, {} signals", parsed.roads.len(), parsed.buildings.len(), parsed.signals.len());

    // Convert roads to line segments in local meters
    let mut road_lines: Vec<serde_json::Value> = Vec::new();
    for road in &parsed.roads {
        let points: Vec<[f64; 3]> = road.nodes.iter().map(|c| {
            let (x, z) = c.to_local_meters(&origin);
            let y = lonely_highway_pipeline::mesh::elevation::vertical_offset(road.layer, road.is_bridge, road.is_tunnel) as f64;
            [x, y, z]
        }).collect();

        let lanes = road.lanes_forward + road.lanes_backward;
        road_lines.push(serde_json::json!({
            "points": points,
            "lanes": lanes,
            "speed": road.speed_limit_kmh,
            "bridge": road.is_bridge,
            "tunnel": road.is_tunnel,
            "class": format!("{:?}", road.highway_class),
            "name": road.name,
        }));
    }

    // Convert buildings to footprints in local meters
    let mut building_data: Vec<serde_json::Value> = Vec::new();
    for b in &parsed.buildings {
        let footprint: Vec<[f64; 2]> = b.footprint.iter().map(|c| {
            let (x, z) = c.to_local_meters(&origin);
            [x, z]
        }).collect();

        building_data.push(serde_json::json!({
            "footprint": footprint,
            "height": b.height,
            "type": format!("{:?}", b.building_type),
        }));
    }

    // Signals
    let mut signal_data: Vec<serde_json::Value> = Vec::new();
    for s in &parsed.signals {
        let (x, z) = s.position.to_local_meters(&origin);
        signal_data.push(serde_json::json!({
            "x": x, "z": z, "cycle": s.cycle_time,
        }));
    }

    let output_json = serde_json::json!({
        "origin": { "lat": origin.lat, "lon": origin.lon },
        "roads": road_lines,
        "buildings": building_data,
        "signals": signal_data,
    });

    let mut file = std::fs::File::create(&output)?;
    file.write_all(serde_json::to_string(&output_json)?.as_bytes())?;
    eprintln!("Wrote {}", output);
    Ok(())
}
