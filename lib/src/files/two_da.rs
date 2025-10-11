// Files to load:
//   - 2DA.zip
//     - Templates.zip
//   - 2DA_X1.zip [Optional: Expansion]
//     - Templates_X1.zip
//   - 2DA_X2.zip [Optional: Expansion]
//     - Templates_X2.zip

use std::io::Read;

#[derive(Debug, Default)]
pub struct DataTable {
    pub columns: Vec<String>,
    pub data: Vec2d<Option<String>>,
}
impl DataTable {
    pub fn find_column_index(&self, column: &str) -> Option<usize> {
        self.columns
            .iter()
            .enumerate()
            .find(|(_, x)| x.as_str() == column)
            .map(|(i, _)| i)
    }

    /// Get an iterator over data in column *index*
    /// ---
    /// Returns empty iter if out of bounds
    pub fn get_column_data(&self, index: usize) -> impl Iterator<Item = Option<&str>> {
        let iter = if index < self.columns.len() {
            let mut row = 0;
            Some(std::iter::from_fn(move || {
                let data = self.data.get(index, row).map(|x| x.as_deref());

                row += 1;

                data
            }))
        } else {
            None
        };

        iter.into_iter().flatten()
    }

    pub fn get_row_data(&self, index: usize) -> impl Iterator<Item = Option<&str>> {
        let iter = if index < self.data.height() {
            let mut col = 0;
            Some(std::iter::from_fn(move || {
                let data = self.data.get(col, index).map(|x| x.as_deref());
                col += 1;
                data
            }))
        } else {
            None
        };

        iter.into_iter().flatten()
    }
}

use rust_utils::{string_stream::StringStream, vec2d::Vec2d};

use crate::{
    error::Error::{self, *},
    utils::pair_second,
};

fn validate_header(header: Option<&Vec<String>>) -> Result<(), Error> {
    let header_matches = header
        .as_ref()
        .map(|x| x.iter().eq(["2DA", "V2.0"]))
        .unwrap_or(false);

    if !header_matches {
        let file_header = header.unwrap_or(&vec![]).join(" ");
        return Err(ParseError(format!(
            "File header does not match: Expected: \"2DA V2.0\", Actual: {:?}",
            file_header
        )));
    }

    Ok(())
}

fn split_line_parts(line: &str) -> Vec<String> {
    let mut parts = vec![];
    let mut strbuf = String::new();

    let chars_to_skip = ['\n', '\r', ' ', '\t'];

    macro_rules! push_buf_to_parts {
        () => {{
            if !strbuf.is_empty() {
                parts.push(std::mem::take(&mut strbuf));
            }
        }};
    }

    let mut chars = line.chars();

    while let Some(ch) = chars.next() {
        if ch == '"' {
            for next_char in chars.by_ref() {
                if next_char == '"' {
                    break;
                } else {
                    strbuf.push(next_char)
                }
            }
        } else if chars_to_skip.contains(&ch) {
            push_buf_to_parts!();
        } else {
            strbuf.push(ch);
        }
    }

    push_buf_to_parts!();

    parts
}

pub fn parse(data: impl Read) -> Result<DataTable, Error> {
    let stream = StringStream::new(data);

    let mut lines = stream.lines().map(|x| split_line_parts(&x)).enumerate();

    let file_header = lines.next().map(pair_second);
    validate_header(file_header.as_ref())?;

    // Skip until first non blank line
    let mut lines = lines.skip_while(|(_, line)| line.is_empty());

    let table_header = lines
        .next()
        .map(pair_second)
        .ok_or_else(|| ParseError("Missing table header".to_string()))?;

    let width = table_header.len();
    let mut table = DataTable {
        columns: table_header,
        data: Vec2d::new(width, 0),
    };

    for (line_num, mut l) in lines {
        if l.is_empty() {
            continue;
        }

        let (idx, columns) = l
            .split_first_mut()
            .ok_or_else(|| ParseError(format!("Missing index in line: {line_num}")))?;

        let idx = idx
            .parse::<usize>()
            .map_err(|e| ParseError(e.to_string()))?;

        for (x, item) in columns.iter_mut().enumerate() {
            if item == "****" {
                table.data.insert_at(x, idx, None);
            } else {
                let item = std::mem::take(item);
                table.data.insert_at(x, idx, Some(item));
            }
        }
    }

    Ok(table)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn parse_2da_test() {
        let file = include_str!("./../tests/files/example.2da");
        let table = parse(Cursor::new(file)).unwrap();

        let expect_row = |y, expected: &[&str]| {
            (0..table.data.width()).for_each(|i| {
                assert_eq!(
                    table.data[(i, y)].as_deref(),
                    Some(expected[i]),
                    "Row mismatch: index {y}"
                )
            });
        };

        assert_eq!(table.columns.len(), 4);
        assert_eq!(table.data.width(), 4);
        assert_eq!(table.data.height(), 4);

        expect_row(0, &["TestValue1", "100", "x", "0"]);
        expect_row(1, &["TestValue2", "200", "y", "1"]);
        expect_row(2, &["Test Value 3", "300", "z", "2"]);

        assert_eq!(table.data[(0, 3)], None);
        assert_eq!(table.data[(1, 3)], None);
        assert_eq!(table.data[(2, 3)], None);
        assert_eq!(table.data[(3, 3)], None);
    }
}
