use anyhow::{anyhow, Context, Result};
use core::str;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

pub struct WaveFile {
    pub format: Format,
    pub data: Vec<f32>,
}

impl WaveFile {
    pub fn read(path: &str) -> Result<WaveFile> {
        let mut file =
            File::open(path).with_context(|| format!("Failed to open file: {}", path))?;

        let riff_chunk = assert_riff_chunk(&mut file)?;
        let mut format_chunk: Option<FormatChunk> = None;
        let mut data_chunk: Option<DataChunk> = None;

        while let Some(chunk) = read_next_chunk(&mut file)? {
            match chunk {
                Chunk::Format(f_chunk) => {
                    format_chunk = Some(f_chunk);
                }
                Chunk::Data(d_chunk) => data_chunk = Some(d_chunk),
                chunk => println!(
                    "skipping chunk {} wiht size {}",
                    chunk.get_name(),
                    chunk.get_size()
                ),
            }
        }

        let format = match format_chunk {
            Some(f_chunk) => Format {
                file_size: riff_chunk.chunk_size,
                file_type: get_file_type(&riff_chunk.t),
                file_format: get_file_format(f_chunk.format),
                sample_type: get_sample_type(f_chunk.sample_size_in_bits),
                chans: f_chunk.chans,
                sample_rate: f_chunk.sample_rate,
                bytes_per_sec: f_chunk.bytes_per_sec,
                frame_size_in_bytes: f_chunk.frame_size_in_bytes,
                sample_size_in_bits: f_chunk.sample_size_in_bits,
            },
            None => {
                return Err(anyhow!("no format chunk found"));
            }
        };

        let data = match data_chunk {
            Some(d_chunk) => match format.sample_type {
                SampleType::U16 => d_chunk
                    .samples
                    .chunks_exact(2)
                    .map(|chunk| {
                        (u16::from_le_bytes([chunk[0], chunk[1]]) as f32 / u16::MAX as f32) * 2.0
                            - 1.0
                    })
                    .collect(),
                SampleType::U32 => d_chunk
                    .samples
                    .chunks_exact(4)
                    .map(|chunk| {
                        (u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as f32
                            / u32::MAX as f32)
                            * 2.0
                            - 1.0
                    })
                    .collect(),
            },
            None => {
                return Err(anyhow!("no data chunk found"));
            }
        };

        Ok(WaveFile { format, data })
    }

    pub fn create(path: &str, format: &FormatCreate, data: &[f32]) -> Result<File> {
        let mut f = File::create(path)?;

        let mut format_buf: [u8; 24] = [0; 24];

        let format_buf_chunk_id = b"fmt ";
        let format_buf_chunk_size: u32 = 16;
        format_buf[..4].copy_from_slice(format_buf_chunk_id);
        format_buf[4..8].copy_from_slice(&format_buf_chunk_size.to_le_bytes());

        // only PCM supported for now
        let format_tag: u16 = 0x0001;
        format_buf[8..10].copy_from_slice(&format_tag.to_le_bytes());

        format_buf[10..12].copy_from_slice(&format.chans.to_le_bytes());
        format_buf[12..16].copy_from_slice(&format.sample_rate.to_le_bytes());

        // only u16 supported for now
        let bits_per_sample = get_sample_size_in_bits(&format.sample_type);
        let frame_size: u16 = format.chans * ((bits_per_sample + 7) / 8);
        let bytes_per_sec: u32 = format.sample_rate * frame_size as u32;

        format_buf[16..20].copy_from_slice(&bytes_per_sec.to_le_bytes());
        format_buf[20..22].copy_from_slice(&frame_size.to_le_bytes());
        format_buf[22..24].copy_from_slice(&bits_per_sample.to_le_bytes());

        let mut data_header_buf: [u8; 8] = [0; 8];
        let data_buf_chunk_id = b"data";
        let data_buf_chunk_size: u32 = (data.len() * 2) as u32;
        data_header_buf[0..4].copy_from_slice(data_buf_chunk_id);
        data_header_buf[4..8].copy_from_slice(&data_buf_chunk_size.to_le_bytes());

        let mut data_body_buf: Vec<u8> = Vec::with_capacity(data_buf_chunk_size as usize);
        for &value in data {
            match format.sample_type {
                SampleType::U16 => {
                    let transformed =
                        ((value.clamp(-1.0, 1.0) + 1.0) * 0.5 * u16::MAX as f32).round() as u16;
                    data_body_buf.extend_from_slice(&transformed.to_le_bytes());
                }
                SampleType::U32 => {
                    let transformed =
                        ((value.clamp(-1.0, 1.0) + 1.0) * 0.5 * u32::MAX as f32).round() as u32;
                    data_body_buf.extend_from_slice(&transformed.to_le_bytes());
                }
            };
        }

        let riff_buf_chunk_id = b"RIFF";
        let riff_buf_chunk_size: u32 = 36;
        let riff_buf_type = b"WAVE";
        let mut riff_buf: [u8; 12] = [0; 12];
        riff_buf[0..4].copy_from_slice(riff_buf_chunk_id);
        riff_buf[4..8].copy_from_slice(&riff_buf_chunk_size.to_le_bytes());
        riff_buf[8..12].copy_from_slice(riff_buf_type);

        let n = f.write(&riff_buf)?;
        assert_eq!(12, n);

        let n = f.write(&format_buf)?;
        assert_eq!(24, n);

        let n = f.write(&data_header_buf)?;
        assert_eq!(8, n);

        let n = f.write(&data_body_buf)?;
        assert_eq!(data_buf_chunk_size as usize, n);

        Ok(f)
    }

    pub fn write_to_file(&self, path: &str) -> Result<File> {
        let format = FormatCreate {
            file_type: self.format.file_type,
            file_format: self.format.file_format,
            sample_type: self.format.sample_type,
            chans: self.format.chans,
            sample_rate: self.format.sample_rate,
        };
        WaveFile::create(path, &format, &self.data)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FileFormat {
    PCM,
}

#[derive(Debug, Clone, Copy)]
pub enum FileType {
    Wave,
}

#[derive(Debug, Clone, Copy)]
pub enum SampleType {
    U16,
    U32,
}

#[derive(Debug)]
pub struct Format {
    file_size: u32,
    file_type: FileType,
    file_format: FileFormat,
    sample_type: SampleType,
    chans: u16,
    sample_rate: u32,
    bytes_per_sec: u32,       // without compression: sample_rate * frame_size
    frame_size_in_bytes: u16, // chans * ((sample_size + 7) / 8)
    sample_size_in_bits: u16,
}

#[derive(Debug)]
pub struct FormatCreate {
    pub file_type: FileType,
    pub file_format: FileFormat,
    pub sample_type: SampleType,
    pub chans: u16,
    pub sample_rate: u32,
}

#[derive(Debug)]
pub enum Chunk {
    Riff(RiffChunk),
    Format(FormatChunk),
    Data(DataChunk),
    Unknown(UnknownChunk),
}

impl Chunk {
    pub fn get_name(&self) -> &str {
        match self {
            Chunk::Riff(riff) => &riff.name,
            Chunk::Format(format) => &format.name,
            Chunk::Data(data) => &data.name,
            Chunk::Unknown(unknown) => &unknown.name,
        }
    }

    pub fn get_size(&self) -> u32 {
        match self {
            Chunk::Riff(riff) => riff.chunk_size,
            Chunk::Format(format) => format.chunk_size,
            Chunk::Data(data) => data.chunk_size,
            Chunk::Unknown(unknown) => unknown.chunk_size,
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

pub fn read_next_chunk(file: &mut File) -> Result<Option<Chunk>> {
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

fn assert_riff_chunk(file: &mut File) -> Result<RiffChunk> {
    if let Some(Chunk::Riff(riff_chunk)) = read_next_chunk(file)? {
        Ok(riff_chunk)
    } else {
        Err(anyhow!("no riff chunk found"))
    }
}

fn read_riff_chunk(file: &mut File, chunk_id: String, chunk_size: u32) -> Result<Chunk> {
    let mut buf = [0; 4];
    let n = file.read(&mut buf[..])?;
    assert_eq!(4, n);

    Ok(Chunk::Riff(RiffChunk {
        name: chunk_id,
        chunk_size,
        t: str::from_utf8(&buf[..])?.to_owned(),
    }))
}

fn read_format_chunk(file: &mut File, chunk_id: String, chunk_size: u32) -> Result<Chunk> {
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

fn read_data_chunk(file: &mut File, chunk_id: String, chunk_size: u32) -> Result<Chunk> {
    let mut buf = vec![0_u8; chunk_size as usize];
    file.read_exact(&mut buf[..])?;

    Ok(Chunk::Data(DataChunk {
        name: chunk_id,
        chunk_size,
        samples: buf,
    }))
}

fn read_unknown_chunk(file: &mut File, chunk_id: String, chunk_size: u32) -> Result<Chunk> {
    file.seek(SeekFrom::Current(chunk_size as i64))?;
    Ok(Chunk::Unknown(UnknownChunk {
        name: chunk_id,
        chunk_size,
    }))
}

fn read_u32_from_buf(buf: &[u8], start: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(buf[start..start + 4].try_into()?))
}

fn read_u16_from_buf(buf: &[u8], start: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(buf[start..start + 2].try_into()?))
}

fn read_chunk_header(file: &mut File) -> Result<Option<(String, u32)>> {
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

fn get_file_type(file_type: &str) -> FileType {
    match file_type {
        "WAVE" => FileType::Wave,
        _ => panic!("unknown file type: {}", file_type),
    }
}

fn get_file_format(file_format: u16) -> FileFormat {
    match file_format {
        1 => FileFormat::PCM,
        _ => panic!("unknown file format: {}", file_format),
    }
}

fn get_sample_type(sample_size_in_bits: u16) -> SampleType {
    match sample_size_in_bits {
        16 => SampleType::U16,
        _ => panic!("can't handle sample size: {}", sample_size_in_bits),
    }
}

fn get_sample_size_in_bits(sample_type: &SampleType) -> u16 {
    match sample_type {
        SampleType::U16 => 16,
        SampleType::U32 => 32,
    }
}
