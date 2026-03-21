use crate::tile::Tile;
use crate::mesh::MeshData;
use anyhow::Result;
use std::path::Path;

/// Export a tile's meshes as a binary glTF (.glb) file.
/// Combines all road and building meshes into a single buffer with separate mesh primitives.
pub fn export_tile_gltf(tile: &Tile, path: &Path) -> Result<()> {
    let all_meshes: Vec<&MeshData> = tile.road_meshes.iter()
        .chain(tile.building_meshes.iter())
        .filter(|m| !m.positions.is_empty())
        .collect();

    if all_meshes.is_empty() {
        // Write an empty/minimal glb
        write_minimal_glb(path)?;
        return Ok(());
    }

    // Build a single combined buffer with all mesh data
    let mut buffer_data: Vec<u8> = Vec::new();
    let mut accessors: Vec<gltf_json::Accessor> = Vec::new();
    let mut buffer_views: Vec<gltf_json::buffer::View> = Vec::new();
    let mut primitives: Vec<gltf_json::mesh::Primitive> = Vec::new();

    for mesh in &all_meshes {
        let pos_view_idx = buffer_views.len() as u32;
        let pos_accessor_idx = accessors.len() as u32;

        // Positions buffer view
        let pos_offset = buffer_data.len();
        let mut pos_min = [f32::INFINITY; 3];
        let mut pos_max = [f32::NEG_INFINITY; 3];
        for p in &mesh.positions {
            for i in 0..3 {
                pos_min[i] = pos_min[i].min(p[i]);
                pos_max[i] = pos_max[i].max(p[i]);
                buffer_data.extend_from_slice(&p[i].to_le_bytes());
            }
        }
        let pos_byte_length = buffer_data.len() - pos_offset;

        buffer_views.push(gltf_json::buffer::View {
            buffer: gltf_json::Index::new(0),
            byte_length: gltf_json::validation::USize64(pos_byte_length as u64),
            byte_offset: Some(gltf_json::validation::USize64(pos_offset as u64)),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: Some(gltf_json::validation::Checked::Valid(gltf_json::buffer::Target::ArrayBuffer)),
        });

        accessors.push(gltf_json::Accessor {
            buffer_view: Some(gltf_json::Index::new(pos_view_idx)),
            byte_offset: Some(gltf_json::validation::USize64(0)),
            count: gltf_json::validation::USize64(mesh.positions.len() as u64),
            component_type: gltf_json::validation::Checked::Valid(gltf_json::accessor::GenericComponentType(gltf_json::accessor::ComponentType::F32)),
            extensions: Default::default(),
            extras: Default::default(),
            type_: gltf_json::validation::Checked::Valid(gltf_json::accessor::Type::Vec3),
            min: Some(gltf_json::Value::from(vec![
                serde_json::Value::from(pos_min[0]),
                serde_json::Value::from(pos_min[1]),
                serde_json::Value::from(pos_min[2]),
            ])),
            max: Some(gltf_json::Value::from(vec![
                serde_json::Value::from(pos_max[0]),
                serde_json::Value::from(pos_max[1]),
                serde_json::Value::from(pos_max[2]),
            ])),
            normalized: false,
            sparse: None,
        });

        // Normals buffer view
        let normal_view_idx = buffer_views.len() as u32;
        let normal_accessor_idx = accessors.len() as u32;
        let normal_offset = buffer_data.len();
        for n in &mesh.normals {
            for i in 0..3 {
                buffer_data.extend_from_slice(&n[i].to_le_bytes());
            }
        }
        let normal_byte_length = buffer_data.len() - normal_offset;

        buffer_views.push(gltf_json::buffer::View {
            buffer: gltf_json::Index::new(0),
            byte_length: gltf_json::validation::USize64(normal_byte_length as u64),
            byte_offset: Some(gltf_json::validation::USize64(normal_offset as u64)),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: Some(gltf_json::validation::Checked::Valid(gltf_json::buffer::Target::ArrayBuffer)),
        });

        accessors.push(gltf_json::Accessor {
            buffer_view: Some(gltf_json::Index::new(normal_view_idx)),
            byte_offset: Some(gltf_json::validation::USize64(0)),
            count: gltf_json::validation::USize64(mesh.normals.len() as u64),
            component_type: gltf_json::validation::Checked::Valid(gltf_json::accessor::GenericComponentType(gltf_json::accessor::ComponentType::F32)),
            extensions: Default::default(),
            extras: Default::default(),
            type_: gltf_json::validation::Checked::Valid(gltf_json::accessor::Type::Vec3),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });

        // Indices buffer view
        let idx_view_idx = buffer_views.len() as u32;
        let idx_accessor_idx = accessors.len() as u32;
        let idx_offset = buffer_data.len();
        for idx in &mesh.indices {
            buffer_data.extend_from_slice(&idx.to_le_bytes());
        }
        let idx_byte_length = buffer_data.len() - idx_offset;

        buffer_views.push(gltf_json::buffer::View {
            buffer: gltf_json::Index::new(0),
            byte_length: gltf_json::validation::USize64(idx_byte_length as u64),
            byte_offset: Some(gltf_json::validation::USize64(idx_offset as u64)),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: Some(gltf_json::validation::Checked::Valid(gltf_json::buffer::Target::ElementArrayBuffer)),
        });

        accessors.push(gltf_json::Accessor {
            buffer_view: Some(gltf_json::Index::new(idx_view_idx)),
            byte_offset: Some(gltf_json::validation::USize64(0)),
            count: gltf_json::validation::USize64(mesh.indices.len() as u64),
            component_type: gltf_json::validation::Checked::Valid(gltf_json::accessor::GenericComponentType(gltf_json::accessor::ComponentType::U32)),
            extensions: Default::default(),
            extras: Default::default(),
            type_: gltf_json::validation::Checked::Valid(gltf_json::accessor::Type::Scalar),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });

        // Build primitive
        let mut attributes = std::collections::BTreeMap::new();
        attributes.insert(
            gltf_json::validation::Checked::Valid(gltf_json::mesh::Semantic::Positions),
            gltf_json::Index::new(pos_accessor_idx),
        );
        attributes.insert(
            gltf_json::validation::Checked::Valid(gltf_json::mesh::Semantic::Normals),
            gltf_json::Index::new(normal_accessor_idx),
        );

        primitives.push(gltf_json::mesh::Primitive {
            attributes,
            extensions: Default::default(),
            extras: Default::default(),
            indices: Some(gltf_json::Index::new(idx_accessor_idx)),
            material: None,
            mode: gltf_json::validation::Checked::Valid(gltf_json::mesh::Mode::Triangles),
            targets: None,
        });
    }

    // Pad buffer to 4-byte alignment
    while buffer_data.len() % 4 != 0 {
        buffer_data.push(0);
    }

    let root = gltf_json::Root {
        accessors,
        buffers: vec![gltf_json::Buffer {
            byte_length: gltf_json::validation::USize64(buffer_data.len() as u64),
            extensions: Default::default(),
            extras: Default::default(),
            uri: None,
        }],
        buffer_views,
        meshes: vec![gltf_json::Mesh {
            extensions: Default::default(),
            extras: Default::default(),
            primitives,
            weights: None,
        }],
        nodes: vec![gltf_json::Node {
            mesh: Some(gltf_json::Index::new(0)),
            ..Default::default()
        }],
        scenes: vec![gltf_json::Scene {
            extensions: Default::default(),
            extras: Default::default(),
            nodes: vec![gltf_json::Index::new(0)],
        }],
        scene: Some(gltf_json::Index::new(0)),
        ..Default::default()
    };

    write_glb(path, &root, &buffer_data)?;

    log::info!("Wrote glTF: {} ({} bytes buffer)", path.display(), buffer_data.len());
    Ok(())
}

fn write_minimal_glb(path: &Path) -> Result<()> {
    let root = gltf_json::Root::default();
    write_glb(path, &root, &[])
}

fn write_glb(path: &Path, root: &gltf_json::Root, buffer_data: &[u8]) -> Result<()> {
    let json_string = serde_json::to_string(root)?;
    let mut json_bytes = json_string.into_bytes();
    // Pad JSON to 4-byte alignment with spaces
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }

    let total_length = 12 + 8 + json_bytes.len() + if !buffer_data.is_empty() { 8 + buffer_data.len() } else { 0 };

    let mut glb = Vec::with_capacity(total_length);

    // GLB header
    glb.extend_from_slice(b"glTF");                           // magic
    glb.extend_from_slice(&2u32.to_le_bytes());                // version
    glb.extend_from_slice(&(total_length as u32).to_le_bytes()); // total length

    // JSON chunk
    glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes());             // chunk type: JSON
    glb.extend_from_slice(&json_bytes);

    // Binary chunk (if any data)
    if !buffer_data.is_empty() {
        glb.extend_from_slice(&(buffer_data.len() as u32).to_le_bytes()); // chunk length
        glb.extend_from_slice(&0x004E4942u32.to_le_bytes());              // chunk type: BIN
        glb.extend_from_slice(buffer_data);
    }

    std::fs::write(path, glb)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{Tile, TileCoord, TileBounds};
    use crate::traffic::lane_graph::TrafficGraph;

    #[test]
    fn exports_valid_glb_file() {
        let tile = Tile {
            coord: TileCoord { x: 0, y: 0 },
            bounds: TileBounds { min_x: 0.0, min_z: 0.0, max_x: 512.0, max_z: 512.0 },
            road_meshes: vec![MeshData {
                positions: vec![[0.0, 0.0, 0.0], [100.0, 0.0, 0.0], [100.0, 0.0, 100.0], [0.0, 0.0, 100.0]],
                normals: vec![[0.0, 1.0, 0.0]; 4],
                indices: vec![0, 1, 2, 0, 2, 3],
                surface_type: None,
            }],
            building_meshes: vec![],
            traffic_graph: TrafficGraph::default(),
            signal_positions: vec![],
        };

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.glb");
        export_tile_gltf(&tile, &path).unwrap();

        let data = std::fs::read(&path).unwrap();
        // Check GLB magic
        assert_eq!(&data[0..4], b"glTF");
        // Check version
        assert_eq!(u32::from_le_bytes([data[4], data[5], data[6], data[7]]), 2);
        // File should be non-trivial size
        assert!(data.len() > 100);
    }

    #[test]
    fn empty_tile_exports_minimal_glb() {
        let tile = Tile {
            coord: TileCoord { x: 0, y: 0 },
            bounds: TileBounds { min_x: 0.0, min_z: 0.0, max_x: 512.0, max_z: 512.0 },
            road_meshes: vec![],
            building_meshes: vec![],
            traffic_graph: TrafficGraph::default(),
            signal_positions: vec![],
        };

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.glb");
        export_tile_gltf(&tile, &path).unwrap();

        let data = std::fs::read(&path).unwrap();
        assert_eq!(&data[0..4], b"glTF");
    }
}
