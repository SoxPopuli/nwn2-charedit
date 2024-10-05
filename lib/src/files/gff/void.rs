use std::io::Read;

use crate::{
    error::{Error, IntoError},
    files::from_bytes_le,
};

#[derive(PartialEq, Eq)]
pub struct Void {
    pub data: Vec<u8>,
}
impl std::fmt::Debug for Void {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02X?}", &self.data)
    }
}


impl Void {
    pub fn read(mut data: impl Read) -> Result<Self, Error> {
        let size: u32 = from_bytes_le(&mut data)?;

        let mut buf = vec![0u8; size as usize];
        data.read_exact(&mut buf).into_parse_error()?;

        Ok(Self { data: buf })
    }
}
