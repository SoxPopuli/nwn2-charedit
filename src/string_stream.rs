use std::io::{Bytes, Read};

fn is_ascii(byte: u8) -> bool {
    (byte & 0b1000_0000) == 0
}

fn starts_with_10(byte: u8) -> bool {
    (byte & 0b1100_0000) == 0b1000_0000
}

fn is_two_bytes(first: u8, second: u8) -> bool {
    (first & 0b1110_0000) == 0b1100_0000 && starts_with_10(second)
}

fn is_three_bytes(first: u8, second: u8, third: u8) -> bool {
    (first & 0b1111_0000) == 0b1110_0000 && starts_with_10(second) && starts_with_10(third)
}

fn is_four_bytes(first: u8, second: u8, third: u8, fourth: u8) -> bool {
    (first & 0b1111_1000) == 0b1111_0000
        && starts_with_10(second)
        && starts_with_10(third)
        && starts_with_10(fourth)
}

pub struct StringStream<T>
where
    T: Read,
{
    bytes: Bytes<T>,
}
impl<T> StringStream<T>
where
    T: Read,
{
    pub fn new(x: T) -> Self {
        Self { bytes: x.bytes() }
    }

    pub fn next_line(&mut self) -> Option<String> {
        let mut buf = String::new();
        while let Some(ch) = self.next() {
            if ch == '\r' {
                let next = self.next();
                if next == Some('\n') {
                    break;
                } else {
                    buf.push(ch);
                }
            } else if ch == '\n' {
                break;
            } else {
                buf.push(ch);
            }
        }

        if !buf.is_empty() {
            Some(buf)
        } else {
            None
        }
    }

    pub fn lines(mut self) -> impl Iterator<Item = String> {
        std::iter::from_fn(move || self.next_line())
    }
}
impl<T> Iterator for StringStream<T>
where
    T: Read,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_byte = || self.bytes.next().and_then(|x| x.ok());

        let first = next_byte()?;
        if is_ascii(first) {
            return char::from_u32(first as u32);
        }

        let second = next_byte()?;
        if is_two_bytes(first, second) {
            let first = first & 0b0001_1111;
            let second = second & 0b0011_1111;
            let ch = (first as u32) << 6 | second as u32;
            return char::from_u32(ch);
        }

        let third = next_byte()?;
        if is_three_bytes(first, second, third) {
            let first = first & 0b0000_1111;
            let second = second & 0b0011_1111;
            let third = third & 0b0011_1111;

            let ch = (first as u32) << 12 | (second as u32) << 6 | third as u32;
            return char::from_u32(ch);
        }

        let fourth = next_byte()?;
        if is_four_bytes(first, second, third, fourth) {
            let first = first & 0b0000_0111;
            let second = second & 0b0011_1111;
            let third = third & 0b0011_1111;
            let fourth = fourth & 0b0011_1111;

            let ch = (first as u32) << 18
                | (second as u32) << 12
                | (third as u32) << 6
                | (fourth as u32);

            return char::from_u32(ch);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use super::StringStream;

    #[test]
    fn one_byte_test() {
        let s = "h";
        let cur = std::io::Cursor::new(s);
        let mut stream = StringStream::new(cur);
        assert_eq!(stream.next(), Some('h'));
    }

    #[test]
    fn two_byte_test() {
        let s = "\u{1B1}";
        let cur = std::io::Cursor::new(s);
        let mut stream = StringStream::new(cur);
        assert_eq!(stream.next(), Some('\u{1B1}'));
    }

    #[test]
    fn three_byte_test() {
        let s = "\u{2713}";
        let cur = std::io::Cursor::new(s);
        let mut stream = StringStream::new(cur);
        assert_eq!(stream.next(), Some('\u{2713}'));
    }

    #[test]
    fn four_byte_test() {
        let s = "😉";
        let cur = std::io::Cursor::new(s);
        let mut stream = StringStream::new(cur);
        assert_eq!(stream.next(), Some('😉'));
    }

    #[test]
    fn next_line_test() {
        let with_trailing = Cursor::new("hi\nhow\r\nare\nyou?\n");
        let without_trailing = Cursor::new("hi\nhow\r\nare\nyou?\n");

        let expected = ["hi", "how", "are", "you?"];

        fn get_lines(data: impl Read) -> Vec<String> {
            let mut stream = StringStream::new(data);
            let mut lines = Vec::new();
            while let Some(l) = stream.next_line() {
                lines.push(l);
            }

            lines
        }

        assert_eq!(get_lines(with_trailing), expected);
        assert_eq!(get_lines(without_trailing), expected);
    }

    #[test]
    fn lines_test() {
        let s = Cursor::new("how\nmuch\nwood\nwould\na\nwoodchuck\nchuck?");
        let stream = StringStream::new(s);
        let lines = stream.lines().collect::<Vec<_>>();

        assert_eq!(
            lines,
            vec!["how", "much", "wood", "would", "a", "woodchuck", "chuck?"]
        )
    }
}
