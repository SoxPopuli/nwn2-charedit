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
    four_cc: [u8; 4],
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
            four_cc: read!(reader)?,
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
    pub size: u32,
    pub flags: u32,
    pub height: u32,
    pub width: u32,
    pub pitch_or_linear_size: u32,
    pub depth: u32,
    pub mip_map_count: u32,
    reserved1: [u32; 11],
    pub pixel_format: DdsPixelFormat,
    pub caps: u32,
    pub caps2: u32,
    pub caps3: u32,
    pub caps4: u32,
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
    BC6H_TYPELESS = 94,
    BC6H_UF16 = 95,
    BC6H_SF16 = 96,
    BC7_TYPELESS = 97,
    BC7_UNORM = 98,
    BC7_UNORM_SRGB = 99,
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

#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone)]
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

mod ffi {
    use std::ffi::{c_int, c_void};

    unsafe extern "C" {
        pub unsafe fn bcdec_bc7(
            compressed_block: *const c_void,
            decompressed_block: *mut c_void,
            destination_pitch: c_int,
        );
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Dds {
    pub four_cc: [u8; 4],
    pub header: Header,
    pub header_extra: Option<HeaderExtra>,
    pub pixels: Vec<Rgba>,
}
impl Dds {
    pub fn read<R>(mut reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        let four_cc = read!(reader)?;
        let header = Header::read(&mut reader)?;

        let header_extra = if header.pixel_format.flags.has_flag(PixelFormatFlags::FourCC)
            && header.pixel_format.four_cc.eq(b"DX10")
        {
            Some(HeaderExtra::read(&mut reader)?)
        } else {
            None
        };

        assert_eq!(
            header_extra.as_ref().map(|x| x.dxgi_format),
            Some(DXGIFormat::BC7_UNORM)
        );

        let data = {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).map(|_| buf)
        }?;
        let mut data_ptr = data.as_ptr();

        let mut pixels =
            vec![const { Rgba::zero() }; header.width as usize * header.height as usize];
        let pixels_ptr = pixels.as_mut_ptr();

        let w = header.width;
        let h = header.height;

        unsafe {
            for i in (0..h).step_by(4) {
                for j in (0..w).step_by(4) {
                    let dst: *mut u8 = pixels_ptr.cast();
                    let dst = dst.add((i as usize * w as usize + j as usize) * 4);

                    ffi::bcdec_bc7(data_ptr.cast(), dst.cast(), w as i32 * 4);
                    data_ptr = data_ptr.add(16);
                }
            }
        };

        Ok(Self {
            four_cc,
            header,
            header_extra,
            pixels,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::{BufReader, BufWriter, Cursor};

    #[test]
    fn file_read() {
        let file = include_bytes!("../../lib/src/tests/files/is_fireball.dds");
        let file = BufReader::new(Cursor::new(file));
        let dds = Dds::read(file).unwrap();

        {
            let out_file = BufWriter::new(Vec::new());
            // let out_file = std::fs::File::create("fireball.png").unwrap();
            let mut encoder = png::Encoder::new(out_file, dds.header.width, dds.header.height);

            let pixel_ptr = unsafe {
                std::slice::from_raw_parts(dds.pixels.as_ptr().cast(), dds.pixels.len() * 4)
            };

            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);

            let mut encoder = encoder.write_header().unwrap();
            encoder.write_image_data(pixel_ptr).unwrap();
            encoder.finish().unwrap();
        }
    }
}
