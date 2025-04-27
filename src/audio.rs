pub fn generate_sine_wave(freq: f32, sample_rate: u32, duration_secs: u32) -> Vec<f32> {
    let len = sample_rate * duration_secs;
    (0..len)
        .flat_map(|i| {
            let t = i as f32 / sample_rate as f32;
            let sample = (2.0 * std::f32::consts::PI * freq * t).sin();
            [sample, sample]
        })
        .collect()
}
