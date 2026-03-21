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
