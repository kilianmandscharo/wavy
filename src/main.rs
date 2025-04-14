use std::error::Error;
use std::fs::File;

use chunk::read_format;

mod chunk;

fn main() -> Result<(), Box<dyn Error>> {
    let mut f = File::open("a_nocturnal_christmas.wav")?;

    let format = read_format(&mut f)?;

    println!("{:?}", format);

    Ok(())
}
