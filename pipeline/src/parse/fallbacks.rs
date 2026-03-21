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
