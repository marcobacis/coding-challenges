/*
JSON Parser library
Copied in (most) part from Matthias Kaak implementation (https://github.com/zvavybir/adventjson/tree/master)
*/

use regex::Regex;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum JsonObject {
    Array(Vec<Self>),
    Obj(Vec<(String, Self)>),
    Number(f64),
    JsonString(String),
    Bool(bool),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JsonError {
    /// The input was empty (or only whitespace)
    Empty,
    /// At the given position the given character was read invalidly
    InvalidChar(char, usize),
    /// A string with no closing quote
    UnterminatedString,
    /// The input ended on a backslash
    EndedOnEscape,
    /// An unknown escape sequence was encountered
    UnknownEscapeSequence(char),
    /// A not string was used as key in an [`JsonObject::Obj`]
    NonStringAsKey,
    /// Per '\uXXXX' was an invalid code point specified
    InvalidCodepoint,
    /// A number that is invalid in json, but is a perfectly fine
    /// floating-point number.
    InvalidNumber,
}

impl JsonObject {
    pub fn read(s: &str) -> Result<Self, JsonError> {
        Self::partial_read(&s.chars().collect::<Vec<_>>(), 0).map(|(x, _)| x)
    }

    /// Reads a json object partially, given its string representation and an index from
    /// where to start reading
    fn partial_read(s: &[char], index: usize) -> Result<(Self, usize), JsonError> {
        let mut newindex = index;
        newindex = Self::skip_whitespace(s, newindex);

        if newindex >= s.len() {
            return Err(JsonError::Empty);
        }

        match s[newindex] {
            '0'..='9' | '+' | '-' => Self::partial_read_number(s, newindex),
            '"' => Self::partial_read_string(s, newindex),
            'f' => Self::partial_read_false(s, newindex),
            't' => Self::partial_read_true(s, newindex),
            'n' => Self::partial_read_null(s, newindex),
            _ => Err(JsonError::InvalidChar(s[newindex], newindex)),
        }
    }

    fn skip_whitespace(s: &[char], index: usize) -> usize {
        let mut newindex = index;
        while s.len() > newindex && s[newindex].is_whitespace() {
            newindex += 1;
        }
        newindex
    }

    // Reads a jsoon number object from a given index
    fn partial_read_number(s: &[char], index: usize) -> Result<(Self, usize), JsonError> {
        let startindex = index;
        let mut newindex = index;
        newindex = Self::skip_whitespace(s, newindex);

        // Isolate string containing number
        // Read sign
        if s.len() > newindex && (s[newindex] == '+' || s[newindex] == '-') {
            newindex += 1;
        }

        // Read integer
        while s.len() > newindex && s[newindex].is_digit(10) {
            newindex += 1;
        }

        // Read fraction
        if s.len() > newindex && s[newindex] == '.' {
            newindex += 1;
            while s.len() > newindex && s[newindex].is_digit(10) {
                newindex += 1;
            }
        }

        // Read exponent
        if s.len() > newindex && s[newindex].to_ascii_lowercase() == 'e' {
            newindex += 1;
            while s.len() > newindex && s[newindex].is_digit(10) {
                newindex += 1;
            }
        }
        let token = &s[startindex..newindex].iter().collect::<String>();

        let regex = Regex::new(r"[+-]?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?$").unwrap();
        if !regex.is_match(token) {
            return Err(JsonError::InvalidNumber);
        }

        let num = f64::from_str(token);
        match num {
            Ok(num) => Ok((Self::Number(num), newindex)),
            Err(_) => Err(JsonError::InvalidNumber),
        }
    }

    fn partial_read_given_string(
        s: &[char],
        index: usize,
        goal: &str,
        value: Self,
    ) -> Result<(Self, usize), JsonError> {
        if s.len() < index + goal.len() {
            return Err(JsonError::Empty); // Might have another name for the error
        }

        let mut newindex = index;
        let word = goal.chars().collect::<Vec<_>>();
        let mut i = 0;
        while s.len() > newindex && i < word.len() && s[newindex] == word[i] {
            newindex += 1;
            i += 1;
        }

        if i == word.len() {
            Ok((value, newindex))
        } else {
            Err(JsonError::InvalidChar(s[newindex], newindex))
        }
    }

    fn partial_read_false(s: &[char], index: usize) -> Result<(Self, usize), JsonError> {
        Self::partial_read_given_string(s, index, "false", Self::Bool(false))
    }

    fn partial_read_true(s: &[char], index: usize) -> Result<(Self, usize), JsonError> {
        Self::partial_read_given_string(s, index, "true", Self::Bool(true))
    }

    fn partial_read_null(s: &[char], index: usize) -> Result<(Self, usize), JsonError> {
        Self::partial_read_given_string(s, index, "null", Self::Null)
    }

    fn partial_read_string(s: &[char], index: usize) -> Result<(Self, usize), JsonError> {
        let mut newindex = index;
        newindex = Self::skip_whitespace(s, newindex);
        newindex += 1;
        let startindex = newindex;

        let mut str = String::new();
        let mut utf16: Vec<u16> = Vec::new();

        while s.len() > newindex && s[newindex] != '"' {
            if s[newindex] == '\\' {
                newindex += 1;
                if s.len() <= newindex {
                    return Err(JsonError::EndedOnEscape);
                }

                // Manage escape character
                let mut utf16char = [0u16; 2];
                match s[newindex] {
                    '\\' => utf16.extend_from_slice('\\'.encode_utf16(&mut utf16char)),
                    '/' => utf16.extend_from_slice('/'.encode_utf16(&mut utf16char)),
                    '"' => utf16.extend_from_slice('"'.encode_utf16(&mut utf16char)),
                    'b' => utf16.extend_from_slice('\u{0008}'.encode_utf16(&mut utf16char)),
                    'f' => utf16.extend_from_slice('\u{000c}'.encode_utf16(&mut utf16char)),
                    'n' => utf16.extend_from_slice('\n'.encode_utf16(&mut utf16char)),
                    'r' => utf16.extend_from_slice('\r'.encode_utf16(&mut utf16char)),
                    't' => utf16.extend_from_slice('\t'.encode_utf16(&mut utf16char)),
                    // Manage unicode coodes \uXXXX
                    'u' => {
                        let mut u = 0u16;
                        for _ in 0..4 {
                            newindex += 1;
                            if s.len() <= newindex {
                                return Err(JsonError::InvalidCodepoint);
                            }
                            if let Some(h) = s[newindex].to_digit(16) {
                                u = u * 0x10 + h as u16;
                            } else {
                                return Err(JsonError::InvalidCodepoint);
                            }
                        }
                        utf16.push(u);
                    }
                    invalid_char => return Err(JsonError::InvalidChar(invalid_char, newindex)),
                };
            } else {
                let mut buf = [0u16; 2];
                utf16.extend_from_slice(s[newindex].encode_utf16(&mut buf));
            }
            newindex += 1;
        }
        if s[newindex] != '"' {
            return Err(JsonError::UnterminatedString);
        }

        let endindex = newindex;
        Ok((
            Self::JsonString(String::from_utf16(&utf16).unwrap()),
            newindex + 1,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::JsonError;
    use crate::JsonObject::{self, Bool, JsonString, Null, Number};

    #[test]
    fn test_empty() {
        assert_eq!(JsonObject::read(""), Err(JsonError::Empty));
        assert_eq!(JsonObject::read("    "), Err(JsonError::Empty));
        assert_eq!(JsonObject::read("   \n\t \t   "), Err(JsonError::Empty));
    }

    #[test]
    fn test_read_number() {
        assert_eq!(JsonObject::read("0").unwrap(), Number(0.0));
        assert_eq!(JsonObject::read("0.00").unwrap(), Number(0.0));
        assert_eq!(JsonObject::read("10").unwrap(), Number(10.0));
        assert_eq!(JsonObject::read("5632").unwrap(), Number(5632.0));
        assert_eq!(JsonObject::read("1.2e3").unwrap(), Number(1200.0));
        assert_eq!(JsonObject::read("4324.6234").unwrap(), Number(4324.6234));
        assert_eq!(JsonObject::read("-4324.6234").unwrap(), Number(-4324.6234));
        assert_eq!(
            JsonObject::read("4324. 6234"),
            Err(JsonError::InvalidNumber)
        );
    }

    #[test]
    fn test_read_fixed_strings() {
        assert_eq!(JsonObject::read("false").unwrap(), Bool(false));

        assert_eq!(
            JsonObject::read("fa lse"),
            Err(JsonError::InvalidChar(' ', 2))
        );

        assert_eq!(JsonObject::read("true").unwrap(), Bool(true));
        assert_eq!(JsonObject::read("null").unwrap(), Null);
        assert_eq!(
            JsonObject::read("treadu"),
            Err(JsonError::InvalidChar('e', 2))
        );
        assert_eq!(JsonObject::read("tru"), Err(JsonError::Empty));
    }

    #[test]
    fn test_read_string() {
        let tests = vec![
            ("\"Hello World\"", "Hello World"),
            ("  \"Hello World\"  ", "Hello World"),
            ("\"Hello \\\\ \\/\\n Wo\\\\rld\"", "Hello \\ /\n Wo\\rld"),
            ("\"deF \\\\ Abc\"", "deF \\ Abc"),
            ("\"deF2 \\\\ 3Abc\"", "deF2 \\ 3Abc"),
            ("\"\\n\"", "\n"),
            ("\"Json\"", "Json"),
            ("\"Json\"", "Json"),
            ("\"ä\"", "ä"),
            ("\"\\u00e4\"", "ä"),
            ("\"𝄞\"", "𝄞"),
            ("\"\\uD834\\uDD1E\"", "𝄞"),
        ];

        for (input, output) in tests {
            assert_eq!(
                JsonObject::read(input).unwrap(),
                JsonString(output.to_string())
            );
        }
    }
}
