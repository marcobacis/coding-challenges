/*
JSON Parser library
Copied in (most) part from Matthias Kaak implementation (https://github.com/zvavybir/adventjson/tree/master)
*/

use regex::Regex;
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum JsonObject {
    Array(Vec<Self>),
    Object(Vec<(String, Self)>),
    Number(f64),
    JsonString(String),
    Bool(bool),
    Null,
}

#[derive(Clone, Debug, PartialEq, Error)]
pub enum JsonError {
    #[error("input was empty or only whitespace")]
    Empty,
    #[error("At the given position the given character was read invalidly")]
    InvalidChar(char, usize),
    #[error("unterminated string")]
    UnterminatedString,
    #[error("input ended in a backslash without the corresponding escape characters")]
    EndedOnEscape,
    #[error("unknown escape sequence")]
    UnknownEscapeSequence(char),
    #[error("A not string was used as key in an [`JsonObject::Obj`]")]
    NonStringAsKey,
    #[error("invalid code point specified")]
    InvalidCodepoint,
    #[error("invalid number")]
    InvalidNumber,
    #[error("json ended without closing the corresponding array bracket")]
    UnterminatedArray,
    #[error("json ended without closing the corresponding object bracket")]
    UnterminatedObject,
}

impl JsonObject {
    pub fn read(s: &str) -> Result<Self, JsonError> {
        let mut parser = Parser::new(s);
        parser.partial_read(true)
    }

    pub fn read_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        Ok(Self::read(&contents)?)
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
    fn partial_read(&mut self, root: bool) -> Result<JsonObject, JsonError> {
        self.skip_whitespace();
        if let Some(c) = self.peek() {
            if root && c != '{' && c != '[' {
                return Err(JsonError::InvalidChar(self.buf[self.index], self.index));
            }
            let result = match c {
                '{' => self.partial_read_object(),
                '[' => self.partial_read_array(),
                '0'..='9' | '+' | '-' => self.partial_read_number(),
                '"' => self.partial_read_string(),
                'f' => self.partial_read_false(),
                't' => self.partial_read_true(),
                'n' => self.partial_read_null(),
                _ => Err(JsonError::InvalidChar(self.buf[self.index], self.index)),
            };
            if result.is_err() {
                return result;
            }
            if root {
                if let Some(c) = self.peek() {
                    return Err(JsonError::InvalidChar(c, self.index));
                }
            }
            return result;
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
        let mut first_digit = -1;
        let mut digits_size = 0;
        while let Some(c @ '0'..='9') = self.peek() {
            token.push(c);
            if first_digit == -1 {
                first_digit = c.to_digit(10).ok_or(JsonError::InvalidNumber)? as i32;
            }
            digits_size += 1;
            self.next_no_skip();
        }

        // Check for possible leading zeros
        if first_digit == 0 && digits_size > 1 {
            return Err(JsonError::InvalidNumber);
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
            self.next_no_skip();
            // Read exponent sign
            if let Some(c @ ('+' | '-')) = self.peek() {
                token.push(c);
                self.next_no_skip();
            }
            // Read exponent digits
            while let Some(f @ '0'..='9') = self.peek() {
                token.push(f);
                self.next_no_skip();
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
                '\n' | '\r' | '\t' => return Err(JsonError::InvalidChar(c, self.index)),
                _ => {
                    let mut buf = [0u16; 2];
                    utf16.extend_from_slice(c.encode_utf16(&mut buf));
                }
            }
        }

        Err(JsonError::UnterminatedString)
    }

    fn partial_read_array(&mut self) -> Result<JsonObject, JsonError> {
        let mut elements: Vec<JsonObject> = Vec::new();
        self.next_no_skip();
        self.skip_whitespace();
        let mut first_elem = true;
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    if first_elem {
                        return Err(JsonError::InvalidChar(',', self.index));
                    }
                    if let Some(c @ (',' | ']')) = self.next() {
                        return Err(JsonError::InvalidChar(c, self.index));
                    }
                }
                Some(']') => {
                    self.next();
                    return Ok(JsonObject::Array(elements));
                }
                Some(c) => {
                    let elem = self.partial_read(false);
                    match elem {
                        Ok(e) => {
                            elements.push(e);
                            first_elem = false;
                        }
                        Err(err) => return Err(err),
                    }
                }
                None => return Err(JsonError::UnterminatedArray),
            }
        }
    }

    fn partial_read_object(&mut self) -> Result<JsonObject, JsonError> {
        let mut elements: Vec<(String, JsonObject)> = Vec::new();
        self.next();

        loop {
            match self.peek() {
                Some('"') => {
                    // Parse "key": val
                    if let JsonObject::JsonString(key) = self.partial_read_string()? {
                        if let Some(c @ (' ' | '\t' | '\r' | '\n')) = self.peek() {
                            self.next();
                        }
                        if let Some(':') = self.peek() {
                            // Parse element
                            self.next();
                            let element = self.partial_read(false)?;
                            elements.push((key, element));
                        } else {
                            println!("error key1 : \"{:?}\"", self.peek());
                            return Err(JsonError::NonStringAsKey);
                        }
                    } else {
                        return Err(JsonError::NonStringAsKey);
                    }
                }
                Some('}') => {
                    self.next();
                    return Ok(JsonObject::Object(elements));
                }
                Some(',') => {
                    self.next();
                    if let Some('}') = self.peek() {
                        return Err(JsonError::InvalidChar('}', self.index));
                    }
                }
                Some(' ' | '\t' | '\r' | '\n') => {
                    self.index += 1;
                }
                Some(c) => {
                    return Err(JsonError::InvalidChar(c, self.index));
                }
                None => return Err(JsonError::UnterminatedObject),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::JsonError;
    use crate::JsonObject::{self, Array, Bool, JsonString, Null, Number, Object};
    use crate::Parser;

    fn test_read(s: &str) -> Result<JsonObject, JsonError> {
        let mut parser = Parser::new(s);
        parser.partial_read(false)
    }

    #[test]
    fn test_empty() {
        assert_eq!(test_read(""), Err(JsonError::Empty));
        assert_eq!(test_read("    "), Err(JsonError::Empty));
        assert_eq!(test_read("   \n\t \t   "), Err(JsonError::Empty));
    }

    #[test]
    fn test_read_number() {
        assert_eq!(test_read("0").unwrap(), Number(0.0));
        assert_eq!(test_read("0.00").unwrap(), Number(0.0));
        assert_eq!(test_read("10").unwrap(), Number(10.0));
        assert_eq!(test_read("5632").unwrap(), Number(5632.0));
        assert_eq!(test_read("1.2e3").unwrap(), Number(1200.0));
        assert_eq!(test_read("4324.6234").unwrap(), Number(4324.6234));
        assert_eq!(test_read("-4324.6234").unwrap(), Number(-4324.6234));
        assert_eq!(
            test_read("0.123456789e-12").unwrap(),
            Number(0.123456789e-12)
        );
        assert_eq!(test_read("4324. 6234"), Err(JsonError::InvalidNumber));
    }

    #[test]
    fn test_read_fixed_strings() {
        assert_eq!(test_read("false").unwrap(), Bool(false));

        assert_eq!(test_read("fa lse"), Err(JsonError::InvalidChar(' ', 2)));

        assert_eq!(test_read("true").unwrap(), Bool(true));
        assert_eq!(test_read("null").unwrap(), Null);
        assert_eq!(test_read("treadu"), Err(JsonError::InvalidChar('e', 2)));
        assert_eq!(test_read("tru"), Err(JsonError::Empty));
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
            assert_eq!(test_read(input).unwrap(), JsonString(output.to_string()));
        }
    }

    #[test]
    fn test_read_array() {
        assert_eq!(test_read("[]").unwrap(), Array(Vec::new()));
        assert_eq!(
            test_read("[1,2]").unwrap(),
            Array(vec![Number(1.0), Number(2.0)])
        );
        assert_eq!(test_read("[3,]"), Err(JsonError::InvalidChar(']', 3)));
        assert_eq!(test_read("[3, , 3.2]"), Err(JsonError::InvalidChar(',', 4)));
        assert_eq!(
            test_read("[\"ciao\", 5.423]").unwrap(),
            Array(vec![JsonString("ciao".to_string()), Number(5.423)])
        );

        let res = test_read(
            "[\"JSON Test Pattern pass1\", {\"object with 1 member\":[\"array with 1 element\"]}]",
        );
        assert_eq!(res.is_ok(), true);
    }

    #[test]
    fn test_read_object() {
        assert_eq!(test_read("{}").unwrap(), Object(Vec::new()));
        assert_eq!(
            test_read("{\"test\": true}").unwrap(),
            Object(vec![("test".to_string(), Bool(true))])
        );
        assert_eq!(
            test_read("{\"test\": true, \"other\": 42.13, \"testnull\": null}").unwrap(),
            Object(vec![
                ("test".to_string(), Bool(true)),
                ("other".to_string(), Number(42.13)),
                ("testnull".to_string(), Null)
            ])
        );

        assert_eq!(
            test_read("{\"object with 1 member\":[\"array with 1 element\"]}").unwrap(),
            Object(vec![(
                "object with 1 member".to_string(),
                Array(vec![JsonString("array with 1 element".to_string())])
            )])
        )
    }
}
