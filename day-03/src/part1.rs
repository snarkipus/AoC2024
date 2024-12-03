use itertools::Itertools;
use miette::{Diagnostic, Result};
use nom::{
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{map, verify},
    sequence::{delimited, pair, separated_pair},
    IResult,
};
use thiserror::Error;

/// Domain model for multiplication operations
#[derive(Debug, Clone, PartialEq)]
struct Multiplication {
    x: i32,
    y: i32,
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(code(parser::error))]
enum ParserError {
    #[error("Failed to parse number")]
    NumberParse,
    #[error("Invalid multiplication format")]
    InvalidFormat,
}

impl Multiplication {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn from_str(s: &str) -> Result<Self, ParserError> {
        let (x, y) = s
            .trim_matches(|p| p == '(' || p == ')')
            .split(',')
            .map(|n| n.parse::<i32>().map_err(|_| ParserError::NumberParse))
            .collect_tuple()
            .ok_or(ParserError::InvalidFormat)?;

        Ok(Self::new(x?, y?))
    }

    fn evaluate(&self) -> i32 {
        self.x * self.y
    }
}

const MAX_NUMBER_LENGTH: usize = 3;

/// Validates input number format
#[tracing::instrument]
fn valid_number(input: &str) -> IResult<&str, &str> {
    verify(digit1, |num: &str| {
        !num.is_empty() && num.len() <= MAX_NUMBER_LENGTH
    })(input)
}

/// Parses a multiplication expression
#[tracing::instrument]
fn mul_expression(input: &str) -> IResult<&str, String> {
    map(
        pair(
            tag("mul"),
            delimited(
                char('('),
                separated_pair(valid_number, char(','), valid_number),
                char(')'),
            ),
        ),
        |(_, (n1, n2))| format!("({},{})", n1, n2),
    )(input)
}

/// Parses all multiplication expressions in input
#[tracing::instrument]
pub fn parse_multiplication(input: &str) -> Result<Vec<String>> {
    let mut results = Vec::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        match mul_expression(remaining) {
            Ok((rest, expr)) => {
                results.push(expr);
                remaining = rest;
            }
            Err(_) => {
                if remaining.len() > 1 {
                    remaining = &remaining[1..];
                } else {
                    break;
                }
            }
        }
    }

    Ok(results)
}

/// Process input string and return sum of multiplications
#[tracing::instrument]
pub fn process(input: &str) -> Result<String> {
    let expressions = parse_multiplication(input)?;

    let result: i32 = expressions
        .iter()
        .map(|expr| Multiplication::from_str(expr))
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .map(Multiplication::evaluate)
        .sum();

    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("mul(2,4)", "8")]
    #[case("xmul(2,4)mul(3,3)", "17")]
    fn test_process_cases(#[case] input: &str, #[case] expected: &str) -> Result<()> {
        assert_eq!(expected, process(input)?);
        Ok(())
    }

    #[test]
    fn test_process() -> Result<()> {
        let input = "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))";
        assert_eq!("161", process(input)?);
        Ok(())
    }

    #[test]
    fn test_instruction_parser() -> Result<()> {
        let input = "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))";
        assert_eq!(
            ["(2,4)", "(5,5)", "(11,8)", "(8,5)"].to_vec(),
            parse_multiplication(input)?
        );
        Ok(())
    }

    #[test]
    fn test_invalid_format() {
        assert!(Multiplication::from_str("invalid").is_err());
    }
}
