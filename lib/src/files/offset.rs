#![allow(private_bounds)]

use crate::error::{Error, IntoError};
use std::{ io::{Seek, SeekFrom}, ops::Add };

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(transparent)]
pub struct Offset(pub u32);
impl Offset {
    pub fn seek_to<T>(&self, read: &mut T) -> Result<u64, Error>
    where
        T: Seek,
    {
        read.seek(SeekFrom::Start(self.0 as u64)).into_parse_error()
    }

    pub fn seek_with_offset<S, O>(&self, read: &mut S, offset: O) -> Result<u64, Error>
    where
        S: Seek,
        O: ToOffset,
    {
        let offset = offset.to_offset().0;
        let pos = (self.0 as u64) + (offset as u64);
        read.seek(SeekFrom::Start(pos)).into_parse_error()
    }
}
impl Add<u32> for Offset {
    type Output = Offset;
    fn add(self, rhs: u32) -> Self::Output {
        Offset(self.0 + rhs)
    }
}

pub(crate) trait ToOffset {
    fn to_offset(self) -> Offset;
}
impl ToOffset for Offset {
    fn to_offset(self) -> Offset {
        self
    }
}
impl ToOffset for u32 {
    fn to_offset(self) -> Offset {
        Offset(self)
    }
}
impl ToOffset for i32 {
    fn to_offset(self) -> Offset {
        Offset(self as u32)
    }
}
impl ToOffset for u64 {
    fn to_offset(self) -> Offset {
        Offset(self as u32)
    }
}
impl ToOffset for i64 {
    fn to_offset(self) -> Offset {
        Offset(self as u32)
    }
}
impl ToOffset for usize {
    fn to_offset(self) -> Offset {
        Offset(self as u32)
    }
}
