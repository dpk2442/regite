#![allow(dead_code)]

use crate::config::Output;

use regex::Regex;

pub struct Parser {
    prefix: String,
    regex: Regex,
    outputs: Vec<Output>,
}

impl Parser {
    pub fn new(prefix: &str, regex: &str, outputs: &[Output]) -> Parser {
        Parser {
            prefix: prefix.to_owned(),
            regex: Regex::new(&format!("(?m){}", regex)).unwrap(),
            outputs: outputs.to_owned(),
        }
    }

    pub fn parse(&self, input: &str) -> Vec<(String, String)> {
        let mut results = vec![];

        for mat in self.regex.find_iter(input) {
            for output in &self.outputs {
                let substring = mat.as_str();
                results.push((
                    format!(
                        "{}.{}",
                        self.prefix,
                        self.regex.replace(substring, &output.name)
                    ),
                    self.regex.replace(substring, &output.value).to_string(),
                ));
            }
        }

        results
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple() {
        let parser = Parser::new(
            "prefix",
            "(.*)",
            &[Output {
                name: "name".to_string(),
                value: "$1".to_string(),
            }],
        );

        assert_eq!(
            parser.parse("input"),
            [("prefix.name".to_string(), "input".to_string())]
        );
    }

    #[test]
    fn test_multiple_outputs() {
        let parser = Parser::new(
            "prefix",
            "(\\d+);(\\d+)",
            &[
                Output {
                    name: "left".to_string(),
                    value: "$1".to_string(),
                },
                Output {
                    name: "right".to_string(),
                    value: "$2".to_string(),
                },
            ],
        );

        assert_eq!(
            parser.parse("1;2"),
            [
                ("prefix.left".to_string(), "1".to_string()),
                ("prefix.right".to_string(), "2".to_string())
            ]
        );
    }

    #[test]
    fn test_multiple_lines() {
        let parser = Parser::new(
            "prefix",
            "^(.*)$",
            &[Output {
                name: "line".to_string(),
                value: "$1".to_string(),
            }],
        );

        assert_eq!(
            parser.parse("line1\nline2"),
            [
                ("prefix.line".to_string(), "line1".to_string()),
                ("prefix.line".to_string(), "line2".to_string())
            ]
        );
    }

    #[test]
    fn test_multiple_lines_multiple_outputs() {
        let parser = Parser::new(
            "prefix",
            "^(\\w+) (\\d+);(\\d+)$",
            &[
                Output {
                    name: "$1.left".to_string(),
                    value: "$2".to_string(),
                },
                Output {
                    name: "$1.right".to_string(),
                    value: "$3".to_string(),
                },
            ],
        );

        assert_eq!(
            parser.parse("line1 1;2\nline2 4;3"),
            [
                ("prefix.line1.left".to_string(), "1".to_string()),
                ("prefix.line1.right".to_string(), "2".to_string()),
                ("prefix.line2.left".to_string(), "4".to_string()),
                ("prefix.line2.right".to_string(), "3".to_string())
            ]
        );
    }
}
