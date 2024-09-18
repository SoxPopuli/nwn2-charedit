// Files to load:
//   - 2DA.zip
//     - Templates.zip
//   - 2DA_X1.zip [Optional: Expansion]
//     - Templates_X1.zip
//   - 2DA_X2.zip [Optional: Expansion]
//     - Templates_X2.zip

use std::io::Read;

use crate::string_stream::StringStream;

fn parse_2da(data: impl Read) {
    let stream = StringStream::new(data);

    //fn validate_header<'a, T: Iterator<Item = &'a str>>(line: Option<T>) {
    //    let is_valid =
    //        line
    //            .map(|l|
    //                ["2DA", "V2.0"].into_iter().eq(l)
    //            )
    //            .unwrap_or(false);
    //}

    //stream.lines()
    //.map(|x| x.split_ascii_whitespace()
    //)
    //.filter(|x| x.l)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn parse_2da_test() {
        let file = include_str!("./tests/files/example.2da");
        parse_2da(Cursor::new(file));
        panic!();
    }
}
