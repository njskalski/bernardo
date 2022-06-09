use regex::Regex;
use crate::text::buffer::Buffer;

pub enum FindError {
    RegexPatternFail,
    CharToByteFail,
}

/*
This is an iterator of consecutive NON-OVERLAPPING matches, meaning not necessarily ALL matches.
 */
pub struct RegexMatches {
    all_bytes: String,
    regex: Regex,
    byte_pos: usize,
}

/*
This will work with both regexes and simple strings.
 */
pub fn regex_find<'a>(pattern: &'a str, rope: &'a dyn Buffer, start_pos_chars: Option<usize>) -> Result<RegexMatches, FindError> {
    let mut all_bytes = String::new();
    for chunk in rope.chunks() {
        all_bytes += chunk;
    }

    let regex = Regex::new(pattern).map_err(|_| FindError::RegexPatternFail)?;

    let byte_pos: usize = match start_pos_chars {
        None => 0,
        Some(char_pos) => {
            match rope.char_to_byte(char_pos) {
                None => {
                    return Err(FindError::CharToByteFail);
                }
                Some(byte_pos) => byte_pos,
            }
        }
    };

    Ok(RegexMatches {
        all_bytes,
        regex,
        byte_pos,
    })
}

impl<'a> Iterator for RegexMatches {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self.regex.find_at(&self.all_bytes, self.byte_pos) {
            Some(m) => {
                self.byte_pos = m.end();
                Some((m.start(), m.end()))
            }
            None => None
        }
    }
}
