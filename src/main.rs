use anyhow::Result;
use wav::{FileFormat, FileType, FormatCreate, SampleType, WaveFile};

mod wav;

fn main() -> Result<()> {
    let format_create = FormatCreate {
        file_type: FileType::Wave,
        file_format: FileFormat::PCM,
        sample_type: SampleType::U16,
        chans: 2,
        sample_rate: 0x0000AC44,
    };
    let data = generate_sine_wave(440.0, 44100, 5);
    let _ = WaveFile::create("out.wav", &format_create, &data);

    Ok(())
}

fn generate_sine_wave(freq: f32, sample_rate: u32, duration_secs: u32) -> Vec<f32> {
    let len = sample_rate * duration_secs;
    (0..len)
        .flat_map(|i| {
            let t = i as f32 / sample_rate as f32;
            let sample = (2.0 * std::f32::consts::PI * freq * t).sin();
            [sample, sample]
        })
        .collect()
}
