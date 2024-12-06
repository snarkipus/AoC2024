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

const MAX_NUMBER_LENGTH: usize = 3;

/// Represents a multiplication operation with two operands
#[derive(Debug, Clone, PartialEq)]
struct Multiplication {
    x: i32,
    y: i32,
}

/// Errors that can occur during parsing
#[derive(Debug, Error, Diagnostic)]
#[diagnostic(code(parser::error))]
enum ParserError {
    #[error("Failed to parse number")]
    NumberParse,
    #[error("Invalid multiplication format")]
    InvalidFormat,
}

/// Parser state to track whether to process next multiplication
#[derive(Debug)]
struct ParserState {
    process_next: bool,
}

impl ParserState {
    fn new() -> Self {
        Self { process_next: true }
    }
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

/// Validates that a number is within length constraints
#[tracing::instrument]
fn valid_number(input: &str) -> IResult<&str, &str> {
    verify(digit1, |num: &str| num.len() <= MAX_NUMBER_LENGTH)(input)
}

/// Parses a multiplication expression in the format mul(x,y)
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

/// Parses and processes a sequence of multiplication operations
#[tracing::instrument]
fn parse_multiplication(input: &str) -> Result<Vec<String>> {
    let mut stack = Vec::new();
    let mut remaining = input;
    let mut state = ParserState::new();

    while !remaining.is_empty() {
        if let Ok((rest, _)) = tag::<&str, &str, nom::error::Error<&str>>("don't()")(remaining) {
            state.process_next = false;
            remaining = rest;
            continue;
        }

        if let Ok((rest, _)) = tag::<&str, &str, nom::error::Error<&str>>("do()")(remaining) {
            state.process_next = true;
            remaining = rest;
            continue;
        }

        if let Ok((rest, mul)) = mul_expression(remaining) {
            if state.process_next {
                stack.push(mul);
            }
            remaining = rest;
        } else {
            remaining = &remaining[1..];
        }
    }

    Ok(stack)
}

/// Processes input string and returns sum of valid multiplication operations
#[tracing::instrument]
pub fn process(input: &str) -> Result<String> {
    let result: i32 = parse_multiplication(input)?
        .iter()
        .map(|s| Multiplication::from_str(s))
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
    #[case("do()mul(2,4)", "8")]
    #[case("abcdo()mul(2,4)", "8")]
    #[case("don't()mul(2,4)", "0")]
    #[case("abcdon't()mul(2,4)", "0")]
    #[case("do()mul(2,4)don't()mul(3,3)", "8")]
    #[case(
        "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))",
        "48"
    )]
    fn test_process_cases(#[case] input: &str, #[case] expected: &str) -> Result<()> {
        assert_eq!(expected, process(input)?);
        Ok(())
    }

    #[test]
    fn test_valid_number() {
        assert!(valid_number("123").is_ok());
        assert!(valid_number("1234").is_err());
        assert!(valid_number("").is_err());
    }

    #[test]
    fn test_mul_expression() {
        assert_eq!(
            mul_expression("mul(123,456)").unwrap().1,
            "(123,456)".to_string()
        );
        assert!(mul_expression("mul(1234,456)").is_err());
        assert!(mul_expression("mul(123,4567)").is_err());
        assert!(mul_expression("mul( 123,456)").is_err());
        assert!(mul_expression("mul(123, 456)").is_err());
    }
}
