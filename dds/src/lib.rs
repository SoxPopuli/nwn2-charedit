use std::io::{Error, Read};

// DDS Format: https://learn.microsoft.com/en-us/windows/win32/direct3ddds/dx-graphics-dds-pguide
// BC7 Format: https://learn.microsoft.com/en-us/windows/win32/direct3d11/bc7-format
// BC7 Format Mode Reference: https://learn.microsoft.com/en-us/windows/win32/direct3d11/bc7-format-mode-reference

macro_rules! read {
    ($reader: expr) => {{
        use rust_utils::byte_readers::*;

        $reader.read_le()
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
        fn reserved1<R: Read>(reader: &mut R) -> Result<[u32; 11], Error> {
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
        }

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
                x.try_into().expect("Unexpected resource dimension")
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

#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

mod ffi {
    use super::Rgba;
    use std::ffi::c_void;

    unsafe extern "C" {
        pub unsafe fn unpack_bc7(block_ptr: *const c_void, pixel_ptr: *mut Rgba) -> bool;
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Dds {
    pub four_cc: FourCC,
    pub header: Header,
    pub header_extra: Option<HeaderExtra>,
    pub pixels: Vec<Rgba>,
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
        }?;
        let data_ptr = data.as_ptr();

        let mut block_idx = 0;
        let block_ptrs = std::iter::from_fn(move || {
            let ptr = unsafe { data_ptr.add(128 * block_idx) };
            block_idx += 1;
            if ptr.addr() >= data_ptr.addr() + data.len() {
                None
            } else {
                Some(ptr)
            }
        });

        let mut pixels = vec![Rgba::default(); header.width as usize * header.height as usize];
        let pixels_ptr = pixels.as_mut_ptr();

        unsafe {
            block_ptrs.enumerate().for_each(|(i, block_ptr)| {
                let first_empty = pixels.iter()
                    .enumerate()
                    .find(|x| *x.1 == Rgba::default())
                    .map(|(i, _)| i)
                    .unwrap();

                let pixel_ptr = pixels_ptr.add(first_empty);
                ffi::unpack_bc7(block_ptr.cast(), pixel_ptr);
            });

            // ffi::unpack_bc7(data_ptr.cast(), pixels_ptr)
        };

        Ok(Self {
            four_cc,
            header,
            header_extra,
            pixels,
        })
    }
}

pub fn decode() {}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn file_read() {
        let file = include_bytes!("../../lib/src/tests/files/is_fireball.dds");
        let file = BufReader::new(Cursor::new(file));
        let dds = Dds::read(file).unwrap();

        // panic!("{:?}", dds.pixels);
    }
}
