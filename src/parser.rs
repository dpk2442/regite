#![allow(dead_code)]

use crate::config::Output;

use regex::Regex;

pub struct Parser {
    prefix: String,
    regex: Regex,
    outputs: Vec<Output>,
}

impl Parser {
    fn new(prefix: &str, regex: &str, outputs: &[Output]) -> Parser {
        Parser {
            prefix: prefix.to_owned(),
            regex: Regex::new(&format!("(?m){}", regex)).unwrap(),
            outputs: outputs.to_owned(),
        }
    }

    fn parse(&self, input: &str) -> Vec<String> {
        let mut results = vec![];

        for mat in self.regex.find_iter(input) {
            for output in &self.outputs {
                let substring = mat.as_str();
                results.push(format!(
                    "{}.{} {}",
                    self.prefix,
                    self.regex.replace(substring, &output.name),
                    self.regex.replace(substring, &output.value),
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

        assert_eq!(parser.parse("input"), ["prefix.name input"]);
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

        assert_eq!(parser.parse("1;2"), ["prefix.left 1", "prefix.right 2"]);
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
            ["prefix.line line1", "prefix.line line2"]
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
                "prefix.line1.left 1",
                "prefix.line1.right 2",
                "prefix.line2.left 4",
                "prefix.line2.right 3"
            ]
        );
    }
}
