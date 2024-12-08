use miette::*;
use rayon::prelude::*;
use thiserror::Error;

use nom::{
    bytes::complete::tag,
    character::complete::{digit1, space1},
    combinator::map_res,
    multi::separated_list1,
    IResult,
};

type TestEquation = (usize, Vec<usize>);

#[derive(Debug, Diagnostic, Error)]
#[error("Failed to parse line: {line}")]
struct ParseLineError {
    line: String,
    #[source_code]
    src: String,
    #[label("here")]
    span: SourceSpan,
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let equations: Vec<TestEquation> = input
        .lines()
        .filter_map(|line| {
            parse_line(line)
                .map(|(_, result)| result)
                .map_err(|_| ParseLineError {
                    line: line.to_string(),
                    src: line.to_string(),
                    span: (0, line.len()).into(),
                })
                .into_diagnostic()
                .ok()
        })
        .collect();

    // Use parallel iterator and sum for the final result
    let total: usize = equations
        .par_iter()
        .filter(|equation| process_equation(equation))
        .map(|(test_value, _)| test_value)
        .sum();

    Ok(total.to_string())
}

// region: parser
fn parse_usize(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse::<usize>())(input)
}

fn parse_line(input: &str) -> IResult<&str, TestEquation> {
    let (input, test_value) = parse_usize(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = space1(input)?;
    let (input, vals) = separated_list1(space1, parse_usize)(input)?;

    Ok((input, (test_value, vals)))
}
// endregion

fn process_equation(equation: &TestEquation) -> bool {
    let (test_value, operands) = equation;
    let combinations = (0..3usize.pow(operands.len() as u32 - 1)).collect::<Vec<_>>();

    // Use parallel iterator to check combinations
    combinations.par_iter().any(|&combination| {
        let mut result = operands[0];
        let mut current_combination = combination;

        for (idx, _) in operands.iter().enumerate().skip(1) {
            let operation = current_combination % 3;
            current_combination /= 3;

            result = match operation {
                0 => add(result, operands[idx]),
                1 => mul(result, operands[idx]),
                2 => concat(result, operands[idx]),
                _ => unreachable!(),
            };

            if result > *test_value {
                return false;
            }
        }

        result == *test_value
    })
}

fn mul(a: usize, b: usize) -> usize {
    a * b
}

fn add(a: usize, b: usize) -> usize {
    a + b
}

fn concat(a: usize, b: usize) -> usize {
    let a_str = a.to_string();
    let b_str = b.to_string();
    let result = a_str + &b_str;

    result.parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "190: 10 19
3267: 81 40 27
83: 17 5
156: 15 6
7290: 6 8 6 15
161011: 16 10 13
192: 17 8 14
21037: 9 7 18 13
292: 11 6 16 20";
        assert_eq!("11387", process(input)?);
        Ok(())
    }

    #[test]
    fn test_concat() -> miette::Result<()> {
        assert_eq!(concat(1, 2), 12);
        assert_eq!(concat(12, 34), 1234);
        assert_eq!(concat(123, 456), 123456);
        Ok(())
    }
}
