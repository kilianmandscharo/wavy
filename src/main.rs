use std::error::Error;
use std::fs::File;

use chunk::{read_format, read_frames_u16, SampleType};

mod chunk;

fn main() -> Result<(), Box<dyn Error>> {
    let mut f = File::open("a_nocturnal_christmas.wav")?;

    let format = read_format(&mut f)?;

    match format.sample_type {
        SampleType::U16 => {
            let samples = read_frames_u16(&mut f)?;
            println!("samples length: {}", samples.len());
        }
    }

    Ok(())
}
