use core::str;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug)]
pub enum Chunk {
    Riff(RiffChunk),
    Format(FormatChunk),
    Data(DataChunk),
    Unknown(UnknownChunk),
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

    pub fn get_name(&self) -> &str {
        match self {
            Chunk::Riff(riff) => &riff.name,
            Chunk::Format(format) => &format.name,
            Chunk::Data(data) => &data.name,
            Chunk::Unknown(unknown) => &unknown.name,
        }
    }
}

#[derive(Debug)]
pub struct RiffChunk {
    name: String,
    chunk_size: u32,
    t: String,
}

#[derive(Debug)]
pub struct FormatChunk {
    name: String,
    chunk_size: u32,
    format: u16,
    chans: u16,
    sample_rate: u32,
    bytes_per_sec: u32,       // without compression: sample_rate * frame_size
    frame_size_in_bytes: u16, // chans * ((sample_size + 7) / 8)
    sample_size_in_bits: u16,
}

#[derive(Debug)]
pub struct DataChunk {
    name: String,
    chunk_size: u32,
    samples: Vec<u8>,
}

#[derive(Debug)]
pub struct UnknownChunk {
    name: String,
    chunk_size: u32,
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

    Ok(Chunk::Riff(RiffChunk {
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
    let w_channels = read_u16_from_buf(&buf, 2)?;
    let dw_samples_per_sec = read_u32_from_buf(&buf, 4)?;
    let dw_avg_bytes_per_sec = read_u32_from_buf(&buf, 8)?;
    let w_block_align = read_u16_from_buf(&buf, 12)?;
    let w_bits_per_sample = read_u16_from_buf(&buf, 14)?;

    Ok(Chunk::Format(FormatChunk {
        name: chunk_id,
        chunk_size,
        format: w_format_tag,
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

    Ok(Chunk::Data(DataChunk {
        name: chunk_id,
        chunk_size,
        samples: buf,
    }))
}

fn read_unknown_chunk(
    file: &mut File,
    chunk_id: String,
    chunk_size: u32,
) -> Result<Chunk, Box<dyn Error>> {
    file.seek(SeekFrom::Current(chunk_size as i64))?;
    Ok(Chunk::Unknown(UnknownChunk {
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

#[derive(Debug)]
pub enum FileType {
    Wave,
}

fn get_file_type(file_type: &str) -> FileType {
    match file_type {
        "WAVE" => FileType::Wave,
        _ => panic!("unknown file type: {}", file_type),
    }
}

#[derive(Debug)]
pub enum FileFormat {
    PCM,
}

fn get_file_format(file_format: u16) -> FileFormat {
    match file_format {
        1 => FileFormat::PCM,
        _ => panic!("unknown file format: {}", file_format),
    }
}

#[derive(Debug)]
pub enum SampleType {
    U16,
}

fn get_sample_type(sample_size_in_bits: u16) -> SampleType {
    match sample_size_in_bits {
        16 => SampleType::U16,
        _ => panic!("can't handle sample size: {}", sample_size_in_bits),
    }
}

#[derive(Debug)]
pub struct Format {
    file_size: u32,
    file_type: FileType,
    file_format: FileFormat,
    pub sample_type: SampleType,
    chans: u16,
    sample_rate: u32,
    bytes_per_sec: u32,       // without compression: sample_rate * frame_size
    frame_size_in_bytes: u16, // chans * ((sample_size + 7) / 8)
    sample_size_in_bits: u16,
}

pub fn read_format(file: &mut File) -> Result<Format, Box<dyn Error>> {
    if let Some(Chunk::Riff(riff_chunk)) = read_next_chunk(file)? {
        let mut format_chunk: Option<FormatChunk> = None;
        while let Some(chunk) = read_next_chunk(file)? {
            match chunk {
                Chunk::Format(fmt_chunk) => {
                    format_chunk = Some(fmt_chunk);
                    break;
                }
                chunk => println!("skipping chunk: {}", chunk.get_name()),
            }
        }
        match format_chunk {
            Some(fmt_chunk) => Ok(Format {
                file_size: riff_chunk.chunk_size,
                file_type: get_file_type(&riff_chunk.t),
                file_format: get_file_format(fmt_chunk.format),
                sample_type: get_sample_type(fmt_chunk.sample_size_in_bits),
                chans: fmt_chunk.chans,
                sample_rate: fmt_chunk.sample_rate,
                bytes_per_sec: fmt_chunk.bytes_per_sec,
                frame_size_in_bytes: fmt_chunk.frame_size_in_bytes,
                sample_size_in_bits: fmt_chunk.sample_size_in_bits,
            }),
            None => panic!("no format chunk found"),
        }
    } else {
        panic!("no riff chunk found")
    }
}

pub fn read_frames_u16(file: &mut File) -> Result<Vec<u16>, Box<dyn Error>> {
    file.seek(SeekFrom::Start(0))?;

    let mut data_chunk: Option<DataChunk> = None;
    while let Some(chunk) = read_next_chunk(file)? {
        match chunk {
            Chunk::Data(d_chunk) => {
                data_chunk = Some(d_chunk);
                break;
            }
            chunk => println!("skipping chunk: {}", chunk.get_name()),
        }
    }

    match data_chunk {
        Some(chunk) => {
            // FIXME: Is there a better way?
            let samples = chunk
                .samples
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect();
            Ok(samples)
        }
        None => panic!("no data chunk found"),
    }
}
