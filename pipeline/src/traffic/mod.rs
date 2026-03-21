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
            controlled_edge_ids: vec![],
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
