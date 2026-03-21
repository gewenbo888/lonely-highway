pub mod lane_graph;
pub mod signals;

use crate::parse::types::*;
use lane_graph::*;
use anyhow::Result;

pub fn build_traffic_graph(_parsed: &ParsedOsmData) -> Result<TrafficGraph> {
    Ok(TrafficGraph::default())
}
