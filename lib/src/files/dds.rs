#![allow(clippy::needless_range_loop)]

use rust_utils::bit_reader::BitReader;

use crate::error::{Error, IntoError};
use std::io::Read;

// DDS Format: https://learn.microsoft.com/en-us/windows/win32/direct3ddds/dx-graphics-dds-pguide
// BC7 Format: https://learn.microsoft.com/en-us/windows/win32/direct3d11/bc7-format
// BC7 Format Mode Reference: https://learn.microsoft.com/en-us/windows/win32/direct3d11/bc7-format-mode-reference

mod constants {
    pub const WEIGHTS_2: [u8; 4] = [0, 21, 43, 64];
    pub const WEIGHTS_3: [u8; 8] = [0, 9, 18, 27, 37, 46, 55, 64];
    pub const WEIGHTS_4: [u8; 16] = [0, 4, 9, 13, 17, 21, 26, 30, 34, 38, 43, 47, 51, 55, 60, 64];

    #[rustfmt::skip]
    pub const PARTITION_2: [u8; 64 * 16] = [
        0,0,1,1,0,0,1,1,0,0,1,1,0,0,1,1,		0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1,		0,1,1,1,0,1,1,1,0,1,1,1,0,1,1,1,		0,0,0,1,0,0,1,1,0,0,1,1,0,1,1,1,		0,0,0,0,0,0,0,1,0,0,0,1,0,0,1,1,		0,0,1,1,0,1,1,1,0,1,1,1,1,1,1,1,		0,0,0,1,0,0,1,1,0,1,1,1,1,1,1,1,		0,0,0,0,0,0,0,1,0,0,1,1,0,1,1,1,
        0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,1,		0,0,1,1,0,1,1,1,1,1,1,1,1,1,1,1,		0,0,0,0,0,0,0,1,0,1,1,1,1,1,1,1,		0,0,0,0,0,0,0,0,0,0,0,1,0,1,1,1,		0,0,0,1,0,1,1,1,1,1,1,1,1,1,1,1,		0,0,0,0,0,0,0,0,1,1,1,1,1,1,1,1,		0,0,0,0,1,1,1,1,1,1,1,1,1,1,1,1,		0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,1,
        0,0,0,0,1,0,0,0,1,1,1,0,1,1,1,1,		0,1,1,1,0,0,0,1,0,0,0,0,0,0,0,0,		0,0,0,0,0,0,0,0,1,0,0,0,1,1,1,0,		0,1,1,1,0,0,1,1,0,0,0,1,0,0,0,0,		0,0,1,1,0,0,0,1,0,0,0,0,0,0,0,0,		0,0,0,0,1,0,0,0,1,1,0,0,1,1,1,0,		0,0,0,0,0,0,0,0,1,0,0,0,1,1,0,0,		0,1,1,1,0,0,1,1,0,0,1,1,0,0,0,1,
        0,0,1,1,0,0,0,1,0,0,0,1,0,0,0,0,		0,0,0,0,1,0,0,0,1,0,0,0,1,1,0,0,		0,1,1,0,0,1,1,0,0,1,1,0,0,1,1,0,		0,0,1,1,0,1,1,0,0,1,1,0,1,1,0,0,		0,0,0,1,0,1,1,1,1,1,1,0,1,0,0,0,		0,0,0,0,1,1,1,1,1,1,1,1,0,0,0,0,		0,1,1,1,0,0,0,1,1,0,0,0,1,1,1,0,		0,0,1,1,1,0,0,1,1,0,0,1,1,1,0,0,
        0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1,		0,0,0,0,1,1,1,1,0,0,0,0,1,1,1,1,		0,1,0,1,1,0,1,0,0,1,0,1,1,0,1,0,		0,0,1,1,0,0,1,1,1,1,0,0,1,1,0,0,		0,0,1,1,1,1,0,0,0,0,1,1,1,1,0,0,		0,1,0,1,0,1,0,1,1,0,1,0,1,0,1,0,		0,1,1,0,1,0,0,1,0,1,1,0,1,0,0,1,		0,1,0,1,1,0,1,0,1,0,1,0,0,1,0,1,
        0,1,1,1,0,0,1,1,1,1,0,0,1,1,1,0,		0,0,0,1,0,0,1,1,1,1,0,0,1,0,0,0,		0,0,1,1,0,0,1,0,0,1,0,0,1,1,0,0,		0,0,1,1,1,0,1,1,1,1,0,1,1,1,0,0,		0,1,1,0,1,0,0,1,1,0,0,1,0,1,1,0,		0,0,1,1,1,1,0,0,1,1,0,0,0,0,1,1,		0,1,1,0,0,1,1,0,1,0,0,1,1,0,0,1,		0,0,0,0,0,1,1,0,0,1,1,0,0,0,0,0,
        0,1,0,0,1,1,1,0,0,1,0,0,0,0,0,0,		0,0,1,0,0,1,1,1,0,0,1,0,0,0,0,0,		0,0,0,0,0,0,1,0,0,1,1,1,0,0,1,0,		0,0,0,0,0,1,0,0,1,1,1,0,0,1,0,0,		0,1,1,0,1,1,0,0,1,0,0,1,0,0,1,1,		0,0,1,1,0,1,1,0,1,1,0,0,1,0,0,1,		0,1,1,0,0,0,1,1,1,0,0,1,1,1,0,0,		0,0,1,1,1,0,0,1,1,1,0,0,0,1,1,0,
        0,1,1,0,1,1,0,0,1,1,0,0,1,0,0,1,		0,1,1,0,0,0,1,1,0,0,1,1,1,0,0,1,		0,1,1,1,1,1,1,0,1,0,0,0,0,0,0,1,		0,0,0,1,1,0,0,0,1,1,1,0,0,1,1,1,		0,0,0,0,1,1,1,1,0,0,1,1,0,0,1,1,		0,0,1,1,0,0,1,1,1,1,1,1,0,0,0,0,		0,0,1,0,0,0,1,0,1,1,1,0,1,1,1,0,		0,1,0,0,0,1,0,0,0,1,1,1,0,1,1,1
    ];

    #[rustfmt::skip]
    pub const PARTITION_3: [u8; 64 * 16] = [
        0,0,1,1,0,0,1,1,0,2,2,1,2,2,2,2,		0,0,0,1,0,0,1,1,2,2,1,1,2,2,2,1,		0,0,0,0,2,0,0,1,2,2,1,1,2,2,1,1,		0,2,2,2,0,0,2,2,0,0,1,1,0,1,1,1,		0,0,0,0,0,0,0,0,1,1,2,2,1,1,2,2,		0,0,1,1,0,0,1,1,0,0,2,2,0,0,2,2,		0,0,2,2,0,0,2,2,1,1,1,1,1,1,1,1,		0,0,1,1,0,0,1,1,2,2,1,1,2,2,1,1,
        0,0,0,0,0,0,0,0,1,1,1,1,2,2,2,2,		0,0,0,0,1,1,1,1,1,1,1,1,2,2,2,2,		0,0,0,0,1,1,1,1,2,2,2,2,2,2,2,2,		0,0,1,2,0,0,1,2,0,0,1,2,0,0,1,2,		0,1,1,2,0,1,1,2,0,1,1,2,0,1,1,2,		0,1,2,2,0,1,2,2,0,1,2,2,0,1,2,2,		0,0,1,1,0,1,1,2,1,1,2,2,1,2,2,2,		0,0,1,1,2,0,0,1,2,2,0,0,2,2,2,0,
        0,0,0,1,0,0,1,1,0,1,1,2,1,1,2,2,		0,1,1,1,0,0,1,1,2,0,0,1,2,2,0,0,		0,0,0,0,1,1,2,2,1,1,2,2,1,1,2,2,		0,0,2,2,0,0,2,2,0,0,2,2,1,1,1,1,		0,1,1,1,0,1,1,1,0,2,2,2,0,2,2,2,		0,0,0,1,0,0,0,1,2,2,2,1,2,2,2,1,		0,0,0,0,0,0,1,1,0,1,2,2,0,1,2,2,		0,0,0,0,1,1,0,0,2,2,1,0,2,2,1,0,
        0,1,2,2,0,1,2,2,0,0,1,1,0,0,0,0,		0,0,1,2,0,0,1,2,1,1,2,2,2,2,2,2,		0,1,1,0,1,2,2,1,1,2,2,1,0,1,1,0,		0,0,0,0,0,1,1,0,1,2,2,1,1,2,2,1,		0,0,2,2,1,1,0,2,1,1,0,2,0,0,2,2,		0,1,1,0,0,1,1,0,2,0,0,2,2,2,2,2,		0,0,1,1,0,1,2,2,0,1,2,2,0,0,1,1,		0,0,0,0,2,0,0,0,2,2,1,1,2,2,2,1,
        0,0,0,0,0,0,0,2,1,1,2,2,1,2,2,2,		0,2,2,2,0,0,2,2,0,0,1,2,0,0,1,1,		0,0,1,1,0,0,1,2,0,0,2,2,0,2,2,2,		0,1,2,0,0,1,2,0,0,1,2,0,0,1,2,0,		0,0,0,0,1,1,1,1,2,2,2,2,0,0,0,0,		0,1,2,0,1,2,0,1,2,0,1,2,0,1,2,0,		0,1,2,0,2,0,1,2,1,2,0,1,0,1,2,0,		0,0,1,1,2,2,0,0,1,1,2,2,0,0,1,1,
        0,0,1,1,1,1,2,2,2,2,0,0,0,0,1,1,		0,1,0,1,0,1,0,1,2,2,2,2,2,2,2,2,		0,0,0,0,0,0,0,0,2,1,2,1,2,1,2,1,		0,0,2,2,1,1,2,2,0,0,2,2,1,1,2,2,		0,0,2,2,0,0,1,1,0,0,2,2,0,0,1,1,		0,2,2,0,1,2,2,1,0,2,2,0,1,2,2,1,		0,1,0,1,2,2,2,2,2,2,2,2,0,1,0,1,		0,0,0,0,2,1,2,1,2,1,2,1,2,1,2,1,
        0,1,0,1,0,1,0,1,0,1,0,1,2,2,2,2,		0,2,2,2,0,1,1,1,0,2,2,2,0,1,1,1,		0,0,0,2,1,1,1,2,0,0,0,2,1,1,1,2,		0,0,0,0,2,1,1,2,2,1,1,2,2,1,1,2,		0,2,2,2,0,1,1,1,0,1,1,1,0,2,2,2,		0,0,0,2,1,1,1,2,1,1,1,2,0,0,0,2,		0,1,1,0,0,1,1,0,0,1,1,0,2,2,2,2,		0,0,0,0,0,0,0,0,2,1,1,2,2,1,1,2,
        0,1,1,0,0,1,1,0,2,2,2,2,2,2,2,2,		0,0,2,2,0,0,1,1,0,0,1,1,0,0,2,2,		0,0,2,2,1,1,2,2,1,1,2,2,0,0,2,2,		0,0,0,0,0,0,0,0,0,0,0,0,2,1,1,2,		0,0,0,2,0,0,0,1,0,0,0,2,0,0,0,1,		0,2,2,2,1,2,2,2,0,2,2,2,1,2,2,2,		0,1,0,1,2,2,2,2,2,2,2,2,2,2,2,2,		0,1,1,1,2,0,1,1,2,2,0,1,2,2,2,0,
    ];

    pub const TABLE_ANCHOR_INDEX_SECOND_SUBSET: [u8; 64] = [
        15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 2, 8, 2, 2, 8, 8, 15,
        2, 8, 2, 2, 8, 8, 2, 2, 15, 15, 6, 8, 2, 8, 15, 15, 2, 8, 2, 2, 2, 15, 15, 6, 6, 2, 6, 8,
        15, 15, 2, 2, 15, 15, 15, 15, 15, 2, 2, 15,
    ];

    pub const TABLE_ANCHOR_INDEX_THIRD_SUBSET_1: [u8; 64] = [
        3, 3, 15, 15, 8, 3, 15, 15, 8, 8, 6, 6, 6, 5, 3, 3, 3, 3, 8, 15, 3, 3, 6, 10, 5, 8, 8, 6,
        8, 5, 15, 15, 8, 15, 3, 5, 6, 10, 8, 15, 15, 3, 15, 5, 15, 15, 15, 15, 3, 15, 5, 5, 5, 8,
        5, 10, 5, 10, 8, 13, 15, 12, 3, 3,
    ];

    pub const TABLE_ANCHOR_INDEX_THIRD_SUBSET_2: [u8; 64] = [
        15, 8, 8, 3, 15, 15, 3, 8, 15, 15, 15, 15, 15, 15, 15, 8, 15, 8, 15, 3, 15, 8, 15, 8, 3,
        15, 6, 10, 15, 15, 10, 8, 15, 3, 15, 10, 10, 8, 9, 10, 6, 15, 8, 15, 3, 6, 6, 8, 15, 3, 15,
        15, 15, 15, 15, 15, 15, 15, 15, 15, 3, 15, 15, 8,
    ];
}

fn bc7_dequant_p(mut val: u8, p_bit: u8, val_bits: u8) -> u8 {
    assert!(val < (1 << val_bits));
    assert!(p_bit < 2);
    assert!((4..=8).contains(&val_bits));
    let total_bits = val_bits + 1;
    val = (val << 1) | p_bit;
    val <<= 8 - total_bits;
    val |= val >> total_bits;

    val
}

fn bc7_dequant(mut val: u8, val_bits: u8) -> u8 {
    assert!(val < (1 << val_bits));
    assert!((4..=8).contains(&val_bits));
    val <<= 8 - val_bits;
    val |= val >> val_bits;
    val
}

fn bc7_interp2(l: u8, h: u8, w: u8) -> u8 {
    assert!(w < 4);
    let w = w as usize;
    (l * (64 - constants::WEIGHTS_2[w]) + h * constants::WEIGHTS_2[w] + 32) >> 6
}

fn bc7_interp3(l: u8, h: u8, w: u8) -> u8 {
    assert!(w < 8);
    let w = w as usize;
    (l * (64 - constants::WEIGHTS_3[w]) + h * constants::WEIGHTS_3[w] + 32) >> 6
}

fn bc7_interp4(l: u8, h: u8, w: u8) -> u8 {
    assert!(w < 16);
    let w = w as usize;
    (l * (64 - constants::WEIGHTS_4[w]) + h * constants::WEIGHTS_4[w] + 32) >> 6
}

fn bc7_interp(l: u8, h: u8, w: u8, bits: u8) -> u8 {
    match bits {
        2 => bc7_interp2(l, h, w),
        3 => bc7_interp3(l, h, w),
        4 => bc7_interp4(l, h, w),

        _ => 0,
    }
}

macro_rules! read {
    ($reader: expr) => {{
        use rust_utils::byte_readers::*;

        $reader.read_le().into_parse_error()
    }};
}

common::open_enum! {
  pub enum PixelFormatFlags: u32 {
    AlphaPixels = 0x1,
    Alpha  = 0x2,
    FourCC  = 0x4,
    Rgb  = 0x40,
    Yuv  = 0x200,
    Luminance  = 0x20000,
  }
}
impl PixelFormatFlags {
    fn has_flag(&self, flag: PixelFormatFlags) -> bool {
        self.0 & flag.0 == flag.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DdsPixelFormat {
    size: u32,
    flags: PixelFormatFlags,
    four_cc: FourCC,
    rgb_bit_count: u32,
    r_bit_mask: u32,
    g_bit_mask: u32,
    b_bit_mask: u32,
    a_bit_mask: u32,
}
impl DdsPixelFormat {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            size: read!(reader)?,
            flags: PixelFormatFlags(read!(reader)?),
            four_cc: FourCC(read!(reader)?),
            rgb_bit_count: read!(reader)?,
            r_bit_mask: read!(reader)?,
            g_bit_mask: read!(reader)?,
            b_bit_mask: read!(reader)?,
            a_bit_mask: read!(reader)?,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Header {
    size: u32,
    flags: u32,
    height: u32,
    width: u32,
    pitch_or_linear_size: u32,
    depth: u32,
    mip_map_count: u32,
    reserved1: [u32; 11],
    pixel_format: DdsPixelFormat,
    caps: u32,
    caps2: u32,
    caps3: u32,
    caps4: u32,
    reserved2: u32,
}
impl Header {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        fn reserved1(reader: &mut impl Read) -> Result<[u32; 11], Error> {
            let mut buf = [0u32; 11];

            for (i, x) in (0..11).map(|_| read!(reader)).enumerate() {
                match x {
                    Ok(x) => {
                        buf[i] = x;
                    }
                    Err(e) => return Err(e),
                }
            }

            Ok(buf)
        };

        Ok(Self {
            size: read!(reader)?,
            flags: read!(reader)?,
            height: read!(reader)?,
            width: read!(reader)?,
            pitch_or_linear_size: read!(reader)?,
            depth: read!(reader)?,
            mip_map_count: read!(reader)?,
            reserved1: reserved1(reader)?,
            pixel_format: DdsPixelFormat::read(reader)?,
            caps: read!(reader)?,
            caps2: read!(reader)?,
            caps3: read!(reader)?,
            caps4: read!(reader)?,
            reserved2: read!(reader)?,
        })
    }
}

common::int_enum! {
    pub enum ResourceDimension: u32 {
        Unknown = 0,
        Buffer = 1,
        Texture1D = 2,
        Texture2D = 3,
        Texture3D = 4,
    }
}

common::open_enum! {
  pub enum DXGIFormat: u32 {
    Unknown = 0,
    R32G32B32A32_TYPELESS = 1,
    R32G32B32A32_FLOAT = 2,
    R32G32B32A32_UINT = 3,
    R32G32B32A32_SINT = 4,
    R32G32B32_TYPELESS = 5,
    R32G32B32_FLOAT = 6,
    R32G32B32_UINT = 7,
    R32G32B32_SINT = 8,
    R16G16B16A16_TYPELESS = 9,
    R16G16B16A16_FLOAT = 10,
    R16G16B16A16_UNORM = 11,
    R16G16B16A16_UINT = 12,
    R16G16B16A16_SNORM = 13,
    R16G16B16A16_SINT = 14,
    R32G32_TYPELESS = 15,
    R32G32_FLOAT = 16,
    R32G32_UINT = 17,
    R32G32_SINT = 18,
    R32G8X24_TYPELESS = 19,
    D32_FLOAT_S8X24_UINT = 20,
    R32_FLOAT_X8X24_TYPELESS = 21,
    X32_TYPELESS_G8X24_UINT = 22,
    R10G10B10A2_TYPELESS = 23,
    R10G10B10A2_UNORM = 24,
    R10G10B10A2_UINT = 25,
    R11G11B10_FLOAT = 26,
    R8G8B8A8_TYPELESS = 27,
    R8G8B8A8_UNORM = 28,
    R8G8B8A8_UNORM_SRGB = 29,
    R8G8B8A8_UINT = 30,
    R8G8B8A8_SNORM = 31,
    R8G8B8A8_SINT = 32,
    R16G16_TYPELESS = 33,
    R16G16_FLOAT = 34,
    R16G16_UNORM = 35,
    R16G16_UINT = 36,
    R16G16_SNORM = 37,
    R16G16_SINT = 38,
    R32_TYPELESS = 39,
    D32_FLOAT = 40,
    R32_FLOAT = 41,
    R32_UINT = 42,
    R32_SINT = 43,
    R24G8_TYPELESS = 44,
    D24_UNORM_S8_UINT = 45,
    R24_UNORM_X8_TYPELESS = 46,
    X24_TYPELESS_G8_UINT = 47,
    R8G8_TYPELESS = 48,
    R8G8_UNORM = 49,
    R8G8_UINT = 50,
    R8G8_SNORM = 51,
    R8G8_SINT = 52,
    R16_TYPELESS = 53,
    R16_FLOAT = 54,
    D16_UNORM = 55,
    R16_UNORM = 56,
    R16_UINT = 57,
    R16_SNORM = 58,
    R16_SINT = 59,
    R8_TYPELESS = 60,
    R8_UNORM = 61,
    R8_UINT = 62,
    R8_SNORM = 63,
    R8_SINT = 64,
    A8_UNORM = 65,
    R1_UNORM = 66,
    R9G9B9E5_SHAREDEXP = 67,
    R8G8_B8G8_UNORM = 68,
    G8R8_G8B8_UNORM = 69,
    BC1_TYPELESS = 70,
    BC1_UNORM = 71,
    BC1_UNORM_SRGB = 72,
    BC2_TYPELESS = 73,
    BC2_UNORM = 74,
    BC2_UNORM_SRGB = 75,
    BC3_TYPELESS = 76,
    BC3_UNORM = 77,
    BC3_UNORM_SRGB = 78,
    BC4_TYPELESS = 79,
    BC4_UNORM = 80,
    BC4_SNORM = 81,
    BC5_TYPELESS = 82,
    BC5_UNORM = 83,
    BC5_SNORM = 84,
    B5G6R5_UNORM = 85,
    B5G5R5A1_UNORM = 86,
    B8G8R8A8_UNORM = 87,
    B8G8R8X8_UNORM = 88,
    R10G10B10_XR_BIAS_A2_UNORM = 89,
    B8G8R8A8_TYPELESS = 90,
    B8G8R8A8_UNORM_SRGB = 91,
    B8G8R8X8_TYPELESS = 92,
    B8G8R8X8_UNORM_SRGB = 93,
    BC6H_TYPELESS = 94,
    BC6H_UF16 = 95,
    BC6H_SF16 = 96,
    BC7_TYPELESS = 97,
    BC7_UNORM = 98,
    BC7_UNORM_SRGB = 99,
    AYUV = 100,
    Y410 = 101,
    Y416 = 102,
    NV12 = 103,
    P010 = 104,
    P016 = 105,
    OPAQUE_420 = 106,
    YUY2 = 107,
    Y210 = 108,
    Y216 = 109,
    NV11 = 110,
    AI44 = 111,
    IA44 = 112,
    P8 = 113,
    A8P8 = 114,
    B4G4R4A4_UNORM = 115,
    P208 = 130,
    V208 = 131,
    V408 = 132,
    SAMPLER_FEEDBACK_MIN_MIP_OPAQUE = 189,
    SAMPLER_FEEDBACK_MIP_REGION_USED_OPAQUE = 190,
    FORCE_UINT = 0xffffffff
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HeaderExtra {
    dxgi_format: DXGIFormat,
    resource_dimension: ResourceDimension,
    misc_flag: u32,
    array_size: u32,
    misc_flags2: u32,
}
impl HeaderExtra {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            dxgi_format: DXGIFormat(read!(reader)?),
            resource_dimension: {
                let x: u32 = read!(reader)?;
                x.try_into()?
            },
            misc_flag: read!(reader)?,
            array_size: read!(reader)?,
            misc_flags2: read!(reader)?,
        })
    }
}

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct FourCC(pub [u8; 4]);
impl std::fmt::Debug for FourCC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        let chars = self
            .0
            .map(|x| unsafe { char::from_u32_unchecked(x as u32) });

        if chars
            .iter()
            .all(|x| x.is_alphanumeric() || x.is_ascii_whitespace() || x.is_ascii_punctuation())
        {
            write!(f, "\"")?;

            for c in chars {
                f.write_char(c)?;
            }

            write!(f, "\"")
        } else {
            self.0.fmt(f)
        }
    }
}
impl PartialEq<[u8; 4]> for FourCC {
    fn eq(&self, other: &[u8; 4]) -> bool {
        self.0.eq(other)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Dds {
    pub four_cc: FourCC,
    pub header: Header,
    pub header_extra: Option<HeaderExtra>,
}
impl Dds {
    pub fn read<R>(mut reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        let four_cc = FourCC(read!(reader)?);
        let header = Header::read(&mut reader)?;

        let header_extra = if header.pixel_format.flags.has_flag(PixelFormatFlags::FourCC)
            && header.pixel_format.four_cc.eq(b"DX10")
        {
            Some(HeaderExtra::read(&mut reader)?)
        } else {
            None
        };

        let data = {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).map(|_| buf)
        }
        .into_parse_error()?;

        let mut reader = BitReader::new(data);
        let mut pixels = Vec::with_capacity(header.height as usize * header.width as usize);

        while reader.bits_remaining() > 0 {
            let _ = read_block(&mut reader, &mut pixels);
        }

        Ok(Self {
            four_cc,
            header,
            header_extra,
        })
    }
}

fn to_error() -> Error {
    Error::ParseError("DDS: Unexpected EOF".to_string())
}

fn get_bit(reader: &mut BitReader) -> Result<u8, Error> {
    reader.get_bit().ok_or_else(to_error)
}

fn get_bits_u8(reader: &mut BitReader, bits: u8) -> Result<u8, Error> {
    reader.get_bits_u8(bits).ok_or_else(to_error)
}

fn get_bits_u32(reader: &mut BitReader, bits: u8) -> Result<u32, Error> {
    reader.get_bits_u32(bits).ok_or_else(to_error)
}

fn get_bits_u64(reader: &mut BitReader, bits: u8) -> Result<u64, Error> {
    reader.get_bits_u64(bits).ok_or_else(to_error)
}

fn read_block(reader: &mut BitReader, pixels: &mut Vec<Rgba>) -> Result<(), Error> {
    let mode = get_mode(reader);

    match mode {
        0 | 2 => unpack_mode_0_2(mode, reader, pixels)?,
        1 | 3 | 7 => unpack_mode_1_3_7(mode, reader, pixels)?,
        4 | 5 => unpack_mode_4_5(mode, reader, pixels)?,
        6 => panic!(),
        x => panic!("Unexpected mode: {x}"),
    }

    Ok(())
}

fn make_array<T: Sized, const N: usize>(
    mut f: impl FnMut() -> Result<T, Error>,
) -> Result<[T; N], Error> {
    let mut output = [const { std::mem::MaybeUninit::uninit() }; N];

    for i in 0..N {
        let x = f()?;
        output[i].write(x);
    }

    Ok(unsafe { std::mem::transmute_copy(&output) })
}

fn unpack_mode_0_2(mode: u8, reader: &mut BitReader, pixels: &mut Vec<Rgba>) -> Result<(), Error> {
    const ENDPOINTS: usize = 6;
    const COMPS: usize = 3;
    let weight_bits = if mode == 0 { 3 } else { 2 };
    let endpoint_bits = if mode == 0 { 4 } else { 5 };
    let p_bits = if mode == 0 { 6 } else { 0 };
    let weight_vals = 1 << weight_bits;

    let part = get_bits_u8(reader, if mode == 0 { 4 } else { 6 })? as usize;

    let mut endpoints = [const { Rgba::zero() }; ENDPOINTS];

    for c in 0..COMPS {
        for e in 0..ENDPOINTS {
            endpoints[e][c] = get_bits_u8(reader, endpoint_bits)?;
        }
    }

    let mut p_bit_array = [0; 6];
    for i in 0..p_bits {
        p_bit_array[i] = get_bit(reader)?;
    }

    let mut weights = [0; 16];
    for i in 0..16 {
        let size = if i == 0
            || i == constants::TABLE_ANCHOR_INDEX_THIRD_SUBSET_1[part]
            || i == constants::TABLE_ANCHOR_INDEX_THIRD_SUBSET_2[part]
        {
            weight_bits - 1
        } else {
            weight_bits
        };

        weights[i as usize] = get_bits_u8(reader, size)?;
    }

    for e in 0..ENDPOINTS {
        for c in 0..4 {
            let x = if c == 3 {
                255
            } else if p_bits != 0 {
                bc7_dequant_p(endpoints[e][c], p_bit_array[e], endpoint_bits)
            } else {
                bc7_dequant(endpoints[e][c], endpoint_bits)
            };
            endpoints[e][c] = x
        }
    }

    let mut block_colors: [[Rgba; 3]; 8] = std::array::from_fn(|_| [const { Rgba::zero() }; 3]);
    for s in 0..3 {
        for i in 0..weight_vals {
            for c in 0..3 {
                block_colors[s][i][c] = bc7_interp(
                    endpoints[s * 2][c],
                    endpoints[s * 2 + 1][c],
                    i as u8,
                    weight_bits,
                );
                block_colors[s][i][3] = 255;
            }
        }
    }

    for i in 0..16 {
        let part = constants::PARTITION_3[part * 16 + i] as usize;
        let weight = weights[i] as usize;
        let p = block_colors[part][weight].clone();

        pixels.push(p);
    }

    Ok(())
}

fn unpack_mode_1_3_7(
    mode: u8,
    reader: &mut BitReader,
    pixels: &mut Vec<Rgba>,
) -> Result<(), Error> {
    const ENDPOINTS: usize = 4;
    let comps = if mode == 7 { 4 } else { 3 };
    let weight_bits = if mode == 1 { 3 } else { 2 };
    let endpoint_bits = match mode {
        7 => 5,
        1 => 6,
        _ => 7,
    };
    let pbits = if mode == 1 { 2 } else { 4 };
    let shared_pbits = mode == 1;
    let weight_vals = 1 << weight_bits;

    let part = get_bits_u8(reader, 6)? as usize;

    let mut endpoints = [const { Rgba::zero() }; ENDPOINTS];
    for c in 0..comps {
        for e in 0..ENDPOINTS {
            endpoints[e][c] = get_bits_u8(reader, endpoint_bits)?;
        }
    }

    let mut p_bit_array = [0; 4];
    for p in 0..pbits {
        p_bit_array[p] = get_bit(reader)?;
    }

    let mut weights = [0; 16];
    for i in 0..16 {
        weights[i] = if i == 0 || i == constants::TABLE_ANCHOR_INDEX_SECOND_SUBSET[part] as usize {
            weight_bits - 1
        } else {
            weight_bits
        };
    }

    for e in 0..ENDPOINTS {
        for c in 0..4 {
            let mode = if mode == 7 { 4 } else { 3 };
            endpoints[e][c] = if c == mode {
                255
            } else {
                let pbits_index = if shared_pbits { e >> 1 } else { e };
                let pbits = p_bit_array[pbits_index];
                bc7_dequant_p(endpoints[e][c], pbits, endpoint_bits)
            };
        }
    }

    let mut block_colors: [[Rgba; 2]; 8] = std::array::from_fn(|_| [const { Rgba::zero() }; 2]);
    for s in 0..2 {
        for i in 0..weight_vals {
            for c in 0..comps {
                block_colors[s][i][c] = bc7_interp(
                    endpoints[s * 2][c],
                    endpoints[s * 2 + 1][c],
                    i as u8,
                    weight_bits,
                );
                block_colors[s][i][3] = if comps == 3 {
                    255
                } else {
                    block_colors[s][i][3]
                };
            }
        }
    }

    for i in 0..16 {
        let part = constants::PARTITION_2[part * 16 + i] as usize;
        let weight = weights[i] as usize;
        pixels.push(block_colors[part][weight].clone());
    }

    Ok(())
}

fn unpack_mode_4_5(mode: u8, reader: &mut BitReader, pixels: &mut Vec<Rgba>) -> Result<(), Error> {
    const ENDPOINTS: usize = 2;
    const COMPS: usize = 4;
    const WEIGHT_BITS: usize = 2;
    let a_weight_bits = if mode == 4 { 3 } else { 2 };
    let endpoint_bits = if mode == 4 { 5 } else { 7 };
    let a_endpoint_bits = if mode == 4 { 6 } else { 8 };

    let comp_rot = get_bits_u8(reader, 2)?;
    let index_mode = if mode == 4 { get_bit(reader)? } else { 0 };

    let mut endpoints = [const { Rgba::zero() }; ENDPOINTS];
    for c in 0..COMPS {
        for e in 0..ENDPOINTS {
            let bits = if c == 3 {
                a_endpoint_bits
            } else {
                endpoint_bits
            };
            endpoints[e][c] = get_bits_u8(reader, bits)?;
        }
    }

    let weight_bits = if index_mode != 0 {
        [a_weight_bits, WEIGHT_BITS]
    } else {
        [WEIGHT_BITS, a_weight_bits]
    };

    let mut weights = [0; 16];
    let mut a_weights = [0; 16];

    for i in 0..16 {
        let bits = if i == 0 { 1 } else { 0 };
        let x = if index_mode != 0 {
            &mut a_weights
        } else {
            &mut weights
        };
        x[i] = get_bits_u8(reader, weight_bits[index_mode as usize] as u8 - bits)?;
    }

    for i in 0..16 {
        let bits = if i == 0 { 1 } else { 0 };
        let x = if index_mode != 0 {
            &mut weights
        } else {
            &mut a_weights
        };
        x[i] = get_bits_u8(reader, weight_bits[1 - index_mode as usize] as u8 - bits)?;
    }

    for e in 0..ENDPOINTS {
        for c in 0..4 {
            endpoints[e][c] = bc7_dequant(
                endpoints[e][c],
                if c == 3 {
                    a_endpoint_bits
                } else {
                    endpoint_bits
                },
            );
        }
    }

    let mut block_colors = [const { Rgba::zero() }; 8];
    for i in 0..(1 << weight_bits[0]) {
        for c in 0..3 {
            block_colors[i][c] = bc7_interp(
                endpoints[0][c],
                endpoints[1][c],
                i as u8,
                weight_bits[0] as u8,
            );
        }
    }

    for i in 0..(1 << weight_bits[1]) {
        block_colors[i][3] = bc7_interp(
            endpoints[0][3],
            endpoints[1][3],
            i as u8,
            weight_bits[1] as u8,
        );
    }

    for i in 0..16 {
        let Rgba {
            mut r,
            mut g,
            mut b,
            ..
        } = block_colors[weights[i] as usize].clone();
        let mut a = block_colors[a_weights[i] as usize].a;

        if comp_rot >= 1 {
            match comp_rot - 1 {
                0 => std::mem::swap(&mut a, &mut r),
                1 => std::mem::swap(&mut a, &mut g),
                2 => std::mem::swap(&mut a, &mut b),
                _ => {}
            }
        }

        pixels.push(Rgba { r, g, b, a });
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Rgba {
    pub const fn zero() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}
impl Default for Rgba {
    fn default() -> Self {
        Self::zero()
    }
}
impl std::ops::Index<usize> for Rgba {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.r,
            1 => &self.g,
            2 => &self.b,
            3 => &self.a,
            x => panic!("RGBA Index out of range: {x}"),
        }
    }
}
impl std::ops::IndexMut<usize> for Rgba {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.r,
            1 => &mut self.g,
            2 => &mut self.b,
            3 => &mut self.a,
            x => panic!("RGBA Index out of range: {x}"),
        }
    }
}

common::int_enum! {
    enum FormatMode: u8 {
        Mode0 = 0,
        Mode1 = 1,
        Mode2 = 2,
        Mode3 = 3,
        Mode4 = 4,
        Mode5 = 5,
        Mode6 = 6,
        Mode7 = 7,
    }
}

fn get_mode(reader: &mut BitReader) -> u8 {
    let mut mode = 0;
    while let Some(bit) = reader.get_bit()
        && bit != 1
    {
        mode += 1;
    }

    mode
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn file_read() {
        let file = include_bytes!("../tests/files/is_fireball.dds");
        Dds::read(file.as_slice()).unwrap();
    }
}
