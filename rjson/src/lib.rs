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
    UnterminatedArray,
}

impl JsonObject {
    pub fn read(s: &str) -> Result<Self, JsonError> {
        let mut parser = Parser::new(s);
        parser.partial_read()
    }
}

struct Parser {
    buf: Vec<char>,
    index: usize,
}

impl Parser {
    fn new(str: &str) -> Self {
        Parser {
            buf: str.chars().collect(),
            index: 0,
        }
    }

    pub fn skip_whitespace(&mut self) {
        while self.buf.len() > self.index && self.buf[self.index].is_whitespace() {
            self.index += 1;
        }
    }

    fn end_reached(&self) -> bool {
        self.index >= self.buf.len()
    }

    fn peek(&self) -> Option<char> {
        if self.end_reached() {
            return None;
        }
        Some(self.buf[self.index])
    }

    fn next(&mut self) -> Option<char> {
        if self.end_reached() {
            return None;
        }
        self.index += 1;
        while let Some(c) = self.peek() {
            if !c.is_whitespace() {
                return Some(c);
            }
            self.index += 1;
        }
        None
    }

    fn next_no_skip(&mut self) -> Option<char> {
        if self.end_reached() {
            return None;
        }
        self.index += 1;
        self.peek()
    }

    /// Reads a json object partially, given its string representation and an index from
    /// where to start reading
    fn partial_read(&mut self) -> Result<JsonObject, JsonError> {
        self.skip_whitespace();
        if let Some(c) = self.peek() {
            match c {
                '[' => self.partial_read_array(),
                '0'..='9' | '+' | '-' => self.partial_read_number(),
                '"' => self.partial_read_string(),
                'f' => self.partial_read_false(),
                't' => self.partial_read_true(),
                'n' => self.partial_read_null(),
                _ => Err(JsonError::InvalidChar(self.buf[self.index], self.index)),
            }
        } else {
            return Err(JsonError::Empty);
        }
    }

    // Reads a jsoon number object from a given index
    fn partial_read_number(&mut self) -> Result<JsonObject, JsonError> {
        let mut token = String::new();

        self.skip_whitespace();

        // Isolate string containing number
        // Read sign
        if let Some(c @ ('+' | '-')) = self.peek() {
            token.push(c);
            self.next_no_skip();
        }

        // Read integer
        while let Some(c @ '0'..='9') = self.peek() {
            token.push(c);
            self.next_no_skip();
        }

        // Read fraction
        if let Some(c @ '.') = self.peek() {
            token.push(c);
            while let Some(f @ '0'..='9') = self.next_no_skip() {
                token.push(f);
            }
        }

        // Read exponent
        if let Some(c @ ('e' | 'E')) = self.peek() {
            token.push(c);
            while let Some(f @ '0'..='9') = self.next_no_skip() {
                token.push(f);
            }
        }

        let regex = Regex::new(r"[+-]?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?$").unwrap();
        if !regex.is_match(&token.as_str()) {
            return Err(JsonError::InvalidNumber);
        }

        let num = f64::from_str(&token);
        match num {
            Ok(num) => Ok(JsonObject::Number(num)),
            Err(_) => Err(JsonError::InvalidNumber),
        }
    }

    fn partial_read_given_string(
        &mut self,
        goal: &str,
        value: JsonObject,
    ) -> Result<JsonObject, JsonError> {
        if self.buf.len() < self.index + goal.len() {
            return Err(JsonError::Empty); // Might have another name for the error
        }

        for g in goal.chars() {
            if let Some(c) = self.peek() {
                if c != g {
                    return Err(JsonError::InvalidChar(c, self.index));
                }
                self.next_no_skip();
            }
        }
        Ok(value)
    }

    fn partial_read_false(&mut self) -> Result<JsonObject, JsonError> {
        self.partial_read_given_string("false", JsonObject::Bool(false))
    }

    fn partial_read_true(&mut self) -> Result<JsonObject, JsonError> {
        self.partial_read_given_string("true", JsonObject::Bool(true))
    }

    fn partial_read_null(&mut self) -> Result<JsonObject, JsonError> {
        self.partial_read_given_string("null", JsonObject::Null)
    }

    fn partial_read_string(&mut self) -> Result<JsonObject, JsonError> {
        self.skip_whitespace();
        let mut utf16: Vec<u16> = Vec::new();

        while let Some(c) = self.next_no_skip() {
            match c {
                '\\' => {
                    if self.end_reached() {
                        return Err(JsonError::EndedOnEscape);
                    }

                    // Manage escape character
                    let mut utf16char = [0u16; 2];
                    match self.next_no_skip() {
                        Some('\\') => utf16.extend_from_slice('\\'.encode_utf16(&mut utf16char)),
                        Some('/') => utf16.extend_from_slice('/'.encode_utf16(&mut utf16char)),
                        Some('"') => utf16.extend_from_slice('"'.encode_utf16(&mut utf16char)),
                        Some('b') => {
                            utf16.extend_from_slice('\u{0008}'.encode_utf16(&mut utf16char))
                        }
                        Some('f') => {
                            utf16.extend_from_slice('\u{000c}'.encode_utf16(&mut utf16char))
                        }
                        Some('n') => utf16.extend_from_slice('\n'.encode_utf16(&mut utf16char)),
                        Some('r') => utf16.extend_from_slice('\r'.encode_utf16(&mut utf16char)),
                        Some('t') => utf16.extend_from_slice('\t'.encode_utf16(&mut utf16char)),
                        //Manage unicode coodes \uXXXX
                        Some('u') => {
                            let mut u = 0u16;
                            for _ in 0..4 {
                                match self.next_no_skip() {
                                    Some(c) => {
                                        if let Some(h) = c.to_digit(16) {
                                            u = u * 0x10 + h as u16;
                                        } else {
                                            return Err(JsonError::InvalidCodepoint);
                                        }
                                    }
                                    None => return Err(JsonError::InvalidCodepoint),
                                }
                            }
                            utf16.push(u);
                        }
                        Some(c) => return Err(JsonError::InvalidChar(c, self.index)),
                        None => return Err(JsonError::Empty),
                    };
                }
                '"' => {
                    self.next_no_skip();
                    return Ok(JsonObject::JsonString(String::from_utf16(&utf16).unwrap()));
                }
                _ => {
                    let mut buf = [0u16; 2];
                    utf16.extend_from_slice(c.encode_utf16(&mut buf));
                }
            }
        }

        println!("Shouldn't go there");

        Err(JsonError::UnterminatedString)
    }

    fn partial_read_array(&mut self) -> Result<JsonObject, JsonError> {
        let mut elements: Vec<JsonObject> = Vec::new();
        self.next_no_skip();
        self.skip_whitespace();
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    if let Some(c @ (',' | ']')) = self.next() {
                        return Err(JsonError::InvalidChar(c, self.index));
                    }
                }
                Some(']') => return Ok(JsonObject::Array(elements)),
                Some(c) => {
                    let elem = self.partial_read();
                    match elem {
                        Ok(e) => elements.push(e),
                        Err(err) => return Err(err),
                    }
                }
                None => return Err(JsonError::UnterminatedArray),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::JsonError;
    use crate::JsonObject::{self, Array, Bool, JsonString, Null, Number};

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

    #[test]
    fn test_read_array() {
        assert_eq!(JsonObject::read("[]").unwrap(), Array(Vec::new()));
        assert_eq!(
            JsonObject::read("[1,2]").unwrap(),
            Array(vec![Number(1.0), Number(2.0)])
        );
        assert_eq!(
            JsonObject::read("[3,]"),
            Err(JsonError::InvalidChar(']', 3))
        );
        assert_eq!(
            JsonObject::read("[3, , 3.2]"),
            Err(JsonError::InvalidChar(',', 4))
        );
        assert_eq!(
            JsonObject::read("[\"ciao\", 5.423]").unwrap(),
            Array(vec![JsonString("ciao".to_string()), Number(5.423)])
        );
    }
}
