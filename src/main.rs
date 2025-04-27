use anyhow::Result;
use wav::WaveFile;

mod audio;
mod wav;

fn main() -> Result<()> {
    // let format_create = FormatCreate {
    //     file_type: FileType::Wave,
    //     file_format: FileFormat::PCM,
    //     sample_type: SampleType::U16,
    //     chans: 2,
    //     sample_rate: 0x0000AC44,
    // };
    // let data = generate_sine_wave(440.0, 44100, 5);
    // let _ = WaveFile::create("out.wav", &format_create, &data);

    let wave_file = WaveFile::read("a_nocturnal_christmas.wav")?;
    wave_file.write_to_file("out.wav")?;

    Ok(())
}
