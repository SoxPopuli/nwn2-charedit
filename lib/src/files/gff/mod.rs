// Note to self: type names ending Data usually means data as read from the file,
// i.e. before being resolved into something more useable

use super::{from_bytes_le, Offset};
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

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Header {
    /// 4-char file type string
    pub file_type: Box<str>,
    /// 4-char GFF Version
    pub file_version: Box<str>,

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
        fn read_string(mut data: impl Read) -> Result<Box<str>, Error> {
            let mut buf = [0u8; 4];
            data.read_exact(&mut buf).into_parse_error()?;

            let s = encoding_rs::WINDOWS_1252
                .decode_without_bom_handling(&buf)
                .0
                .into();

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
        write_all(writer, &self.file_type.as_bytes()[..4])?;
        write_all(writer, &self.file_version.as_bytes()[..4])?;

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
    file_type: Box<str>,
    file_version: Box<str>,
    root: Struct,
}
impl Gff {
    pub fn from_binary<R>(gff: &bin::Gff, tlk: &Tlk<R>) -> Result<Self, Error>
    where
        R: Read + Seek,
    {
        let root = gff.structs.first().expect("Missing root struct");

        Ok(Self {
            file_type: gff.header.file_type.clone(),
            file_version: gff.header.file_version.clone(),
            root: Struct::new(root, gff, tlk)?,
        })
    }

    pub fn write<W>(&self, file_type: &str, file_version: &str, writer: &mut W) -> Result<(), Error>
    where
        W: Write,
    {
        // Store all the data in `vec`s then work out the offsets
        // when writing

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn header_write_test() {
        let header = Header {
            file_type: "IFO ".into(),
            file_version: "V3.2".into(),
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

        let gff = Gff::from_binary(&gff, &tlk).unwrap();

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
        let gff = Cursor::new(include_bytes!("../../tests/files/playerlist.ifo"));
        let mut tlk = Cursor::new(include_bytes!("../../tests/files/dialog.TLK"));

        let (_, gff) = read_tlk_and_gff(gff, &mut tlk);
        tlk.rewind().unwrap();

        let mut buf = Cursor::new(vec![]);
        gff.write(&gff.file_type, &gff.file_version, &mut buf)
            .unwrap();
        buf.rewind().unwrap();

        let (_, gff_2) = read_tlk_and_gff(buf, tlk);

        assert_eq!(gff, gff_2)
    }
}
