use std::fmt;

use regex::Regex;

#[derive(Debug, PartialEq, Eq)]
pub struct Field {
    pub from: Option<usize>,
    pub to: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ParseFieldError;
impl fmt::Display for ParseFieldError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "illegal list value")
    }
}

impl Field {
    pub fn parse(input: &str) -> Result<Self, ParseFieldError> {
        let field_regex = Regex::new(r"(?P<from>\d+)?(?P<range>-)?(?P<to>\d+)?").unwrap();

        match field_regex.captures(input) {
            None => Err(ParseFieldError {}),
            Some(caps) => {
                if caps.name("range").is_none() {
                    Ok(Field::single(input.parse::<usize>().unwrap()))
                } else {
                    Ok(Field {
                        from: match caps.name("from") {
                            Some(m) => m.as_str().parse::<usize>().ok(),
                            None => None,
                        },
                        to: match caps.name("to") {
                            Some(m) => m.as_str().parse::<usize>().ok(),
                            None => None,
                        },
                    })
                }
            }
        }
    }

    pub fn values(&self, max_fields: usize) -> Vec<usize> {
        let from = match self.from {
            Some(from) => from,
            None => 1,
        };

        let to = match self.to {
            Some(to) => to,
            None => max_fields,
        };

        (from..=to).collect()
    }

    pub fn is_from_start(&self) -> bool {
        self.from.is_none()
    }

    pub fn is_till_end(&self) -> bool {
        self.to.is_none()
    }

    pub fn single(field: usize) -> Self {
        Field {
            from: Some(field),
            to: Some(field),
        }
    }

    pub fn from(field: usize) -> Self {
        Field {
            from: Some(field),
            to: None,
        }
    }

    pub fn to(field: usize) -> Self {
        Field {
            from: None,
            to: Some(field),
        }
    }

    pub fn range(from: usize, to: usize) -> Self {
        Field {
            from: Some(from),
            to: Some(to),
        }
    }
}

pub fn parse_ranges(list: &str) -> Vec<Field> {
    let split_regex = Regex::new(r"(\s+)|,").unwrap();
    let split = split_regex.split(list);
    split.filter_map(|s| Field::parse(s).ok()).collect()
}

#[cfg(test)]
mod tests {
    use crate::{field::parse_ranges, field::Field};

    #[test]
    fn single_field() {
        let actual = parse_ranges("1");
        assert_eq!(vec![Field::single(1)], actual);
    }

    #[test]
    fn multiple_fields() {
        let actual = parse_ranges("1,3");
        assert_eq!(vec![Field::single(1), Field::single(3)], actual);
    }

    #[test]
    fn multiple_fields_whitespace() {
        let actual = parse_ranges("1,3 4\t  5");
        assert_eq!(
            vec![
                Field::single(1),
                Field::single(3),
                Field::single(4),
                Field::single(5)
            ],
            actual
        );
    }

    #[test]
    fn ranges() {
        let actual = parse_ranges("1-4");
        assert_eq!(vec![Field::range(1, 4)], actual);

        let actual = parse_ranges("-4");
        assert_eq!(
            vec![Field {
                from: None,
                to: Some(4)
            }],
            actual
        );

        let actual = parse_ranges("1-");
        assert_eq!(
            vec![Field {
                from: Some(1),
                to: None
            }],
            actual
        );
    }
}
