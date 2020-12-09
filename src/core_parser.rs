use crate::errors::ParseError;
use crate::parsing_utils::{decode_str, decode_utf16, decode_windows1252, le_i32};

#[derive(Debug, Clone, PartialEq)]
pub struct CoreParser<'a> {
    data: &'a [u8],

    /// Current offset in regards to the whole view of the replay
    col: i32,
}

impl<'a> CoreParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        CoreParser { data, col: 0 }
    }

    pub fn bytes_read(&self) -> i32 {
        self.col
    }

    /// Used for skipping some amount of data
    pub fn advance(&mut self, ind: usize) {
        self.col += ind as i32;
        self.data = &self.data[ind..];
    }

    /// Returns a slice of the replay after ensuring there is enough space for the requested slice
    pub fn view_data(&self, size: usize) -> Result<&'a [u8], ParseError> {
        if size > self.data.len() {
            Err(ParseError::InsufficientData(
                size as i32,
                self.data.len() as i32,
            ))
        } else {
            Ok(&self.data[..size])
        }
    }

    pub fn take_data(&mut self, size: usize) -> Result<&'a [u8], ParseError> {
        let res = self.view_data(size)?;
        self.advance(size);
        Ok(res)
    }

    /// Take the next `size` of bytes and interpret them in an infallible fashion
    #[inline]
    pub fn take<F, T>(&mut self, size: usize, mut f: F) -> Result<T, ParseError>
    where
        F: FnMut(&'a [u8]) -> T,
    {
        let res = f(self.view_data(size)?);
        self.advance(size);
        Ok(res)
    }

    pub fn skip(&mut self, size: usize) -> Result<(), ParseError> {
        self.take(size, |_| ())
    }

    pub fn take_i32(&mut self, section: &'static str) -> Result<i32, ParseError> {
        self.take(4, le_i32)
            .map_err(|e| ParseError::ParseError(section, self.bytes_read(), Box::new(e)))
    }

    pub fn take_u32(&mut self, section: &'static str) -> Result<u32, ParseError> {
        self.take(4, le_i32)
            .map(|x| x as u32)
            .map_err(|e| ParseError::ParseError(section, self.bytes_read(), Box::new(e)))
    }

    /// Repeatedly parse the same elements from replay until `size` elements parsed
    pub fn repeat<F, T>(size: usize, mut f: F) -> Result<Vec<T>, ParseError>
    where
        F: FnMut() -> Result<T, ParseError>,
    {
        if size > 25_000 {
            return Err(ParseError::ListTooLarge(size));
        }

        let mut res = Vec::with_capacity(size);
        for _ in 0..size {
            res.push(f()?);
        }
        Ok(res)
    }

    pub fn list_of<F, T>(&mut self, mut f: F) -> Result<Vec<T>, ParseError>
    where
        F: FnMut(&mut Self) -> Result<T, ParseError>,
    {
        let size = self.take(4, le_i32)?;
        CoreParser::repeat(size as usize, || f(self))
    }

    pub fn text_list(&mut self) -> Result<Vec<String>, ParseError> {
        self.list_of(CoreParser::parse_text)
    }

    /// Parses UTF-8 string from replay
    pub fn parse_str(&mut self) -> Result<&'a str, ParseError> {
        let mut size = self.take(4, le_i32)? as usize;

        // Replay 6688 has a property name that is listed as having a length of 0x5000000, but it's
        // really the `\0\0\0None` property. I'm guess at some point in Rocket League, this was a
        // bug that was fixed. What's interesting is that I couldn't find this constant in
        // `RocketLeagueReplayParser`, only rattletrap.
        if size == 0x0500_0000 {
            size = 8;
        }
        self.take_data(size).and_then(decode_str)
    }

    /// Parses either UTF-16 or Windows-1252 encoded strings
    pub fn parse_text(&mut self) -> Result<String, ParseError> {
        // The number of bytes that the string is composed of. If negative, the string is UTF-16,
        // else the string is windows 1252 encoded.
        let characters = self.take(4, le_i32)?;

        // size.abs() will panic at min_value, so we eschew it for manual checking
        if characters == 0 {
            Err(ParseError::ZeroSize)
        } else if characters > 10_000 || characters < -10_000 {
            Err(ParseError::TextTooLarge(characters))
        } else if characters < 0 {
            // We're dealing with UTF-16 and each character is two bytes, we
            // multiply the size by 2. The last two bytes included in the count are
            // null terminators
            let size = characters * -2;
            self.take_data(size as usize).and_then(decode_utf16)
        } else {
            self.take_data(characters as usize)
                .and_then(decode_windows1252)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ParseError;

    #[test]
    fn parse_text_encoding() {
        // dd skip=16 count=28 if=rumble.replay of=text.replay bs=1
        let data = include_bytes!("../assets/replays/partial/text.replay");
        let mut parser = CoreParser::new(&data[..]);
        assert_eq!(parser.parse_str().unwrap(), "TAGame.Replay_Soccar_TA");
    }

    #[test]
    fn parse_text_encoding_bad() {
        // dd skip=16 count=28 if=rumble.replay of=text.replay bs=1
        let data = include_bytes!("../assets/replays/partial/text.replay");
        let mut parser = CoreParser::new(&data[..data.len() - 1]);
        let res = parser.parse_str();
        assert!(res.is_err());
        let error = res.unwrap_err();
        assert_eq!(error, ParseError::InsufficientData(24, 23));
    }

    #[test]
    fn parse_text_zero_size() {
        let mut parser = CoreParser::new(&[0, 0, 0, 0, 0]);
        let res = parser.parse_str();
        assert!(res.is_err());
        let error = res.unwrap_err();
        assert_eq!(error, ParseError::ZeroSize);
    }

    #[test]
    fn parse_text_zero_size2() {
        let mut parser = CoreParser::new(&[0, 0, 0, 0, 0]);
        let res = parser.parse_text();
        assert!(res.is_err());
        let error = res.unwrap_err();
        assert_eq!(error, ParseError::ZeroSize);
    }

    #[test]
    fn parse_text_too_large() {
        let mut parser = CoreParser::new(&[0xcc, 0xcc, 0xcc, 0xcc, 0xcc]);
        let res = parser.parse_text();
        assert!(res.is_err());
        let error = res.unwrap_err();
        assert_eq!(error, ParseError::TextTooLarge(-858993460));
    }

    #[test]
    fn parse_text_encoding_bad_2() {
        // Test for when there is not enough data to decode text length
        // dd skip=16 count=28 if=rumble.replay of=text.replay bs=1
        let data = include_bytes!("../assets/replays/partial/text.replay");
        let mut parser = CoreParser::new(&data[..2]);
        let res = parser.parse_str();
        assert!(res.is_err());
        let error = res.unwrap_err();
        assert_eq!(error, ParseError::InsufficientData(4, 2));
    }

    #[test]
    fn parse_utf16_string() {
        // dd skip=((0x120)) count=28 if=utf-16.replay of=utf-16-text.replay bs=1
        let data = include_bytes!("../assets/replays/partial/utf-16-text.replay");
        let mut parser = CoreParser::new(&data[..]);
        let res = parser.parse_text().unwrap();
        assert_eq!(res, "\u{2623}D[e]!v1zz\u{2623}");
    }

    #[test]
    fn test_windows1252_string() {
        let data = include_bytes!("../assets/replays/partial/windows_1252.replay");
        let mut parser = CoreParser::new(&data[0x1ad..0x1c4]);
        let res = parser.parse_text().unwrap();
        assert_eq!(res, "caudillman6000\u{b3}(2)");
    }

    /// Define behavior on invalid UTF-16 sequences.
    #[test]
    fn parse_invalid_utf16_string() {
        let data = [0xfd, 0xff, 0xff, 0xff, 0xd8, 0xd8, 0x00, 0x00, 0x00, 0x00];
        let mut parser = CoreParser::new(&data[..]);
        let res = parser.parse_text().unwrap();
        assert_eq!(res, "�\u{0}");
    }
}
