use crate::{
    error::{Error, IntoError},
    files::{from_bytes_le, Offset},
};
use std::{
    io::{Read, Seek, SeekFrom},
    sync::Arc,
};

fn read_str(mut data: impl Read, strlen: usize) -> Result<Arc<str>, Error> {
    let mut buf = vec![0u8; strlen];

    data.read_exact(&mut buf).into_parse_error()?;

    let x = String::from_utf8_lossy(&buf);
    let ptr: Arc<str> = x.into();

    Ok(ptr)
}

fn read_string(mut data: impl Read, strlen: usize) -> Result<String, Error> {
    let mut buf = vec![0u8; strlen];

    data.read_exact(&mut buf).into_parse_error()?;

    Ok(String::from_utf8_lossy(&buf).to_string())
}

pub fn read(mut data: impl Read + Seek, entries_offset: u64) -> Result<Arc<str>, Error> {
    // let _flags: u32 = from_bytes_le(&mut data)?;
    // let _sound_res_ref: [u8; 16] = read_bytes(&mut data)?;
    // let _volume_variance: u32 = from_bytes_le(&mut data)?;
    // let _pitch_variance: u32 = from_bytes_le(&mut data)?;

    data.seek_relative(28).into_parse_error()?;

    let offset_to_string = Offset(from_bytes_le(&mut data)?);
    let string_size: u32 = from_bytes_le(&mut data)?;

    // let _sound_length: f32 = from_bytes_le(&mut data)?;
    data.seek_relative(4).into_parse_error()?;

    let current_position = data.stream_position().into_parse_error()?;

    let string = if string_size > 0 {
        offset_to_string.seek_with_offet(&mut data, entries_offset)?;
        read_str(&mut data, string_size as usize)?
    } else {
        super::EMPTY_STRING.clone()
    };

    data.seek(SeekFrom::Start(current_position))
        .into_parse_error()?;

    Ok(string)
}
