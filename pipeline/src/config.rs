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
