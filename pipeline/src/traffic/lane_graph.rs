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
pub enum SignalPhase { Green, Yellow, Red, LeftTurnArrow, PedestrianWalk }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalController {
    pub id: u64,
    pub x: f32,
    pub z: f32,
    pub cycle_time: f32,
    pub phases: Vec<(SignalPhase, f32)>,
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
