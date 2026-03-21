use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coordinate {
    pub lat: f64,
    pub lon: f64,
}

impl Coordinate {
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
    Motorway, MotorwayLink, Trunk, TrunkLink, Primary, PrimaryLink,
    Secondary, SecondaryLink, Tertiary, TertiaryLink, Residential, Service, Unclassified,
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
pub enum SurfaceType { Asphalt, Concrete, PaintedLine, Gravel, Grass }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingType { Residential, Commercial, Industrial, Other }

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
