use super::{
    bin::{Gff as BinGff, Struct as BinStruct},
    field::LabeledField,
};
use crate::{error::Error, files::tlk::Tlk};
use std::{
    io::{Read, Seek},
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub struct StructField(pub Arc<RwLock<LabeledField>>);
impl std::cmp::PartialEq for StructField {
    fn eq(&self, other: &Self) -> bool {
        let lhs = self.0.read().unwrap();
        let rhs = other.0.read().unwrap();

        lhs.eq(&rhs)
    }
}
impl std::ops::Deref for StructField {
    type Target = Arc<RwLock<LabeledField>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl StructField {
    pub fn new(field: LabeledField) -> Self {
        let inner = Arc::new(RwLock::new(field));
        Self(inner)
    }

    pub fn has_label(&self, x: &str) -> bool {
        match self.read() {
            Ok(lock) => lock.label == x,
            _ => false,
        }
    }

    pub fn has_label_case_insensitive(&self, x: &str) -> bool {
        match self.read() {
            Ok(lock) => lock.label.eq_ignore_ascii_case(x),
            _ => false,
        }
    }
}

/// *Warning*: duplicate labels possible?
#[derive(Debug, PartialEq, Clone)]
pub struct Struct {
    pub id: u32,
    pub fields: Vec<StructField>,
}
impl Struct {
    pub fn new<R>(s: &BinStruct, gff: &BinGff, tlk: Option<&Tlk<R>>) -> Result<Self, Error>
    where
        R: Read + Seek,
    {
        let fields = (0..s.field_count)
            .map(|i| {
                let field = s
                    .get_field(gff, i)
                    .ok_or_else(|| Error::ParseError(format!("Field index {i} not found")))?;

                let label = gff.labels[field.label_index as usize].clone();
                let field_data = field.to_field(gff, tlk)?;

                let labeled_field = LabeledField {
                    label: label.clone(),
                    field: field_data,
                };

                Ok::<_, Error>(labeled_field)
            })
            .map(|x| x.map(StructField::new))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { id: s.id, fields })
    }

    /// Searches fields for `name` using depth first search
    pub fn dfs_iter(&self) -> impl Iterator<Item = StructField> {
        use super::field::Field;
        use std::collections::VecDeque;

        let mut stack = VecDeque::from_iter(self.fields.iter().cloned());

        std::iter::from_fn(move || {
            if let Some(x) = stack.pop_front() {
                let lock = x.read().unwrap();
                match &lock.field {
                    Field::Struct(s) => {
                        for f in &s.fields {
                            stack.push_front(f.clone());
                        }
                    }
                    Field::List(l) => {
                        for s in l {
                            for f in &s.fields {
                                stack.push_front(f.clone());
                            }
                        }
                    }
                    _ => {}
                }

                Some(x.clone())
            } else {
                None
            }
        })
    }

    /// Searches fields for `name` using breadth first search
    pub fn bfs_iter(&self) -> impl Iterator<Item = StructField> {
        use super::field::Field;
        use std::collections::VecDeque;

        let mut stack = VecDeque::from_iter(self.fields.iter().cloned());

        std::iter::from_fn(move || {
            if let Some(x) = stack.pop_front() {
                let lock = x.0.read().expect("Failed to lock struct field");

                match &lock.field {
                    Field::Struct(s) => {
                        for f in &s.fields {
                            stack.push_back(f.clone());
                        }
                    }
                    Field::List(l) => {
                        for s in l {
                            for f in &s.fields {
                                stack.push_back(f.clone());
                            }
                        }
                    }
                    _ => {}
                }

                Some(x.clone())
            } else {
                None
            }
        })
    }

    /// Search for `name` in direct children
    pub fn find_direct(&self, name: &str) -> Option<StructField> {
        self.fields.iter().find(|f| f.has_label(name)).cloned()
    }
}
