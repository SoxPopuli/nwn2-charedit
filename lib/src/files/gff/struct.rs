use super::{
    bin::{Gff as BinGff, Struct as BinStruct},
    field::LabeledField,
};
use crate::{error::Error, files::tlk::Tlk};
use std::io::{Read, Seek};

/// *Warning*: duplicate labels possible?
#[derive(Debug, PartialEq)]
pub struct Struct {
    pub id: u32,
    pub fields: Vec<LabeledField>,
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
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { id: s.id, fields })
    }

    /// Searches fields for `name` using depth first search
    pub fn find_dfs(&self, name: &str) -> Option<&LabeledField> {
        use super::field::Field;
        use std::collections::VecDeque;

        let mut stack = VecDeque::from_iter(self.fields.iter());
        while !stack.is_empty() {
            if let Some(x) = stack.pop_front() {
                if x.label == name {
                    return Some(x);
                }

                match &x.field {
                    Field::Struct(s) => {
                        for f in &s.fields {
                            stack.push_front(f);
                        }
                    }
                    Field::List(l) => {
                        for s in l {
                            for f in &s.fields {
                                stack.push_front(f);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        None
    }

    /// Searches fields for `name` using depth first search
    pub fn find_dfs_mut(&mut self, name: &str) -> Option<&mut LabeledField> {
        use super::field::Field;
        use std::collections::VecDeque;

        let mut stack = VecDeque::from_iter(self.fields.iter_mut());
        while !stack.is_empty() {
            if let Some(x) = stack.pop_front() {
                if x.label == name {
                    return Some(x);
                }

                match &mut x.field {
                    Field::Struct(s) => {
                        for f in &mut s.fields {
                            stack.push_front(f);
                        }
                    }
                    Field::List(l) => {
                        for s in l {
                            for f in &mut s.fields {
                                stack.push_front(f);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        None
    }

    /// Searches fields for `name` using breadth first search
    pub fn find_bfs(&self, name: &str) -> Option<&LabeledField> {
        use super::field::Field;
        use std::collections::VecDeque;

        let mut stack = VecDeque::from_iter(self.fields.iter());
        while !stack.is_empty() {
            if let Some(x) = stack.pop_front() {
                if x.label == name {
                    return Some(x);
                }

                match &x.field {
                    Field::Struct(s) => {
                        for f in &s.fields {
                            stack.push_back(f);
                        }
                    }
                    Field::List(l) => {
                        for s in l {
                            for f in &s.fields {
                                stack.push_back(f);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        None
    }

    /// Searches fields for `name` using breadth first search
    pub fn find_bfs_mut(&mut self, name: &str) -> Option<&mut LabeledField> {
        use super::field::Field;
        use std::collections::VecDeque;

        let mut stack = VecDeque::from_iter(self.fields.iter_mut());
        while !stack.is_empty() {
            if let Some(x) = stack.pop_front() {
                if x.label == name {
                    return Some(x);
                }

                match &mut x.field {
                    Field::Struct(s) => {
                        for f in &mut s.fields {
                            stack.push_back(f);
                        }
                    }
                    Field::List(l) => {
                        for s in l {
                            for f in &mut s.fields {
                                stack.push_back(f);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        None
    }

    /// Search for `name` in direct children
    pub fn find_direct(&self, name: &str) -> Option<&LabeledField> {
        self.fields.iter().find(|f| f.label == name)
    }
}
