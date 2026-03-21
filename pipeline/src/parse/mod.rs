pub mod types;
pub mod fallbacks;

use crate::config::Fallbacks;
use types::*;
use anyhow::Result;

pub fn parse_osm(_xml: &str, _fallbacks: &Fallbacks) -> Result<ParsedOsmData> {
    Ok(ParsedOsmData::default())
}
