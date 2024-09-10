// Files to load:
//   - 2DA.zip
//     - Templates.zip
//   - 2DA_X1.zip [Optional: Expansion]
//     - Templates_X1.zip
//   - 2DA_X2.zip [Optional: Expansion]
//     - Templates_X2.zip

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, anychar, line_ending, one_of, space1, tab},
    combinator::{consumed, eof, map_res, not},
    multi::{many0, many1, many_till},
    sequence::terminated,
    IResult,
};

use crate::utils::{BindFirst, BindSecond, MapSecond, Pipe};

fn parse_2da(data: &str) {
    fn skip_whitespace(data: &str) -> IResult<&str, ()> {
        let mut parser = one_of("\n\r\t ").pipe(many0);
        parser(data).map(|(a, _)| (a, ()))
    }

    fn parse_file_header(data: &str) -> IResult<&str, ()> {
        let (data, _) = tag("2DA")(data)?;
        let (data, _) = skip_whitespace(data)?;
        let (data, _) = tag("V2.0")(data)?;
        let (data, _) = skip_whitespace(data)?;

        Ok((data, ()))
    }

    fn parse_table_header(data: &str) -> IResult<&str, Vec<&str>> {
        let (data, mut headers) = many1(terminated(alphanumeric1, many1(one_of("\t "))))(data)?;
        let (data, last_header) = alphanumeric1(data)?;
        let (data, _) = nom::character::complete::line_ending(data)?;
        headers.push(last_header);

        Ok((data, headers))
    }

    fn get_lines(data: &str) -> IResult<&str, Vec<String>> {
        //let separator = many1(one_of("\t "));

        let non_space = {
            let space_or_eof = alt( (tag("\t"), tag(" "), eof ));
            let not_space = many_till(anychar, space_or_eof);

            map_res(not_space, |x| unsafe {
                String::from_utf8(std::mem::transmute(x.0))
            })
        };

        let (data, (columns, _)) = many_till(non_space, alt((line_ending, eof)))(data)?;
        Ok((data, columns))
    }

    let (data, _) = parse_file_header(data).unwrap();
    let (data, columns) = parse_table_header(data).unwrap();
    let (data, lines) = get_lines(data).unwrap();

    println!("{columns:?}");
    println!("{lines:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_2da_test() {
        let file = include_str!("./tests/files/example.2da");
        parse_2da(file);
        panic!();
    }
}
