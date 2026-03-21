use super::lane_graph::SignalPhase;

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
