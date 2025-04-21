use rand::Rng;
use std::error::Error;
use std::fs::File;

use chunk::{
    create_file_u16, open_file_read, read_frames_u16, FileFormat, FileType, FormatCreate,
    SampleType,
};

mod chunk;

fn main() -> Result<(), Box<dyn Error>> {
    let mut f = File::open("a_nocturnal_christmas.wav")?;

    let format = open_file_read(&mut f)?;

    match format.sample_type {
        SampleType::U16 => {
            let samples = read_frames_u16(&mut f)?;
            println!("samples length: {}", samples.len());
        }
    }

    let format_create = FormatCreate {
        file_type: FileType::Wave,
        file_format: FileFormat::PCM,
        sample_type: SampleType::U16,
        chans: 2,
        sample_rate: 0x0000AC44,
    };
    let data = generate_white_noise_u16(44100 * 5);
    let _ = create_file_u16("out.wav", &format_create, &data);

    Ok(())
}

fn generate_white_noise_u16(len: usize) -> Vec<u16> {
    let mut rng = rand::rng();
    (0..len).map(|_| rng.random::<u16>()).collect()
}
