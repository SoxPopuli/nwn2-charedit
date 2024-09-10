#![allow(unused)]

use crate::error::Error::{self, *};
use crate::utils::sequence_result;

use roxmltree::Node;

#[derive(Debug, PartialEq)]
pub struct Entry<T> {
    pub name: String,
    pub value: T,
}

#[derive(Default, Debug, PartialEq)]
pub struct Globals {
    pub integers: Vec<Entry<i32>>,
    pub booleans: Vec<Entry<bool>>,
    pub floats: Vec<Entry<f32>>,
    pub strings: Vec<Entry<String>>,
}

impl Globals {
    pub fn read(data: &str) -> Result<Self, Error> {
        let mut globals = Globals::default();

        let doc = roxmltree::Document::parse(data)?;

        for child in doc.root().first_child().unwrap().children() {
            match child.tag_name().name() {
                "Integers" => {
                    add_pairs_to_vec(child, &mut globals.integers, |s| Ok(s.parse::<i32>()?))?
                }
                "Booleans" => {
                    add_pairs_to_vec(child, &mut globals.booleans, |s| Ok(parse_bool(&s)?))?
                }
                "Floats" => {
                    add_pairs_to_vec(child, &mut globals.floats, |s| Ok(s.parse::<f32>()?))?
                }
                "Strings" => add_pairs_to_vec(child, &mut globals.strings, |s| Ok(s))?,

                _ => continue,
            }
        }

        Ok(globals)
    }
}

fn get_name_value_pairs(category_node: Node) -> Vec<(String, String)> {
    category_node
        .children()
        .filter(|node| node.has_children())
        .map(|node| {
            let mut name = None;
            let mut value = None;

            for e in node.children() {
                match e.tag_name().name() {
                    "Name" => name = e.text(),
                    "Value" => value = e.text(),
                    _ => continue,
                }
            }

            let name = name.expect(&format!("Missing name in: {}", node.tag_name().name()));
            let value = value.expect(&format!("Missing value in: {}", node.tag_name().name()));

            (name.into(), value.into())
        })
        .collect()
}

fn add_pairs_to_vec<T>(
    node: Node,
    v: &mut Vec<Entry<T>>,
    parse_fn: impl Fn(String) -> Result<T, Error>,
) -> Result<(), Error> {
    let elements = get_name_value_pairs(node)
        .into_iter()
        .map(|(name, value)| (name, parse_fn(value)))
        .map(|(name, value)| match value {
            Ok(value) => Ok(Entry { name, value }),
            Err(e) => Err(e),
        });

    for e in elements {
        let e = e?;
        v.push(e)
    }

    Ok(())
}

fn parse_bool(b: &str) -> Result<bool, Error> {
    match b.to_ascii_lowercase().as_ref() {
        "true" => Ok(true),
        "false" => Ok(false),
        x => Err(ParseError(format!("Unexpected bool value: {x}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_test() {
        let xml = r"
            <Globals>
                <Integers>
                    <Integer>
                        <Name>Int 1</Name>
                        <Value>1</Value>
                    </Integer>
                    <Integer>
                        <Name>Int 2</Name>
                        <Value>2</Value>
                    </Integer>
                </Integers>
                <Booleans>
                    <Boolean>
                        <Name>Bool 1</Name>
                        <Value>true</Value>
                    </Boolean>
                    <Boolean>
                        <Name>Bool 2</Name>
                        <Value>False</Value>
                    </Boolean>
                </Booleans>
                <Floats>
                    <Float>
                        <Name>Float 1</Name>
                        <Value>1</Value>
                    </Float>
                    <Float>
                        <Name>Float 2</Name>
                        <Value>2.0</Value>
                    </Float>
                </Floats>
                <Strings>
                    <String>
                        <Name>String 1</Name>
                        <Value>One</Value>
                    </String>
                    <String>
                        <Name>String 2</Name>
                        <Value>Two</Value>
                    </String>
                </Strings>
            </Globals>
        ";

        let globals = Globals::read(xml).unwrap();
        println!("{globals:#?}");

        assert_eq!(
            globals,
            Globals {
                integers: vec![
                    Entry {
                        name: "Int 1".into(),
                        value: 1,
                    },
                    Entry {
                        name: "Int 2".into(),
                        value: 2,
                    },
                ],
                booleans: vec![
                    Entry {
                        name: "Bool 1".into(),
                        value: true,
                    },
                    Entry {
                        name: "Bool 2".into(),
                        value: false,
                    },
                ],
                floats: vec![
                    Entry {
                        name: "Float 1".into(),
                        value: 1.0,
                    },
                    Entry {
                        name: "Float 2".into(),
                        value: 2.0,
                    },
                ],
                strings: vec![
                    Entry {
                        name: "String 1".into(),
                        value: "One".into(),
                    },
                    Entry {
                        name: "String 2".into(),
                        value: "Two".into(),
                    },
                ],
            }
        )
    }
}
