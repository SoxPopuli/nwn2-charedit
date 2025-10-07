// Note to self: type names ending Data usually means data as read from the file,
// i.e. before being resolved into something more useable

use super::{Offset, from_bytes_le};
use crate::{
    error::{Error, IntoError},
    files::{tlk::Tlk, write_all},
};

use std::io::{Read, Seek, Write};

pub mod bin;
pub mod exo_string;
pub mod field;
pub mod label;
pub mod r#struct;
pub mod void;
use r#struct::Struct;

pub(crate) trait Writeable {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error>;
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct FixedSizeString<const N: usize>([u8; N]);
impl<const N: usize> FixedSizeString<N> {
    /// Errors if not utf-8
    pub fn new(x: [u8; N]) -> Result<Self, Error> {
        std::str::from_utf8(&x).into_parse_error()?;
        Ok(Self(x))
    }

    pub const fn len() -> usize {
        N
    }

    pub fn to_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
impl<const N: usize> Default for FixedSizeString<N> {
    fn default() -> Self {
        Self([0u8; N])
    }
}
impl<const N: usize> AsRef<str> for FixedSizeString<N> {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}
impl<const N: usize> std::fmt::Display for FixedSizeString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
impl<const N: usize> std::fmt::Debug for FixedSizeString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FixedSizeString")
            .field(&self.to_str())
            .finish()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Header {
    /// 4-char file type string
    pub file_type: FixedSizeString<4>,
    /// 4-char GFF Version
    pub file_version: FixedSizeString<4>,

    /// Offset of Struct array as bytes from the beginning of the file
    pub struct_offset: Offset,
    /// Number of elements in Struct array
    pub struct_count: u32,

    /// Offset of Field array as bytes from the beginning of the file
    pub field_offset: Offset,
    /// Number of elements in Field array
    pub field_count: u32,

    /// Offset of Label array as bytes from the beginning of the file
    pub label_offset: Offset,
    /// Number of elements in Label array
    pub label_count: u32,

    /// Offset of Field Data as bytes from the beginning of the file
    pub field_data_offset: Offset,
    /// Number of bytes in Field Data block
    pub field_data_count: u32,

    /// Offset of Field Indices array as bytes from the beginning of the file
    pub field_indices_offset: Offset,
    /// Number of bytes in Field Indices array
    pub field_indices_count: u32,

    /// Offset of List Indices array as bytes from the beginning of the file
    pub list_indices_offset: Offset,
    /// Number of bytes in List Indices array
    pub list_indices_count: u32,
}
impl Header {
    fn read(mut data: impl Read) -> Result<Self, Error> {
        fn read_string(mut data: impl Read) -> Result<FixedSizeString<4>, Error> {
            let mut buf = [0u8; 4];
            data.read_exact(&mut buf).into_parse_error()?;

            let s = FixedSizeString::new(buf)?;

            Ok(s)
        }

        Ok(Self {
            file_type: read_string(&mut data)?,
            file_version: read_string(&mut data)?,

            struct_offset: Offset(from_bytes_le(&mut data)?),
            struct_count: from_bytes_le(&mut data)?,

            field_offset: Offset(from_bytes_le(&mut data)?),
            field_count: from_bytes_le(&mut data)?,

            label_offset: Offset(from_bytes_le(&mut data)?),
            label_count: from_bytes_le(&mut data)?,

            field_data_offset: Offset(from_bytes_le(&mut data)?),
            field_data_count: from_bytes_le(&mut data)?,

            field_indices_offset: Offset(from_bytes_le(&mut data)?),
            field_indices_count: from_bytes_le(&mut data)?,

            list_indices_offset: Offset(from_bytes_le(&mut data)?),
            list_indices_count: from_bytes_le(&mut data)?,
        })
    }

    fn write<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        write_all(writer, &self.file_type.0)?;
        write_all(writer, &self.file_version.0)?;

        write_all(writer, &self.struct_offset.0.to_le_bytes())?;
        write_all(writer, &self.struct_count.to_le_bytes())?;

        write_all(writer, &self.field_offset.0.to_le_bytes())?;
        write_all(writer, &self.field_count.to_le_bytes())?;

        write_all(writer, &self.label_offset.0.to_le_bytes())?;
        write_all(writer, &self.label_count.to_le_bytes())?;

        write_all(writer, &self.field_data_offset.0.to_le_bytes())?;
        write_all(writer, &self.field_data_count.to_le_bytes())?;

        write_all(writer, &self.field_indices_offset.0.to_le_bytes())?;
        write_all(writer, &self.field_indices_count.to_le_bytes())?;

        write_all(writer, &self.list_indices_offset.0.to_le_bytes())?;
        write_all(writer, &self.list_indices_count.to_le_bytes())?;

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Gff {
    pub file_type: FixedSizeString<4>,
    pub file_version: FixedSizeString<4>,
    pub root: Struct,
}
impl Gff {
    pub fn from_binary<R>(gff: &bin::Gff, tlk: Option<&Tlk<R>>) -> Result<Self, Error>
    where
        R: Read + Seek,
    {
        let root = gff.structs.first().expect("Missing root struct");

        Ok(Self {
            file_type: gff.header.file_type,
            file_version: gff.header.file_version,
            root: Struct::new(root, gff, tlk)?,
        })
    }

    pub fn to_binary(&self) -> bin::Gff {
        bin::Gff::from_data(self)
    }

    pub fn read<A, B>(data: A, tlk: Option<&Tlk<B>>) -> Result<Self, Error>
    where
        A: Read + Seek,
        B: Read + Seek,
    {
        let bin = bin::Gff::read(data)?;
        Self::from_binary(&bin, tlk)
    }

    pub fn read_without_tlk(data: impl Read + Seek) -> Result<Self, Error> {
        use std::io::Cursor;
        Self::read::<_, Cursor<Vec<u8>>>(data, None)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.to_binary().write(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::io::Cursor;

    #[test]
    fn header_write_test() {
        let header = Header {
            file_type: FixedSizeString::new(*b"IFO ").unwrap(),
            file_version: FixedSizeString::new(*b"V3.2").unwrap(),
            struct_offset: Offset(1),
            struct_count: 2,
            field_offset: Offset(3),
            field_count: 4,
            label_offset: Offset(5),
            label_count: 6,
            field_data_offset: Offset(7),
            field_data_count: 8,
            field_indices_offset: Offset(9),
            field_indices_count: 10,
            list_indices_offset: Offset(11),
            list_indices_count: 12,
        };

        let mut buf = Cursor::new(vec![]);
        header.write(&mut buf).unwrap();
        buf.rewind().unwrap();

        let header_2 = Header::read(buf).unwrap();

        assert_eq!(header, header_2);
    }

    fn read_tlk_and_gff<A, B>(gff_file: A, tlk_file: B) -> (Tlk<B>, Gff)
    where
        A: Read + Seek,
        B: Read + Seek,
    {
        let tlk = Tlk::read(tlk_file).unwrap();
        let gff = bin::Gff::read(gff_file).unwrap();

        let gff = Gff::from_binary(&gff, Some(&tlk)).unwrap();

        (tlk, gff)
    }

    #[test]
    fn read_test() {
        let gff_file = Cursor::new(include_bytes!("../../tests/files/playerlist.ifo"));
        let tlk_file = Cursor::new(include_bytes!("../../tests/files/dialog.TLK"));

        let (_, gff) = read_tlk_and_gff(gff_file, tlk_file);

        println!("{:#?}", gff.root);
    }

    #[test]
    fn write_test() {
        let mut gff_file = Cursor::new(include_bytes!("../../tests/files/playerlist.ifo"));
        let tlk_file = Cursor::new(include_bytes!("../../tests/files/dialog.TLK"));

        let tlk = Tlk::read(tlk_file).unwrap();

        let gff_bin = bin::Gff::read(&mut gff_file).unwrap();
        let gff = Gff::from_binary(&gff_bin, Some(&tlk)).unwrap();

        let gff_2_bin = bin::Gff::from_data(&gff);

        assert_eq!(gff_bin.header, gff_2_bin.header);
        assert_eq!(gff_bin.field_data, gff_2_bin.field_data);

        assert_eq!(gff_bin.labels, gff_2_bin.labels);
        assert_eq!(gff_bin.fields, gff_2_bin.fields);
        assert_eq!(gff_bin.structs, gff_2_bin.structs);

        // Takes too long to print with pretty_assertions
        ::core::assert_eq!(gff_bin, gff_2_bin);

        let gff_2 = Gff::from_binary(&gff_2_bin, Some(&tlk)).unwrap();
        assert_eq!(gff, gff_2);

        let mut buf = Cursor::new(vec![]);
        gff_2.write(&mut buf).unwrap();

        gff_file.rewind().unwrap();
        assert_eq!(buf.into_inner(), gff_file.into_inner());
    }

    #[test]
    fn find_test() {
        use crate::files::{Gender, Language};
        use exo_string::*;
        let gff_file = Cursor::new(include_bytes!("../../tests/files/playerlist.ifo"));

        let gff = Gff::read_without_tlk(gff_file).unwrap();

        let expected = {
            let exo_string = ExoLocString {
                str_ref: 4294967295,
                tlk_string: None,
                substrings: vec![ExoLocSubString {
                    data: "Cassie".into(),
                    gender: Gender::Masculine,
                    language: Language::English,
                }],
            };
            field::Field::ExoLocString(exo_string)
        };

        {
            let first_name = gff
                .root
                .bfs_iter()
                .find(|x| x.has_label("FirstName"))
                .and_then(|x| match x.read() {
                    Ok(lock) => Some(lock.field.clone()),
                    _ => None,
                })
                .unwrap();

            assert_eq!(first_name, expected);
        }
        {
            let first_name = gff
                .root
                .dfs_iter()
                .find(|x| x.has_label("FirstName"))
                .and_then(|x| match x.read() {
                    Ok(lock) => Some(lock.field.clone()),
                    _ => None,
                })
                .unwrap();

            assert_eq!(first_name, expected);
        }
    }
}
