use super::{
    Header,
    label::{LABEL_SIZE, Label},
};
use crate::{
    error::{Error, IntoError},
    files::{
        Offset, from_bytes_le,
        gff::{
            Writeable,
            exo_string::{ExoLocString, ExoString},
            field::U32Char,
            void::Void,
        },
        res_ref::ResRef,
        tlk::Tlk,
        write_all,
    },
    int_enum,
};
use rust_utils::collect_vec::CollectVecResult;
use std::{
    collections::HashMap,
    io::{Read, Seek, Write},
};

const fn u32_size_of<T>() -> u32 {
    size_of::<T>() as u32
}

const INDEX_SIZE: u32 = u32_size_of::<u32>();

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Gff {
    pub header: Header,
    pub structs: Vec<Struct>,
    pub fields: Vec<Field>,
    pub labels: Vec<Label>,
    pub field_data: Vec<u8>,
    pub field_indices: Vec<u32>,
    pub list_indices: Vec<u32>,
}
impl Gff {
    pub fn read(mut data: impl Read + Seek) -> Result<Self, Error> {
        let header = Header::read(&mut data)?;

        header.struct_offset.seek_to(&mut data)?;

        let structs = (0..header.struct_count)
            .map(|_| Struct::read(&mut data))
            .collect_vec_result()?;

        header.field_offset.seek_to(&mut data)?;

        let fields = (0..header.field_count)
            .map(|_| Field::read(&mut data))
            .collect_vec_result()?;

        header.label_offset.seek_to(&mut data)?;

        let labels = (0..header.label_count)
            .map(|_| Label::read(&mut data))
            .collect_vec_result()?;

        header.field_data_offset.seek_to(&mut data)?;

        let field_data = {
            let mut buf = vec![0u8; header.field_data_count as usize];
            data.read_exact(&mut buf).into_parse_error()?;
            buf
        };

        header.field_indices_offset.seek_to(&mut data)?;

        let field_indices = {
            (0..header.field_indices_count / INDEX_SIZE)
                .map(|_| from_bytes_le(&mut data))
                .collect_vec_result()
                .into_parse_error()
        }?;

        header.list_indices_offset.seek_to(&mut data)?;

        let list_indices = {
            (0..header.list_indices_count / INDEX_SIZE)
                .map(|_| from_bytes_le(&mut data))
                .collect_vec_result()
                .into_parse_error()
        }?;

        Ok(Self {
            header,
            structs,
            fields,
            labels,
            field_data,
            field_indices,
            list_indices,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.header.write(writer)?;

        for s in &self.structs {
            s.write(writer)?;
        }

        for f in &self.fields {
            f.write(writer)?;
        }

        for l in &self.labels {
            l.write(writer)?;
        }

        write_all(writer, &self.field_data)?;

        for fi in &self.field_indices {
            write_all(writer, &fi.to_le_bytes())?;
        }

        for li in &self.list_indices {
            write_all(writer, &li.to_le_bytes())?;
        }

        Ok(())
    }

    /// Stores *label* -> *label index* in `label_map`
    /// *Returns*: label index
    fn register_label(
        &mut self,
        label_map: &mut HashMap<Label, u32>,
        label: &super::label::Label,
    ) -> u32 {
        if let Some(index) = label_map.get(label) {
            *index
        } else {
            let new_index = label_map.len() as u32;
            label_map.insert(label.clone(), new_index);

            new_index
        }
    }

    /// *Returns*: data_or_data_offset
    fn store_field(
        &mut self,
        label_map: &mut HashMap<Label, u32>,
        labeled_field: &super::field::LabeledField,
    ) -> u32 {
        fn write_to_data(item: impl Writeable, data: &mut Vec<u8>) -> u32 {
            let offset = data.len();
            item.write(data).expect("Failed to write to data");
            offset as u32
        }

        macro_rules! write_primitive {
            ($val: expr) => {{
                let offset = self.field_data.len();
                let bytes = $val.to_le_bytes();
                self.field_data.extend_from_slice(&bytes);
                offset as u32
            }};
        }

        let label_index = self.register_label(label_map, &labeled_field.label);
        let f = Field {
            id: labeled_field.field.get_field_type(),
            label_index,
            data_or_data_offset: 0,
        };

        let field_index = self.fields.len();
        self.fields.push(f);

        use super::field::Field::*;
        let offset = match &labeled_field.field {
            Byte(b) => *b as u32,
            ExoLocString(s) => write_to_data(s, &mut self.field_data),
            ExoString(s) => write_to_data(s, &mut self.field_data),
            Char(c) => c.0,
            ResRef(r) => write_to_data(r, &mut self.field_data),
            Double(d) => write_primitive!(d),
            DWord(w) => *w,
            DWord64(w) => write_primitive!(w),
            Float(f) => {
                let bytes = f.to_le_bytes();
                u32::from_le_bytes(bytes)
            }
            Int(i) => *i as u32,
            Int64(i) => write_primitive!(i),
            Short(s) => *s as u32,
            Void(v) => write_to_data(v, &mut self.field_data),
            Word(w) => *w as u32,
            Struct(s) => self.store_struct(label_map, s),
            List(l) => {
                let offset = self.list_indices.len();
                let struct_count = l.len() as u32;

                self.list_indices.push(struct_count);
                self.list_indices
                    .resize(self.list_indices.len() + l.len(), 0);
                for (i, s) in l.iter().enumerate() {
                    let index = offset + i + 1;

                    let struct_index = self.store_struct(label_map, s);
                    self.list_indices[index] = struct_index;
                }

                offset as u32 * INDEX_SIZE
            }
        };

        self.fields[field_index].data_or_data_offset = offset;
        field_index as u32
    }

    /// *Returns*: struct index
    fn store_struct(&mut self, label_map: &mut HashMap<Label, u32>, s: &super::Struct) -> u32 {
        let field_count = s.fields.len() as u32;

        let bin_struct = Struct {
            id: s.id,
            field_count,
            data_or_data_offset: 0,
        };

        let struct_index = self.structs.len();
        self.structs.push(bin_struct.clone());

        let offset = if field_count == 0 {
            s.original_data_or_data_offset
        } else if s.fields.len() == 1 {
            //Index into field array
            let field = &s.fields[0].read().unwrap();
            self.store_field(label_map, field)
        } else {
            // Byte offset into field indices
            let index_offset = self.field_indices.len();
            self.field_indices
                .resize(self.field_indices.len() + s.fields.len(), 0);

            for (i, f) in s.fields.iter().enumerate() {
                let field = f.read().unwrap();
                let index = self.store_field(label_map, &field);
                self.field_indices[index_offset + i] = index;
            }

            index_offset as u32 * INDEX_SIZE
        };

        self.structs[struct_index].data_or_data_offset = offset;

        struct_index as u32
    }

    pub fn from_data(data: &super::Gff) -> Self {
        let header = Header {
            file_type: data.file_type,
            file_version: data.file_version,
            struct_offset: Offset(u32_size_of::<Header>()),
            ..Default::default()
        };

        let mut this = Self {
            header,
            ..Default::default()
        };

        let mut label_map = HashMap::default();

        this.store_struct(&mut label_map, &data.root);

        let labels: Vec<Label> = {
            let mut labels = vec![];
            labels.resize_with(label_map.len(), std::mem::MaybeUninit::uninit);

            for (label, index) in label_map {
                labels[index as usize].write(label);
            }

            unsafe { std::mem::transmute(labels) }
        };

        this.labels = labels;

        let header = &mut this.header;

        header.field_count = this.fields.len() as u32;
        header.label_count = this.labels.len() as u32;
        header.struct_count = this.structs.len() as u32;
        header.field_data_count = this.field_data.len() as u32;
        header.list_indices_count = this.list_indices.len() as u32 * INDEX_SIZE;
        header.field_indices_count = this.field_indices.len() as u32 * INDEX_SIZE;

        header.field_offset =
            header.struct_offset + (header.struct_count * u32_size_of::<Struct>());
        header.label_offset = header.field_offset + (header.field_count * FIELD_SIZE);
        header.field_data_offset = header.label_offset + (header.label_count * LABEL_SIZE as u32);
        header.field_indices_offset = header.field_data_offset + header.field_data_count;
        header.list_indices_offset = header.field_indices_offset + header.field_indices_count;

        this
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Struct {
    pub id: u32,
    pub data_or_data_offset: u32,
    pub field_count: u32,
}
impl Struct {
    pub fn read(mut data: impl Read) -> Result<Self, Error> {
        let struct_type = from_bytes_le(&mut data)?;
        let data_or_data_offset = from_bytes_le(&mut data)?;
        let field_count = from_bytes_le(&mut data)?;

        Ok(Self {
            id: struct_type,
            data_or_data_offset,
            field_count,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        write_all(writer, &self.id.to_le_bytes())?;
        write_all(writer, &self.data_or_data_offset.to_le_bytes())?;
        write_all(writer, &self.field_count.to_le_bytes())?;

        Ok(())
    }

    pub fn get_field<'a>(&self, file: &'a Gff, index: u32) -> Option<&'a Field> {
        if index >= self.field_count {
            return None;
        }

        if self.field_count == 0 {
            None
        } else if self.field_count == 1 {
            // Index into field array
            let field = &file.fields[self.data_or_data_offset as usize];
            Some(field)
        } else {
            // Byte offset into field indices
            assert!(
                self.data_or_data_offset.is_multiple_of(4),
                "Data index {} not aligned on u32 boundary :(",
                self.data_or_data_offset
            );

            let index = (self.data_or_data_offset / 4) + index;
            let field_index = file.field_indices[index as usize];
            let field = &file.fields[field_index as usize];

            Some(field)
        }
    }
}

fn shrink_array<const BIG: usize, const SMALL: usize>(x: &[u8; BIG]) -> [u8; SMALL] {
    assert!(BIG >= SMALL, "Tried to shrink {x:?} to size {SMALL}");

    std::array::from_fn(|i| x[i])
}

const FIELD_SIZE: u32 = size_of::<u32>() as u32 * 3;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field {
    pub id: FieldType,
    pub label_index: u32,
    pub data_or_data_offset: u32,
}
impl Field {
    fn read(mut data: impl Read) -> Result<Self, Error> {
        let index = {
            let index: u32 = from_bytes_le(&mut data)?;
            FieldType::try_from(index as u8)?
        };
        let label_index = from_bytes_le(&mut data)?;
        let data_or_data_offset = from_bytes_le(&mut data)?;

        Ok(Self {
            id: index,
            label_index,
            data_or_data_offset,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let index = self.id.as_num() as u32;
        write_all(writer, &index.to_le_bytes()).into_write_error()?;
        write_all(writer, &self.label_index.to_le_bytes())?;
        write_all(writer, &self.data_or_data_offset.to_le_bytes())?;

        Ok(())
    }

    pub fn to_field<R>(
        &self,
        file: &Gff,
        tlk: Option<&Tlk<R>>,
    ) -> Result<super::field::Field, Error>
    where
        R: Read + Seek,
    {
        macro_rules! read_smaller {
            ($t: ty) => {{
                let bytes = self.data_or_data_offset.to_le_bytes();
                let data = <$t>::from_le_bytes(shrink_array(&bytes));

                data
            }};
        }

        macro_rules! read_complex {
            ($t: ty, $data_source: expr) => {{
                const DATA_SIZE: usize = size_of::<$t>();

                let index = self.data_or_data_offset as usize;
                let data = &$data_source[index..index + DATA_SIZE];

                let mut buf = [0u8; DATA_SIZE];
                buf.copy_from_slice(data);

                <$t>::from_le_bytes(buf)
            }};
        }

        use super::field::Field;

        fn field_data_offset(file: &Gff, offset: u32) -> &[u8] {
            &file.field_data[offset as usize..]
        }

        match self.id {
            FieldType::Byte => {
                assert!(self.data_or_data_offset <= 255);
                let bytes = self.data_or_data_offset.to_le_bytes();
                Ok(Field::Byte(bytes[0]))
            }
            FieldType::Char => Ok(Field::Char(U32Char(self.data_or_data_offset))),
            FieldType::Word => Ok(Field::Word(read_smaller!(u16))),
            FieldType::Short => Ok(Field::Short(read_smaller!(i16))),
            FieldType::DWord => Ok(Field::DWord(self.data_or_data_offset)),
            FieldType::Int => Ok(Field::Int(self.data_or_data_offset as i32)),
            FieldType::DWord64 => Ok(Field::DWord64(read_complex!(u64, file.field_data))),
            FieldType::Int64 => Ok(Field::Int64(read_complex!(i64, file.field_data))),
            FieldType::Float => Ok(Field::Float(read_smaller!(f32))),
            FieldType::Double => Ok(Field::Double(read_complex!(f64, file.field_data))),
            FieldType::ExoString => {
                let mut data = field_data_offset(file, self.data_or_data_offset);

                let exo_string = ExoString::read(&mut data)?;
                Ok(Field::ExoString(exo_string))
            }
            FieldType::ResRef => {
                let mut data = field_data_offset(file, self.data_or_data_offset);

                let res_ref = ResRef::read(&mut data)?;
                Ok(Field::ResRef(res_ref))
            }
            FieldType::ExoLocString => {
                let mut data = field_data_offset(file, self.data_or_data_offset);

                let s = ExoLocString::read(&mut data, tlk)?;

                Ok(Field::ExoLocString(s))
            }
            FieldType::Void => {
                let mut data = field_data_offset(file, self.data_or_data_offset);

                Ok(Field::Void(Void::read(&mut data)?))
            }
            FieldType::Struct => {
                let index = self.data_or_data_offset as usize;
                let s = &file.structs[index];

                Ok(Field::Struct(super::Struct::new(s, file, tlk)?))
            }
            FieldType::List => {
                let index = (self.data_or_data_offset / INDEX_SIZE) as usize;
                let struct_count = file.list_indices[index] as usize;

                let start = index + 1;
                let end = start + struct_count;

                let structs = file.list_indices[start..end]
                    .iter()
                    .map(|i| {
                        let s = &file.structs[*i as usize];
                        super::Struct::new(s, file, tlk)
                    })
                    .collect_vec_result()?;

                Ok(Field::List(structs))
            }
            FieldType::Invalid => panic!("to_field called on invalid field type"),
        }
    }
}

int_enum! {
    pub enum FieldType: u8 {
        Byte = 0,
        Char = 1,
        Word = 2,
        Short = 3,
        DWord = 4,
        Int = 5,
        DWord64 = 6,
        Int64 = 7,
        Float = 8,
        Double = 9,
        ExoString = 10,
        ResRef = 11,
        ExoLocString = 12,
        Void = 13,
        Struct = 14,
        List = 15,
        Invalid = 255
    }
}
impl FieldType {
    // A type is complex if it can't be represented using only 4 bytes
    pub fn is_complex(&self) -> bool {
        match self {
            FieldType::Byte
            | FieldType::Char
            | FieldType::Word
            | FieldType::Short
            | FieldType::DWord
            | FieldType::Int
            | FieldType::Invalid
            | FieldType::Float => false,
            FieldType::DWord64
            | FieldType::Int64
            | FieldType::Double
            | FieldType::ExoString
            | FieldType::ResRef
            | FieldType::ExoLocString
            | FieldType::Void
            | FieldType::Struct
            | FieldType::List => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FieldType, Gff};
    use crate::files::gff::{
        field::{Field, LabeledField},
        label::Label,
    };
    use std::collections::HashMap;

    #[test]
    fn register_label_test() {
        let mut file = Gff::default();
        let mut label_map = HashMap::new();

        let labels = [
            Label("hello".into()),
            Label("hello".into()),
            Label("hello".into()),
            Label("goodbye".into()),
        ];

        labels.iter().for_each(|l| {
            file.register_label(&mut label_map, l);
        });

        assert_eq!(label_map.len(), 2);

        assert_eq!(
            label_map,
            HashMap::from_iter([(Label("hello".into()), 0), (Label("goodbye".into()), 1),])
        )
    }

    fn setup_store_test(field: Field) -> (Gff, HashMap<Label, u32>, LabeledField) {
        let file = Gff::default();
        let label_map = HashMap::new();

        let labeled_field = LabeledField {
            label: Label("hello".into()),
            field,
        };

        (file, label_map, labeled_field)
    }

    #[test]
    fn store_int_field_test() {
        let (mut file, mut label_map, labeled_field) = setup_store_test(Field::Int(4));

        file.store_field(&mut label_map, &labeled_field);

        assert_eq!(label_map.len(), 1);
        assert_eq!(
            file.fields,
            [super::Field {
                id: FieldType::Int,
                label_index: 0,
                data_or_data_offset: 4
            }]
        );
    }

    #[test]
    fn store_int64_field_test() {
        let (mut file, mut label_map, labeled_field) = setup_store_test(Field::Int64(8));

        file.store_field(&mut label_map, &labeled_field);
        assert_eq!(label_map.len(), 1);
        assert_eq!(
            file.fields,
            [super::Field {
                id: FieldType::Int64,
                label_index: 0,
                data_or_data_offset: 0
            }]
        );
        assert_eq!(file.field_data, 8i64.to_le_bytes())
    }
}
