use std::error::Error;
use std::fs::File;

use chunk::{read_next_chunk, Data, FileFormat, Format, Riff};

mod chunk;

fn read_wave_file(file: &mut File) -> Result<(), Box<dyn Error>> {
    while let Some(chunk) = read_next_chunk(file)? {
        chunk.print_info();
    }
    Ok(())
}

#[derive(Debug)]
struct WaveFile {
    riff: Riff,
    format: Format,
    data: Data,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut f = File::open("a_nocturnal_christmas.wav")?;

    let _ = read_wave_file(&mut f);

    Ok(())
}
