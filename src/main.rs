use core::str;
use std::error::Error;
use std::fs::File;
use std::io::Read;

enum Format {
    PCM,
}

struct FormatInfo {
    format: Format,
    chans: u16,
    sample_rate: u32,
    bytes_per_sec: u32, // without compression: sample_rate * frame_size
    frame_size: u16,    // chans * ((sample_size + 7) / 8)
    sample_size: u16,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut f = File::open("a_nocturnal_christmas.wav")?;

    let mut buf = [0; 12];
    let n = f.read(&mut buf[..])?;
    assert_eq!(12, n);

    let chunk_id = str::from_utf8(&buf[..4])?;
    assert_eq!("RIFF", chunk_id);

    let chunk_size = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
    println!("File size in bytes: {chunk_size}");

    let riff_type = str::from_utf8(&buf[8..])?;
    assert_eq!(
        "WAVE", riff_type,
        "format {} cannot be handled as of right now",
        riff_type
    );
    println!("File format: {riff_type}");

    let mut buf = [0; 24];
    let n = f.read(&mut buf[..])?;
    assert_eq!(24, n);

    let chunk_id = str::from_utf8(&buf[..4])?;
    assert_eq!("fmt ", chunk_id);

    let chunk_size = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
    println!("Chunk size in bytes: {chunk_size}");

    let w_format_tag = u16::from_le_bytes([buf[8], buf[9]]);
    assert_eq!(1, w_format_tag, "Expected pcm format");
    println!("wFormatTag: {w_format_tag}");

    let w_channels = u16::from_le_bytes([buf[10], buf[11]]);
    println!("wChannels: {w_channels}");

    let dw_samples_per_sec = u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);
    println!("dwSamplesPerSec: {dw_samples_per_sec}");

    let dw_avg_bytes_per_sec = u32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]);
    println!("dwAvgBytesPerSec: {dw_avg_bytes_per_sec}");

    let w_block_align = u16::from_le_bytes([buf[20], buf[21]]);
    println!("wBlockAlign: {w_block_align}");

    let w_bits_per_sample = u16::from_le_bytes([buf[22], buf[23]]);
    println!("wBitsPerSample: {w_bits_per_sample}");

    let format_info = FormatInfo {
        format: Format::PCM,
        chans: w_channels,
        sample_rate: dw_samples_per_sec,
        bytes_per_sec: dw_avg_bytes_per_sec,
        frame_size: w_block_align,
        sample_size: w_bits_per_sample,
    };

    Ok(())
}
