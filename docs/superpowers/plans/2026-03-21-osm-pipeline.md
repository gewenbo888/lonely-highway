# OSM Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust CLI tool that fetches Shenzhen OpenStreetMap data, processes it into road meshes, building shells, and traffic lane graphs, then exports Unity-ready tile assets (glTF + JSON).

**Architecture:** A multi-stage pipeline: Fetch (Overpass API) → Parse (OSM XML/PBF) → Process (road mesh extrusion, building extrusion, traffic graph construction, elevated/tunnel handling) → Chunk (512m tiles with boundary duplication) → Export (glTF meshes + traffic JSON + tile metadata). Each stage is a standalone module with clear input/output types.

**Tech Stack:** Rust, `osmpbf` crate (OSM parsing), `reqwest` (HTTP for Overpass API), `gltf-json`/`gltf` (glTF export), `serde`/`serde_json` (JSON serialization), `geo` crate (geometry operations), `clap` (CLI args)

**Spec reference:** `docs/superpowers/specs/2026-03-21-lonely-highway-design.md` — Section 2 (OSM Pipeline)

---

## File Structure

```
pipeline/
  Cargo.toml                          — Workspace root
  src/
    main.rs                           — CLI entry point, stage orchestration
    config.rs                         — Pipeline configuration (bounding box, tile size, fallbacks)
    fetch/
      mod.rs                          — Overpass API fetcher
    parse/
      mod.rs                          — OSM data parser, extract roads/buildings/signals
      types.rs                        — Parsed data types (Road, Building, Signal, etc.)
      fallbacks.rs                    — Default values for missing OSM tags
    mesh/
      mod.rs                          — Mesh generation orchestrator
      road.rs                         — Road centerline → mesh extrusion
      building.rs                     — Building footprint → shell extrusion
      elevation.rs                    — Layer/bridge/tunnel vertical positioning
    traffic/
      mod.rs                          — Traffic graph builder
      lane_graph.rs                   — Directed lane graph with intersections
      signals.rs                      — Signal phase generation
    tile/
      mod.rs                          — Tile chunking and boundary handling
      boundary.rs                     — Feature duplication at tile edges, shared edge IDs
    export/
      mod.rs                          — Export orchestrator
      gltf.rs                         — glTF mesh writer
      traffic_json.rs                 — Traffic graph JSON writer
      metadata.rs                     — Tile metadata JSON writer
      minimap.rs                      — Pre-baked minimap texture per tile
  tests/
    integration/
      fetch_test.rs                   — Integration test: real Overpass API call (small area)
      pipeline_test.rs                — End-to-end: small bbox → tile output
    fixtures/
      small_area.osm.xml             — Cached OSM data for offline tests
```

---

### Task 1: Rust Project Setup

**Files:**
- Create: `pipeline/Cargo.toml`
- Create: `pipeline/src/main.rs`
- Create: `pipeline/src/config.rs`

- [ ] **Step 1: Create Cargo project**

```bash
cd /Users/geir/Game
cargo init pipeline
```

- [ ] **Step 2: Add dependencies to Cargo.toml**

```toml
[package]
name = "lonely-highway-pipeline"
version = "0.1.0"
edition = "2021"

[dependencies]
osmpbf = "0.3"
reqwest = { version = "0.12", features = ["blocking", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
clap = { version = "4", features = ["derive"] }
geo = "0.28"
geo-types = "0.7"
gltf-json = "1"
anyhow = "1"
log = "0.4"
env_logger = "0.11"
image = { version = "0.25", features = ["png"] }

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3: Write config.rs**

```rust
// pipeline/src/config.rs
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "lonely-highway-pipeline")]
#[command(about = "Generate Unity tile assets from OpenStreetMap data for Lonely Highway")]
pub struct CliArgs {
    /// Bounding box: south,west,north,east (decimal degrees)
    #[arg(long, value_delimiter = ',')]
    pub bbox: Vec<f64>,

    /// Output directory for generated tiles
    #[arg(long, default_value = "output")]
    pub output: String,

    /// Tile size in meters
    #[arg(long, default_value_t = 512.0)]
    pub tile_size: f64,

    /// Skip fetch stage (use cached OSM data)
    #[arg(long)]
    pub cached: Option<String>,

    /// Only process a single tile (x,y)
    #[arg(long, value_delimiter = ',')]
    pub single_tile: Option<Vec<i32>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub bbox: BoundingBox,
    pub tile_size: f64,
    pub output_dir: String,
    pub fallbacks: Fallbacks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fallbacks {
    pub lane_count: LaneCountFallbacks,
    pub speed_limit: SpeedLimitFallbacks,
    pub building_height: BuildingHeightFallbacks,
    pub signal_timing: SignalTimingFallbacks,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LaneCountFallbacks {
    pub motorway: u8,
    pub primary: u8,
    pub secondary: u8,
    pub tertiary: u8,
    pub residential: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpeedLimitFallbacks {
    pub motorway: f32,
    pub primary: f32,
    pub secondary: f32,
    pub tertiary: f32,
    pub residential: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildingHeightFallbacks {
    pub commercial: f32,
    pub residential: f32,
    pub industrial: f32,
    pub default: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignalTimingFallbacks {
    pub two_way: f32,
    pub four_way: f32,
    pub complex: f32,
}

impl Default for Fallbacks {
    fn default() -> Self {
        Fallbacks {
            lane_count: LaneCountFallbacks {
                motorway: 4, primary: 3, secondary: 2, tertiary: 1, residential: 1,
            },
            speed_limit: SpeedLimitFallbacks {
                motorway: 100.0, primary: 60.0, secondary: 40.0, tertiary: 30.0, residential: 30.0,
            },
            building_height: BuildingHeightFallbacks {
                commercial: 40.0, residential: 25.0, industrial: 12.0, default: 10.0,
            },
            signal_timing: SignalTimingFallbacks {
                two_way: 60.0, four_way: 90.0, complex: 120.0,
            },
        }
    }
}

impl PipelineConfig {
    pub fn from_args(args: &CliArgs) -> anyhow::Result<Self> {
        if args.bbox.len() != 4 {
            anyhow::bail!("bbox must have exactly 4 values: south,west,north,east");
        }
        Ok(PipelineConfig {
            bbox: BoundingBox {
                south: args.bbox[0],
                west: args.bbox[1],
                north: args.bbox[2],
                east: args.bbox[3],
            },
            tile_size: args.tile_size,
            output_dir: args.output.clone(),
            fallbacks: Fallbacks::default(),
        })
    }
}
```

- [ ] **Step 4: Write main.rs skeleton**

```rust
// pipeline/src/main.rs
mod config;
mod fetch;
mod parse;
mod mesh;
mod traffic;
mod tile;
mod export;

use clap::Parser;
use config::{CliArgs, PipelineConfig};

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = CliArgs::parse();
    let config = PipelineConfig::from_args(&args)?;

    log::info!("Lonely Highway Pipeline");
    log::info!("Bounding box: {:?}", config.bbox);
    log::info!("Tile size: {}m", config.tile_size);
    log::info!("Output: {}", config.output_dir);

    // Stage 1: Fetch
    let osm_data = if let Some(cached_path) = &args.cached {
        log::info!("Loading cached OSM data from {}", cached_path);
        std::fs::read_to_string(cached_path)?
    } else {
        log::info!("Fetching OSM data from Overpass API...");
        fetch::fetch_osm_data(&config.bbox)?
    };

    // Stage 2: Parse
    log::info!("Parsing OSM data...");
    let parsed = parse::parse_osm(&osm_data, &config.fallbacks)?;
    log::info!(
        "Parsed: {} roads, {} buildings, {} signals",
        parsed.roads.len(),
        parsed.buildings.len(),
        parsed.signals.len()
    );

    // Stage 3: Generate meshes
    log::info!("Generating meshes...");
    let meshes = mesh::generate_meshes(&parsed)?;

    // Stage 4: Build traffic graph
    log::info!("Building traffic graph...");
    let traffic_graph = traffic::build_traffic_graph(&parsed)?;

    // Stage 5: Chunk into tiles
    log::info!("Chunking into {}m tiles...", config.tile_size);
    let tiles = tile::chunk_into_tiles(&meshes, &traffic_graph, &parsed, &config)?;
    log::info!("Generated {} tiles", tiles.len());

    // Stage 6: Export
    log::info!("Exporting tiles to {}...", config.output_dir);
    std::fs::create_dir_all(&config.output_dir)?;
    for tile in &tiles {
        export::export_tile(tile, &config.output_dir)?;
    }

    log::info!("Pipeline complete!");
    Ok(())
}
```

- [ ] **Step 5: Create module stubs**

Create empty module files so the project compiles:
- `pipeline/src/fetch/mod.rs`
- `pipeline/src/parse/mod.rs`
- `pipeline/src/parse/types.rs`
- `pipeline/src/parse/fallbacks.rs`
- `pipeline/src/mesh/mod.rs`
- `pipeline/src/mesh/road.rs`
- `pipeline/src/mesh/building.rs`
- `pipeline/src/mesh/elevation.rs`
- `pipeline/src/traffic/mod.rs`
- `pipeline/src/traffic/lane_graph.rs`
- `pipeline/src/traffic/signals.rs`
- `pipeline/src/tile/mod.rs`
- `pipeline/src/tile/boundary.rs`
- `pipeline/src/export/mod.rs`
- `pipeline/src/export/gltf.rs`
- `pipeline/src/export/traffic_json.rs`
- `pipeline/src/export/metadata.rs`
- `pipeline/src/export/minimap.rs`

Each stub should have the function signatures from main.rs returning `Ok(Default)` or empty vecs.

- [ ] **Step 6: Verify it compiles**

```bash
cd /Users/geir/Game/pipeline && cargo check
```

- [ ] **Step 7: Commit**

```bash
git add pipeline/
git commit -m "feat: scaffold Rust OSM pipeline with config, CLI, and module stubs"
```

---

### Task 2: Parsed Data Types

**Files:**
- Create: `pipeline/src/parse/types.rs`
- Modify: `pipeline/src/parse/mod.rs`

- [ ] **Step 1: Write test for parsed types**

```rust
// In pipeline/src/parse/types.rs, add tests at bottom
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn road_default_surface_is_asphalt() {
        let road = Road {
            id: 1,
            nodes: vec![],
            highway_class: HighwayClass::Primary,
            lanes_forward: 3,
            lanes_backward: 3,
            speed_limit_kmh: 60.0,
            surface: SurfaceType::Asphalt,
            is_oneway: false,
            layer: 0,
            is_bridge: false,
            is_tunnel: false,
            name: None,
        };
        assert_eq!(road.surface, SurfaceType::Asphalt);
    }

    #[test]
    fn building_height_from_levels() {
        let b = Building {
            id: 1,
            footprint: vec![],
            height: 30.0,
            building_type: BuildingType::Commercial,
        };
        assert_eq!(b.height, 30.0);
    }

    #[test]
    fn coordinate_to_meters_near_shenzhen() {
        let origin = Coordinate { lat: 22.5, lon: 114.0 };
        let point = Coordinate { lat: 22.501, lon: 114.001 };
        let (dx, dy) = point.to_local_meters(&origin);
        // ~111m per degree lat, ~102m per degree lon at 22.5N
        assert!((dy - 111.0).abs() < 5.0);
        assert!((dx - 102.0).abs() < 5.0);
    }
}
```

- [ ] **Step 2: Write parsed types**

```rust
// pipeline/src/parse/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coordinate {
    pub lat: f64,
    pub lon: f64,
}

impl Coordinate {
    /// Convert to local meters relative to an origin point.
    /// Uses equirectangular approximation (good enough for city-scale).
    pub fn to_local_meters(&self, origin: &Coordinate) -> (f64, f64) {
        let lat_rad = origin.lat.to_radians();
        let meters_per_degree_lat = 111_320.0;
        let meters_per_degree_lon = 111_320.0 * lat_rad.cos();

        let dx = (self.lon - origin.lon) * meters_per_degree_lon;
        let dy = (self.lat - origin.lat) * meters_per_degree_lat;
        (dx, dy)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HighwayClass {
    Motorway,
    MotorwayLink,
    Trunk,
    TrunkLink,
    Primary,
    PrimaryLink,
    Secondary,
    SecondaryLink,
    Tertiary,
    TertiaryLink,
    Residential,
    Service,
    Unclassified,
}

impl HighwayClass {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "motorway" => Some(Self::Motorway),
            "motorway_link" => Some(Self::MotorwayLink),
            "trunk" => Some(Self::Trunk),
            "trunk_link" => Some(Self::TrunkLink),
            "primary" => Some(Self::Primary),
            "primary_link" => Some(Self::PrimaryLink),
            "secondary" => Some(Self::Secondary),
            "secondary_link" => Some(Self::SecondaryLink),
            "tertiary" => Some(Self::Tertiary),
            "tertiary_link" => Some(Self::TertiaryLink),
            "residential" => Some(Self::Residential),
            "service" => Some(Self::Service),
            "unclassified" => Some(Self::Unclassified),
            _ => None,
        }
    }

    pub fn is_link(&self) -> bool {
        matches!(self,
            Self::MotorwayLink | Self::TrunkLink |
            Self::PrimaryLink | Self::SecondaryLink | Self::TertiaryLink)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceType {
    Asphalt,
    Concrete,
    PaintedLine,
    Gravel,
    Grass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingType {
    Residential,
    Commercial,
    Industrial,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Road {
    pub id: u64,
    pub nodes: Vec<Coordinate>,
    pub highway_class: HighwayClass,
    pub lanes_forward: u8,
    pub lanes_backward: u8,
    pub speed_limit_kmh: f32,
    pub surface: SurfaceType,
    pub is_oneway: bool,
    pub layer: i8,
    pub is_bridge: bool,
    pub is_tunnel: bool,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: u64,
    pub footprint: Vec<Coordinate>,
    pub height: f32,
    pub building_type: BuildingType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSignal {
    pub id: u64,
    pub position: Coordinate,
    pub cycle_time: f32,
    pub connected_road_ids: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crosswalk {
    pub id: u64,
    pub nodes: Vec<Coordinate>,
    pub signal_id: Option<u64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ParsedOsmData {
    pub roads: Vec<Road>,
    pub buildings: Vec<Building>,
    pub signals: Vec<TrafficSignal>,
    pub crosswalks: Vec<Crosswalk>,
    pub origin: Option<Coordinate>,
}
```

- [ ] **Step 3: Run tests**

```bash
cd /Users/geir/Game/pipeline && cargo test parse::types
```

- [ ] **Step 4: Commit**

```bash
git add pipeline/src/parse/
git commit -m "feat: define parsed OSM data types (Road, Building, Signal, Coordinate)"
```

---

### Task 3: Overpass API Fetcher

**Files:**
- Create: `pipeline/src/fetch/mod.rs`

- [ ] **Step 1: Write test**

```rust
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
```

- [ ] **Step 2: Write fetcher**

```rust
// pipeline/src/fetch/mod.rs
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
```

- [ ] **Step 3: Run tests**

```bash
cd /Users/geir/Game/pipeline && cargo test fetch
```

- [ ] **Step 4: Commit**

```bash
git add pipeline/src/fetch/
git commit -m "feat: implement Overpass API fetcher with configurable bounding box"
```

---

### Task 4: OSM XML Parser

**Files:**
- Create: `pipeline/src/parse/mod.rs`
- Create: `pipeline/src/parse/fallbacks.rs`
- Create: `pipeline/tests/fixtures/small_area.osm.xml`

- [ ] **Step 1: Create a small test fixture**

Save a minimal OSM XML snippet to `pipeline/tests/fixtures/small_area.osm.xml` containing:
- 2 road ways (one primary, one residential) with nodes
- 1 building way with nodes
- 1 traffic signal node
- Various tags (highway, lanes, maxspeed, building, height)

- [ ] **Step 2: Write fallbacks.rs**

```rust
// pipeline/src/parse/fallbacks.rs
use crate::config::Fallbacks;
use crate::parse::types::{HighwayClass, BuildingType};

impl Fallbacks {
    pub fn lane_count_for(&self, class: &HighwayClass) -> u8 {
        match class {
            HighwayClass::Motorway | HighwayClass::MotorwayLink => self.lane_count.motorway,
            HighwayClass::Trunk | HighwayClass::TrunkLink => self.lane_count.primary,
            HighwayClass::Primary | HighwayClass::PrimaryLink => self.lane_count.primary,
            HighwayClass::Secondary | HighwayClass::SecondaryLink => self.lane_count.secondary,
            HighwayClass::Tertiary | HighwayClass::TertiaryLink => self.lane_count.tertiary,
            _ => self.lane_count.residential,
        }
    }

    pub fn speed_limit_for(&self, class: &HighwayClass) -> f32 {
        match class {
            HighwayClass::Motorway | HighwayClass::MotorwayLink => self.speed_limit.motorway,
            HighwayClass::Trunk | HighwayClass::TrunkLink => self.speed_limit.primary,
            HighwayClass::Primary | HighwayClass::PrimaryLink => self.speed_limit.primary,
            HighwayClass::Secondary | HighwayClass::SecondaryLink => self.speed_limit.secondary,
            HighwayClass::Tertiary | HighwayClass::TertiaryLink => self.speed_limit.tertiary,
            _ => self.speed_limit.residential,
        }
    }

    pub fn building_height_for(&self, btype: &BuildingType) -> f32 {
        match btype {
            BuildingType::Commercial => self.building_height.commercial,
            BuildingType::Residential => self.building_height.residential,
            BuildingType::Industrial => self.building_height.industrial,
            BuildingType::Other => self.building_height.default,
        }
    }
}
```

- [ ] **Step 3: Write OSM XML parser**

```rust
// pipeline/src/parse/mod.rs
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

    // Simple XML parsing using string scanning
    // For production, use quick-xml crate. This is a minimal implementation.
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

                // Check for traffic signal in self-closing node with tags
                // (handled below if node has child tags)
            }

            // Self-closing node with traffic_signals tag inline
            if line.contains("traffic_signals") || line.ends_with("/>") {
                // Will be handled when we find the tag
            }
        }

        // Node with child tags (traffic signals)
        if line.starts_with("<node") && !line.ends_with("/>") {
            if let Some(id) = extract_attr(line, "id") {
                current_way_id = Some(id.parse().unwrap_or(0));
                current_tags.clear();
                // We reuse way tracking for nodes temporarily
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
                        nodes: way_coords,
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
                    footprint: way_coords,
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
```

- [ ] **Step 5: Run tests**

```bash
cd /Users/geir/Game/pipeline && cargo test parse
```

- [ ] **Step 6: Commit**

```bash
git add pipeline/src/parse/ pipeline/tests/
git commit -m "feat: implement OSM XML parser with fallback strategy"
```

---

### Task 5: Road Mesh Generation

**Files:**
- Create: `pipeline/src/mesh/road.rs`
- Create: `pipeline/src/mesh/elevation.rs`
- Modify: `pipeline/src/mesh/mod.rs`

- [ ] **Step 1: Write mesh types and road extrusion tests**

```rust
// pipeline/src/mesh/mod.rs
pub mod road;
pub mod building;
pub mod elevation;

use crate::parse::types::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,   // x, y, z
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub surface_type: Option<crate::parse::types::SurfaceType>,
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
```

- [ ] **Step 2: Write elevation module**

```rust
// pipeline/src/mesh/elevation.rs

/// Calculate vertical offset for a road based on layer, bridge, and tunnel tags.
/// Default 8m per layer for elevated roads.
/// Tunnels go below ground.
pub fn vertical_offset(layer: i8, is_bridge: bool, is_tunnel: bool) -> f32 {
    let layer_height = 8.0; // meters per layer

    if is_tunnel {
        // Tunnels go below ground level
        return (layer as f32) * layer_height - layer_height;
    }

    if is_bridge || layer > 0 {
        return (layer as f32).max(1.0) * layer_height;
    }

    // Ground-level road
    (layer as f32) * layer_height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ground_level_returns_zero() {
        assert_eq!(vertical_offset(0, false, false), 0.0);
    }

    #[test]
    fn bridge_returns_elevated() {
        let offset = vertical_offset(1, true, false);
        assert_eq!(offset, 8.0);
    }

    #[test]
    fn layer_2_bridge_returns_16m() {
        assert_eq!(vertical_offset(2, true, false), 16.0);
    }

    #[test]
    fn tunnel_goes_below() {
        let offset = vertical_offset(0, false, true);
        assert!(offset < 0.0);
    }

    #[test]
    fn bridge_with_zero_layer_still_elevated() {
        let offset = vertical_offset(0, true, false);
        assert_eq!(offset, 8.0);
    }
}
```

- [ ] **Step 3: Write road mesh extrusion**

```rust
// pipeline/src/mesh/road.rs
use crate::parse::types::*;
use super::{MeshData, elevation};
use anyhow::Result;

const LANE_WIDTH: f32 = 3.5; // meters
const CURB_WIDTH: f32 = 0.3;

/// Extrude a road centerline into a flat mesh strip.
/// Each segment between two nodes becomes a quad (2 triangles).
pub fn extrude_road(road: &Road, origin: &Coordinate) -> Result<MeshData> {
    if road.nodes.len() < 2 {
        return Ok(MeshData::default());
    }

    let total_lanes = road.lanes_forward + road.lanes_backward;
    let road_half_width = (total_lanes as f32 * LANE_WIDTH + CURB_WIDTH * 2.0) / 2.0;
    let y_offset = elevation::vertical_offset(road.layer, road.is_bridge, road.is_tunnel);

    // Convert nodes to local meters
    let points: Vec<(f32, f32)> = road
        .nodes
        .iter()
        .map(|c| {
            let (x, z) = c.to_local_meters(origin);
            (x as f32, z as f32)
        })
        .collect();

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    for i in 0..points.len() {
        let (x, z) = points[i];

        // Calculate perpendicular direction
        let (dx, dz) = if i < points.len() - 1 {
            let next = points[i + 1];
            (next.0 - x, next.1 - z)
        } else {
            let prev = points[i - 1];
            (x - prev.0, z - prev.1)
        };

        let len = (dx * dx + dz * dz).sqrt();
        if len < 0.001 {
            continue;
        }

        // Perpendicular (rotate 90 degrees)
        let perp_x = -dz / len;
        let perp_z = dx / len;

        // Left and right edges
        let left_x = x + perp_x * road_half_width;
        let left_z = z + perp_z * road_half_width;
        let right_x = x - perp_x * road_half_width;
        let right_z = z - perp_z * road_half_width;

        let base_idx = positions.len() as u32;

        // Left vertex
        positions.push([left_x, y_offset, left_z]);
        normals.push([0.0, 1.0, 0.0]);

        // Right vertex
        positions.push([right_x, y_offset, right_z]);
        normals.push([0.0, 1.0, 0.0]);

        // Add triangles for the quad between this segment and the previous
        if i > 0 {
            let prev_left = base_idx - 2;
            let prev_right = base_idx - 1;
            let curr_left = base_idx;
            let curr_right = base_idx + 1;

            // Triangle 1
            indices.push(prev_left);
            indices.push(curr_left);
            indices.push(prev_right);

            // Triangle 2
            indices.push(prev_right);
            indices.push(curr_left);
            indices.push(curr_right);
        }
    }

    Ok(MeshData {
        positions,
        normals,
        indices,
        surface_type: Some(road.surface),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_straight_road() -> Road {
        Road {
            id: 1,
            nodes: vec![
                Coordinate { lat: 22.530, lon: 114.050 },
                Coordinate { lat: 22.531, lon: 114.050 },
            ],
            highway_class: HighwayClass::Primary,
            lanes_forward: 2,
            lanes_backward: 2,
            speed_limit_kmh: 60.0,
            surface: SurfaceType::Asphalt,
            is_oneway: false,
            layer: 0,
            is_bridge: false,
            is_tunnel: false,
            name: None,
        }
    }

    #[test]
    fn straight_road_produces_quad() {
        let origin = Coordinate { lat: 22.530, lon: 114.050 };
        let road = make_straight_road();
        let mesh = extrude_road(&road, &origin).unwrap();

        assert_eq!(mesh.positions.len(), 4); // 2 nodes * 2 edges
        assert_eq!(mesh.indices.len(), 6);   // 1 quad = 2 triangles = 6 indices
    }

    #[test]
    fn road_width_matches_lanes() {
        let origin = Coordinate { lat: 22.530, lon: 114.050 };
        let road = make_straight_road();
        let mesh = extrude_road(&road, &origin).unwrap();

        // 4 lanes * 3.5m + 0.6m curb = 14.6m total, half = 7.3m
        let left_x = mesh.positions[0][0];
        let right_x = mesh.positions[1][0];
        let width = (left_x - right_x).abs();
        assert!((width - 14.6).abs() < 0.5, "Road width should be ~14.6m, got {}", width);
    }

    #[test]
    fn bridge_road_is_elevated() {
        let origin = Coordinate { lat: 22.530, lon: 114.050 };
        let mut road = make_straight_road();
        road.is_bridge = true;
        road.layer = 1;
        let mesh = extrude_road(&road, &origin).unwrap();

        assert_eq!(mesh.positions[0][1], 8.0, "Bridge should be at 8m height");
    }

    #[test]
    fn empty_road_returns_empty_mesh() {
        let origin = Coordinate { lat: 22.530, lon: 114.050 };
        let road = Road {
            id: 1, nodes: vec![], highway_class: HighwayClass::Primary,
            lanes_forward: 2, lanes_backward: 2, speed_limit_kmh: 60.0,
            surface: SurfaceType::Asphalt, is_oneway: false, layer: 0,
            is_bridge: false, is_tunnel: false, name: None,
        };
        let mesh = extrude_road(&road, &origin).unwrap();
        assert!(mesh.positions.is_empty());
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cd /Users/geir/Game/pipeline && cargo test mesh
```

- [ ] **Step 5: Commit**

```bash
git add pipeline/src/mesh/
git commit -m "feat: implement road mesh extrusion with elevation handling"
```

---

### Task 6: Building Shell Generation

**Files:**
- Create: `pipeline/src/mesh/building.rs`

- [ ] **Step 1: Write tests and implementation**

```rust
// pipeline/src/mesh/building.rs
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
```

- [ ] **Step 2: Run tests**

```bash
cd /Users/geir/Game/pipeline && cargo test mesh::building
```

- [ ] **Step 3: Commit**

```bash
git add pipeline/src/mesh/building.rs
git commit -m "feat: implement building shell extrusion with walls and roof"
```

---

### Task 7: Traffic Lane Graph

**Files:**
- Create: `pipeline/src/traffic/lane_graph.rs`
- Create: `pipeline/src/traffic/signals.rs`
- Modify: `pipeline/src/traffic/mod.rs`

- [ ] **Step 1: Write lane graph types and builder**

```rust
// pipeline/src/traffic/lane_graph.rs
use serde::{Deserialize, Serialize};

pub type NodeId = u64;
pub type EdgeId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneNode {
    pub id: NodeId,
    pub x: f32,
    pub z: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneEdge {
    pub id: EdgeId,
    pub from: NodeId,
    pub to: NodeId,
    pub speed_limit_kmh: f32,
    pub lane_index: u8,
    pub road_id: u64,
    pub layer: i8,
    pub length: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalPhase {
    Green,
    Yellow,
    Red,
    LeftTurnArrow,
    PedestrianWalk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalController {
    pub id: u64,
    pub x: f32,
    pub z: f32,
    pub cycle_time: f32,
    pub phases: Vec<(SignalPhase, f32)>, // (phase, duration in seconds)
    pub controlled_edge_ids: Vec<EdgeId>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TrafficGraph {
    pub nodes: Vec<LaneNode>,
    pub edges: Vec<LaneEdge>,
    pub signals: Vec<SignalController>,
}

impl TrafficGraph {
    pub fn add_node(&mut self, x: f32, z: f32, y: f32) -> NodeId {
        let id = self.nodes.len() as NodeId;
        self.nodes.push(LaneNode { id, x, z, y });
        id
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId, speed_limit: f32, lane_index: u8, road_id: u64, layer: i8) -> EdgeId {
        let id = self.edges.len() as EdgeId;

        let from_node = &self.nodes[from as usize];
        let to_node = &self.nodes[to as usize];
        let dx = to_node.x - from_node.x;
        let dz = to_node.z - from_node.z;
        let length = (dx * dx + dz * dz).sqrt();

        self.edges.push(LaneEdge {
            id, from, to, speed_limit_kmh: speed_limit, lane_index, road_id, layer, length,
        });
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_nodes_and_edges() {
        let mut graph = TrafficGraph::default();
        let n0 = graph.add_node(0.0, 0.0, 0.0);
        let n1 = graph.add_node(100.0, 0.0, 0.0);
        let e0 = graph.add_edge(n0, n1, 60.0, 0, 1, 0);

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert!((graph.edges[0].length - 100.0).abs() < 0.1);
    }

    #[test]
    fn edge_length_calculated_correctly() {
        let mut graph = TrafficGraph::default();
        let n0 = graph.add_node(0.0, 0.0, 0.0);
        let n1 = graph.add_node(30.0, 0.0, 40.0);
        graph.add_edge(n0, n1, 60.0, 0, 1, 0);

        assert!((graph.edges[0].length - 50.0).abs() < 0.1); // 3-4-5 triangle
    }
}
```

- [ ] **Step 2: Write traffic graph builder from parsed roads**

```rust
// pipeline/src/traffic/mod.rs
pub mod lane_graph;
pub mod signals;

use crate::parse::types::*;
use crate::mesh::elevation;
use lane_graph::*;
use anyhow::Result;

/// Build a directed lane graph from parsed road data.
pub fn build_traffic_graph(parsed: &ParsedOsmData) -> Result<TrafficGraph> {
    let origin = parsed.origin.unwrap_or(Coordinate { lat: 22.5, lon: 114.0 });
    let mut graph = TrafficGraph::default();

    for road in &parsed.roads {
        if road.nodes.len() < 2 {
            continue;
        }

        let y = elevation::vertical_offset(road.layer, road.is_bridge, road.is_tunnel);

        // Create nodes for each road point
        let node_ids: Vec<NodeId> = road
            .nodes
            .iter()
            .map(|c| {
                let (x, z) = c.to_local_meters(&origin);
                graph.add_node(x as f32, z as f32, y)
            })
            .collect();

        // Create forward lane edges
        for lane in 0..road.lanes_forward {
            for i in 0..node_ids.len() - 1 {
                graph.add_edge(
                    node_ids[i], node_ids[i + 1],
                    road.speed_limit_kmh, lane, road.id, road.layer,
                );
            }
        }

        // Create backward lane edges (if not one-way)
        if !road.is_oneway {
            for lane in 0..road.lanes_backward {
                for i in (1..node_ids.len()).rev() {
                    graph.add_edge(
                        node_ids[i], node_ids[i - 1],
                        road.speed_limit_kmh, lane, road.id, road.layer,
                    );
                }
            }
        }
    }

    // Add signal controllers
    for signal in &parsed.signals {
        let (x, z) = signal.position.to_local_meters(&origin);
        graph.signals.push(SignalController {
            id: signal.id,
            x: x as f32,
            z: z as f32,
            cycle_time: signal.cycle_time,
            phases: signals::default_signal_phases(signal.cycle_time),
            controlled_edge_ids: vec![], // Wired during tile chunking when we know nearby edges
        });
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Fallbacks;
    use crate::parse;

    #[test]
    fn builds_graph_from_parsed_roads() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<osm version="0.6">
  <node id="1" lat="22.530" lon="114.050"/>
  <node id="2" lat="22.531" lon="114.050"/>
  <way id="10">
    <nd ref="1"/>
    <nd ref="2"/>
    <tag k="highway" v="primary"/>
    <tag k="lanes" v="4"/>
  </way>
</osm>"#;
        let parsed = parse::parse_osm(xml, &Fallbacks::default()).unwrap();
        let graph = build_traffic_graph(&parsed).unwrap();

        // 2 nodes, 4 edges (2 fwd lanes + 2 bwd lanes)
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 4);
    }

    #[test]
    fn oneway_road_has_only_forward_edges() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<osm version="0.6">
  <node id="1" lat="22.530" lon="114.050"/>
  <node id="2" lat="22.531" lon="114.050"/>
  <way id="10">
    <nd ref="1"/>
    <nd ref="2"/>
    <tag k="highway" v="residential"/>
    <tag k="oneway" v="yes"/>
    <tag k="lanes" v="2"/>
  </way>
</osm>"#;
        let parsed = parse::parse_osm(xml, &Fallbacks::default()).unwrap();
        let graph = build_traffic_graph(&parsed).unwrap();

        assert_eq!(graph.edges.len(), 2); // 2 forward lanes only
        // All edges should go from node 0 to node 1
        for edge in &graph.edges {
            assert_eq!(edge.from, 0);
            assert_eq!(edge.to, 1);
        }
    }
}
```

- [ ] **Step 3: Write signal defaults**

```rust
// pipeline/src/traffic/signals.rs
use super::lane_graph::SignalPhase;

/// Generate default signal phase timing for a given cycle time.
pub fn default_signal_phases(cycle_time: f32) -> Vec<(SignalPhase, f32)> {
    let green_time = cycle_time * 0.45;
    let yellow_time = 3.0;
    let red_time = cycle_time - green_time - yellow_time;

    vec![
        (SignalPhase::Green, green_time),
        (SignalPhase::Yellow, yellow_time),
        (SignalPhase::Red, red_time),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_durations_sum_to_cycle_time() {
        let phases = default_signal_phases(90.0);
        let total: f32 = phases.iter().map(|(_, d)| d).sum();
        assert!((total - 90.0).abs() < 0.01);
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cd /Users/geir/Game/pipeline && cargo test traffic
```

- [ ] **Step 5: Commit**

```bash
git add pipeline/src/traffic/
git commit -m "feat: implement traffic lane graph builder with signal phases"
```

---

### Task 8: Tile Chunking

**Files:**
- Create: `pipeline/src/tile/mod.rs`
- Create: `pipeline/src/tile/boundary.rs`

- [ ] **Step 1: Write tile types and chunking logic**

```rust
// pipeline/src/tile/mod.rs
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
    parsed: &ParsedOsmData,
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
```

- [ ] **Step 2: Write boundary stub**

```rust
// pipeline/src/tile/boundary.rs
// Boundary duplication and shared edge IDs for tile seam stitching.
// Features crossing tile boundaries get duplicated into both tiles.
// Each duplicate carries a shared edge ID for runtime stitching.

use super::TileBounds;

/// Unique ID for a shared boundary feature.
pub type SharedEdgeId = u64;

/// Check if a line segment crosses a tile boundary.
pub fn crosses_boundary(x0: f32, z0: f32, x1: f32, z1: f32, bounds: &TileBounds) -> bool {
    let in0 = x0 >= bounds.min_x && x0 <= bounds.max_x && z0 >= bounds.min_z && z0 <= bounds.max_z;
    let in1 = x1 >= bounds.min_x && x1 <= bounds.max_x && z1 >= bounds.min_z && z1 <= bounds.max_z;
    in0 != in1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_inside_tile_does_not_cross() {
        let bounds = TileBounds { min_x: 0.0, min_z: 0.0, max_x: 512.0, max_z: 512.0 };
        assert!(!crosses_boundary(100.0, 100.0, 200.0, 200.0, &bounds));
    }

    #[test]
    fn segment_crossing_east_boundary() {
        let bounds = TileBounds { min_x: 0.0, min_z: 0.0, max_x: 512.0, max_z: 512.0 };
        assert!(crosses_boundary(500.0, 256.0, 520.0, 256.0, &bounds));
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cd /Users/geir/Game/pipeline && cargo test tile
```

- [ ] **Step 4: Commit**

```bash
git add pipeline/src/tile/
git commit -m "feat: implement tile chunking with boundary detection"
```

---

### Task 9: glTF Export

**Files:**
- Create: `pipeline/src/export/gltf.rs`
- Create: `pipeline/src/export/traffic_json.rs`
- Create: `pipeline/src/export/metadata.rs`
- Create: `pipeline/src/export/minimap.rs`
- Modify: `pipeline/src/export/mod.rs`

- [ ] **Step 1: Write export module**

```rust
// pipeline/src/export/mod.rs
pub mod gltf;
pub mod traffic_json;
pub mod metadata;
pub mod minimap;

use crate::tile::Tile;
use anyhow::Result;
use std::path::Path;

pub fn export_tile(tile: &Tile, output_dir: &str) -> Result<()> {
    let dir = Path::new(output_dir);
    let prefix = format!("tile_{}_{}", tile.coord.x, tile.coord.y);

    // Export road + building meshes as glTF
    gltf::export_tile_gltf(tile, &dir.join(format!("{}.glb", prefix)))?;

    // Export traffic graph as JSON
    traffic_json::export_traffic_json(tile, &dir.join(format!("{}_traffic.json", prefix)))?;

    // Export tile metadata
    metadata::export_metadata(tile, &dir.join(format!("{}_meta.json", prefix)))?;

    // Export minimap tile
    minimap::export_minimap(tile, &dir.join(format!("{}_minimap.png", prefix)))?;

    Ok(())
}
```

- [ ] **Step 2: Write glTF exporter**

Write `pipeline/src/export/gltf.rs` that combines all road and building meshes in a tile into a single binary glTF (.glb) file. Use the `gltf-json` crate to construct the JSON, then write the binary buffer.

The implementation should:
- Combine all mesh positions/normals/indices into a single buffer
- Create separate mesh primitives for roads and buildings
- Write the .glb with embedded buffer

- [ ] **Step 3: Write traffic JSON exporter**

```rust
// pipeline/src/export/traffic_json.rs
use crate::tile::Tile;
use anyhow::Result;
use std::path::Path;

pub fn export_traffic_json(tile: &Tile, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(&tile.traffic_graph)?;
    std::fs::write(path, json)?;
    log::info!("Wrote traffic graph: {}", path.display());
    Ok(())
}
```

- [ ] **Step 4: Write metadata exporter**

```rust
// pipeline/src/export/metadata.rs
use crate::tile::Tile;
use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct TileMetadata {
    coord_x: i32,
    coord_y: i32,
    bounds: TileBoundsJson,
    road_count: usize,
    building_count: usize,
    traffic_nodes: usize,
    traffic_edges: usize,
    signal_positions: Vec<[f32; 2]>,
}

#[derive(Serialize)]
struct TileBoundsJson {
    min_x: f32,
    min_z: f32,
    max_x: f32,
    max_z: f32,
}

pub fn export_metadata(tile: &Tile, path: &Path) -> Result<()> {
    let meta = TileMetadata {
        coord_x: tile.coord.x,
        coord_y: tile.coord.y,
        bounds: TileBoundsJson {
            min_x: tile.bounds.min_x,
            min_z: tile.bounds.min_z,
            max_x: tile.bounds.max_x,
            max_z: tile.bounds.max_z,
        },
        road_count: tile.road_meshes.len(),
        building_count: tile.building_meshes.len(),
        traffic_nodes: tile.traffic_graph.nodes.len(),
        traffic_edges: tile.traffic_graph.edges.len(),
        signal_positions: tile.signal_positions.iter().map(|(x, z)| [*x, *z]).collect(),
    };
    let json = serde_json::to_string_pretty(&meta)?;
    std::fs::write(path, json)?;
    log::info!("Wrote metadata: {}", path.display());
    Ok(())
}
```

- [ ] **Step 5: Write minimap exporter (placeholder)**

```rust
// pipeline/src/export/minimap.rs
use crate::tile::Tile;
use anyhow::Result;
use std::path::Path;

/// Generate a top-down minimap image for this tile.
/// Roads are drawn as white lines on a dark background.
pub fn export_minimap(tile: &Tile, path: &Path) -> Result<()> {
    let size = 256u32; // pixels
    let tile_size = tile.bounds.max_x - tile.bounds.min_x;
    let scale = size as f32 / tile_size;

    let mut img = image::RgbaImage::new(size, size);

    // Dark background
    for pixel in img.pixels_mut() {
        *pixel = image::Rgba([30, 30, 40, 255]);
    }

    // Draw road center lines as white pixels
    for mesh in &tile.road_meshes {
        for pos in &mesh.positions {
            let px = ((pos[0] - tile.bounds.min_x) * scale) as i32;
            let py = size as i32 - ((pos[2] - tile.bounds.min_z) * scale) as i32;
            if px >= 0 && px < size as i32 && py >= 0 && py < size as i32 {
                img.put_pixel(px as u32, py as u32, image::Rgba([220, 220, 220, 255]));
            }
        }
    }

    img.save(path)?;
    log::info!("Wrote minimap: {}", path.display());
    Ok(())
}
```

- [ ] **Step 6: Run all tests**

```bash
cd /Users/geir/Game/pipeline && cargo test
```

- [ ] **Step 7: Commit**

```bash
git add pipeline/src/export/
git commit -m "feat: implement tile export (glTF, traffic JSON, metadata, minimap)"
```

---

### Task 10: End-to-End Integration Test

**Files:**
- Create: `pipeline/tests/integration/pipeline_test.rs`

- [ ] **Step 1: Write integration test**

```rust
// pipeline/tests/integration/pipeline_test.rs
// Run the full pipeline on a small hardcoded OSM snippet and verify output.

use std::process::Command;

#[test]
fn pipeline_produces_tile_files() {
    let output_dir = tempfile::tempdir().unwrap();

    // Write test OSM data
    let osm_path = output_dir.path().join("test.osm.xml");
    std::fs::write(&osm_path, include_str!("../fixtures/small_area.osm.xml")).unwrap();

    // Run pipeline
    let status = Command::new("cargo")
        .args(["run", "--",
            "--bbox", "22.529,114.049,22.533,114.053",
            "--output", output_dir.path().join("tiles").to_str().unwrap(),
            "--cached", osm_path.to_str().unwrap(),
            "--tile-size", "512",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .unwrap();

    assert!(status.success(), "Pipeline should exit successfully");

    // Verify output files exist
    let tiles_dir = output_dir.path().join("tiles");
    assert!(tiles_dir.exists(), "Output directory should exist");

    // Should have at least one tile with glb, traffic json, metadata, and minimap
    let entries: Vec<_> = std::fs::read_dir(&tiles_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    assert!(!entries.is_empty(), "Should produce at least one tile");

    // Check for expected file types
    let file_names: Vec<String> = entries.iter().map(|e| e.file_name().to_string_lossy().to_string()).collect();
    assert!(file_names.iter().any(|f| f.ends_with("_meta.json")), "Should have metadata JSON");
    assert!(file_names.iter().any(|f| f.ends_with("_traffic.json")), "Should have traffic JSON");
    assert!(file_names.iter().any(|f| f.ends_with("_minimap.png")), "Should have minimap PNG");
}
```

- [ ] **Step 2: Create fixture file**

Save the test XML from Task 4 into `pipeline/tests/fixtures/small_area.osm.xml`.

- [ ] **Step 3: Run integration test**

```bash
cd /Users/geir/Game/pipeline && cargo test pipeline_produces_tile_files -- --nocapture
```

- [ ] **Step 4: Commit**

```bash
git add pipeline/tests/
git commit -m "feat: add end-to-end pipeline integration test"
```

---

### Task 11: CLI Polish & First Run

**Files:**
- Modify: `pipeline/src/main.rs`

- [ ] **Step 1: Test with a real Shenzhen bounding box**

Fetch a small area of Futian CBD:
```bash
cd /Users/geir/Game/pipeline
RUST_LOG=info cargo run -- \
    --bbox 22.530,114.050,22.540,114.060 \
    --output ../unity-project/Assets/GeneratedTiles \
    --tile-size 512
```

This is a ~1km x 1km area. Verify:
- Pipeline completes without errors
- Output directory has tile files (.glb, _traffic.json, _meta.json, _minimap.png)
- Metadata JSON has reasonable values

- [ ] **Step 2: Cache the fetched data**

Save the downloaded OSM data for future development:
```bash
mkdir -p /Users/geir/Game/pipeline/cache
# Re-run with output capture, or add a --save-cache flag
```

- [ ] **Step 3: Commit**

```bash
git add pipeline/
git commit -m "feat: pipeline CLI ready for Futian CBD test area"
```

---

## Summary

| Task | Component | Tests |
|------|-----------|-------|
| 1 | Rust project setup | — |
| 2 | Parsed data types | 3 unit tests |
| 3 | Overpass API fetcher | 1 unit test |
| 4 | OSM XML parser | 5 unit tests |
| 5 | Road mesh generation | 4 unit + 5 elevation tests |
| 6 | Building shell generation | 3 unit tests |
| 7 | Traffic lane graph | 5 unit tests |
| 8 | Tile chunking | 5 unit tests |
| 9 | Export (glTF, JSON, minimap) | — |
| 10 | End-to-end integration test | 1 integration test |
| 11 | CLI polish & first run | — (manual) |

**Total: 11 tasks, ~32 unit tests, 1 integration test**

## Known Simplifications (to revisit later)

- OSM XML parser is hand-rolled string scanning — production should use `quick-xml` crate
- glTF export is minimal — no materials, UV coordinates, or LOD meshes
- Boundary duplication is stubbed — full shared-edge-ID system needed for seamless tile borders
- No SRTM terrain data integration yet — ground is flat per tile
- Road mesh doesn't model intersections (no junction geometry)
- No LOD generation — single detail level per tile
