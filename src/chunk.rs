use core::str;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug)]
pub enum Chunk {
    Riff(Riff),
    Format(Format),
    Data(Data),
    Unknown(Unknown),
}

impl Chunk {
    pub fn print_info(&self) {
        match self {
            Chunk::Riff(riff) => println!("{} -> {}", riff.name, riff.chunk_size),
            Chunk::Format(format) => println!("{} -> {}", format.name, format.chunk_size),
            Chunk::Data(data) => println!("{} -> {}", data.name, data.chunk_size),
            Chunk::Unknown(unknown) => println!("{} -> {}", unknown.name, unknown.chunk_size),
        }
    }
}

#[derive(Debug)]
pub struct Riff {
    name: String,
    chunk_size: u32,
    t: String,
}

#[derive(Debug)]
pub struct Format {
    name: String,
    chunk_size: u32,
    format: FileFormat,
    chans: u16,
    sample_rate: u32,
    bytes_per_sec: u32,       // without compression: sample_rate * frame_size
    frame_size_in_bytes: u16, // chans * ((sample_size + 7) / 8)
    sample_size_in_bits: u16,
}

#[derive(Debug)]
pub struct Data {
    name: String,
    chunk_size: u32,
    samples: Vec<u16>,
}

#[derive(Debug)]
struct Unknown {
    name: String,
    chunk_size: u32,
}

#[derive(Debug)]
pub enum FileFormat {
    PCM,
}

pub fn read_next_chunk(file: &mut File) -> Result<Option<Chunk>, Box<dyn Error>> {
    match read_chunk_header(file)? {
        Some((chunk_id, chunk_size)) => {
            let chunk = match &chunk_id[..] {
                "RIFF" => read_riff_chunk(file, chunk_id, chunk_size),
                "fmt " => read_format_chunk(file, chunk_id, chunk_size),
                "data" => read_data_chunk(file, chunk_id, chunk_size),
                _ => read_unknown_chunk(file, chunk_id, chunk_size),
            }?;
            Ok(Some(chunk))
        }
        None => Ok(None),
    }
}

fn read_riff_chunk(
    file: &mut File,
    chunk_id: String,
    chunk_size: u32,
) -> Result<Chunk, Box<dyn Error>> {
    let mut buf = [0; 4];
    let n = file.read(&mut buf[..])?;
    assert_eq!(4, n);

    Ok(Chunk::Riff(Riff {
        name: chunk_id,
        chunk_size,
        t: str::from_utf8(&buf[..])?.to_owned(),
    }))
}

fn read_format_chunk(
    file: &mut File,
    chunk_id: String,
    chunk_size: u32,
) -> Result<Chunk, Box<dyn Error>> {
    let mut buf = [0; 16];
    let n = file.read(&mut buf[..])?;
    assert_eq!(16, n);

    let w_format_tag = read_u16_from_buf(&buf, 0)?;
    assert_eq!(1, w_format_tag, "Expected pcm format");

    let w_channels = read_u16_from_buf(&buf, 2)?;
    let dw_samples_per_sec = read_u32_from_buf(&buf, 4)?;
    let dw_avg_bytes_per_sec = read_u32_from_buf(&buf, 8)?;
    let w_block_align = read_u16_from_buf(&buf, 12)?;
    let w_bits_per_sample = read_u16_from_buf(&buf, 14)?;

    Ok(Chunk::Format(Format {
        name: chunk_id,
        chunk_size,
        format: FileFormat::PCM,
        chans: w_channels,
        sample_rate: dw_samples_per_sec,
        bytes_per_sec: dw_avg_bytes_per_sec,
        frame_size_in_bytes: w_block_align,
        sample_size_in_bits: w_bits_per_sample,
    }))
}

fn read_data_chunk(
    file: &mut File,
    chunk_id: String,
    chunk_size: u32,
) -> Result<Chunk, Box<dyn Error>> {
    let mut buf = vec![0_u8; chunk_size as usize];
    file.read_exact(&mut buf[..])?;

    // FIXME: Very inefficient...and presupposes a sample_size_in_bits of 16
    let samples = buf
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    Ok(Chunk::Data(Data {
        name: chunk_id,
        chunk_size,
        samples,
    }))
}

fn read_unknown_chunk(
    file: &mut File,
    chunk_id: String,
    chunk_size: u32,
) -> Result<Chunk, Box<dyn Error>> {
    file.seek(SeekFrom::Current(chunk_size as i64))?;
    Ok(Chunk::Unknown(Unknown {
        name: chunk_id,
        chunk_size,
    }))
}

fn read_u32_from_buf(buf: &[u8], start: usize) -> Result<u32, Box<dyn Error>> {
    Ok(u32::from_le_bytes(buf[start..start + 4].try_into()?))
}

fn read_u16_from_buf(buf: &[u8], start: usize) -> Result<u16, Box<dyn Error>> {
    Ok(u16::from_le_bytes(buf[start..start + 2].try_into()?))
}

fn read_chunk_header(file: &mut File) -> Result<Option<(String, u32)>, Box<dyn Error>> {
    let mut buf = [0; 8];
    let n = file.read(&mut buf[..])?;

    // EOF
    if n == 0 {
        return Ok(None);
    }

    assert_eq!(8, n);

    let chunk_id = str::from_utf8(&buf[..4])?.to_owned();
    let chunk_size = read_u32_from_buf(&buf, 4)?;

    Ok(Some((chunk_id, chunk_size)))
}
