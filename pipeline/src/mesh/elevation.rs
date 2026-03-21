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
