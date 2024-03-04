use field::Field;

pub mod field;

#[derive(Debug)]
pub enum Command {
    Fields,
    Bytes,
    Chars,
}

#[derive(Debug)]
pub struct Cut {
    pub ranges: Vec<Field>,
    pub delimiter: char,
    pub whitespace: bool,
    pub suppress: bool,
    pub command: Command,
}

impl Cut {
    pub fn default() -> Self {
        Cut {
            ranges: vec![],
            delimiter: '\t',
            whitespace: false,
            suppress: false,
            command: Command::Fields,
        }
    }

    pub fn execute_line(&self, input: &str) -> Option<String> {
        match self.command {
            Command::Fields => self.execute_str(input),
            Command::Bytes => self.execute_bytes(input),
            Command::Chars => self.execute_chars(input),
        }
    }

    fn execute_str(&self, input: &str) -> Option<String> {
        let split: Vec<&str> = match self.whitespace {
            true => input.split_whitespace().collect(),
            false => input.split(self.delimiter).collect(),
        };
        if split.len() <= 1 && self.suppress {
            return None;
        }

        let outputs = self.execute(split);
        Some(outputs.join(&self.delimiter.to_string()))
    }

    fn execute_bytes(&self, input: &str) -> Option<String> {
        let outputs = self.execute(input.bytes().collect());
        String::from_utf8(outputs).ok()
    }

    fn execute_chars(&self, input: &str) -> Option<String> {
        Some(self.execute(input.chars().collect()).iter().collect())
    }

    fn execute<T>(&self, splits: Vec<T>) -> Vec<T>
    where
        T: Copy,
    {
        let ranges = self.compute_ranges(splits.len());
        ranges
            .iter()
            .filter_map(|n| {
                if n <= &splits.len() {
                    Some(splits[n - 1])
                } else {
                    None
                }
            })
            .collect()
    }

    fn compute_ranges(&self, max_splits: usize) -> Vec<usize> {
        let mut fields: Vec<usize> = vec![];
        for f in self.ranges.iter() {
            fields.extend(f.values(max_splits));
        }
        fields.sort();
        fields
    }
}

#[cfg(test)]
mod tests {
    use crate::{field::Field, Command, Cut};

    #[test]
    fn simple_single_field() {
        let cut = Cut {
            ranges: vec![Field::single(2)],
            ..Cut::default()
        };
        let result = cut.execute_line("test\texpected\tfield");
        assert_eq!("expected", result.unwrap());
    }

    #[test]
    fn simple_custom_delimiter() {
        let cut = Cut {
            ranges: vec![Field::single(2)],
            delimiter: ',',
            ..Cut::default()
        };
        let result = cut.execute_line("test,expected,field");
        assert_eq!("expected", result.unwrap());
    }

    #[test]
    fn multiple_fields() {
        let cut = Cut {
            ranges: vec![Field::single(2), Field::single(1)],
            delimiter: ',',
            ..Cut::default()
        };
        let result = cut.execute_line("test,expected,field");
        assert_eq!("test,expected", result.unwrap());
    }

    #[test]
    fn whitespace() {
        let cut = Cut {
            ranges: vec![Field::single(1), Field::single(2)],
            whitespace: true,
            ..Cut::default()
        };
        let result = cut.execute_line("test\texpected  field");
        assert_eq!("test\texpected", result.unwrap());
    }

    #[test]
    fn ranges() {
        let cut = Cut {
            ranges: vec![Field::from(9), Field::to(2), Field::range(5, 7)],
            delimiter: ',',
            suppress: true,
            ..Cut::default()
        };
        let result = cut.execute_line("no delimiter in sight");
        assert!(result.is_none());
    }

    #[test]
    fn bytes() {
        let cut = Cut {
            ranges: vec![Field::from(6)],
            command: Command::Bytes,
            ..Cut::default()
        };
        let result = cut.execute_line("test with bytes");
        assert_eq!("with bytes", result.unwrap());
    }

    #[test]
    fn chars() {
        let cut = Cut {
            ranges: vec![Field::range(3, 6)],
            command: Command::Chars,
            ..Cut::default()
        };
        let result = cut.execute_line("test with chars");
        assert_eq!("st w", result.unwrap());
    }
}
