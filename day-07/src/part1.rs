use miette::*;
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

    let total = equations
        .iter()
        .filter(|equation| process_equation(equation))
        .fold(0, |acc, (test_value, _)| acc + test_value);

    Ok(total.to_string())
}

// region: parser
fn parse_usize(input: &str) -> IResult<&str, usize> {
    // parse a digit sequence and convert it to a usize
    map_res(digit1, |s: &str| s.parse::<usize>())(input)
}

fn parse_line(input: &str) -> IResult<&str, TestEquation> {
    // 1. Parse the initial usize (test_value)
    let (input, test_value) = parse_usize(input)?;
    // 2. Parse the colon delimiter
    let (input, _) = tag(":")(input)?;
    // 3. Consume at least one whitespace character
    let (input, _) = space1(input)?;
    // 4. Parse one or more usize values separated by spaces
    let (input, vals) = separated_list1(space1, parse_usize)(input)?;

    Ok((input, (test_value, vals)))
}
// endregion

fn process_equation(equation: &TestEquation) -> bool {
    let (test_value, operands) = equation;

    // create `m` combinations binary options where:
    //  * m = 2^(n-1)
    //  * n = number of operands (length of `operands`)
    let combinations = (0..2usize.pow(operands.len() as u32 - 1)).collect::<Vec<_>>();
    let mut valid_combinations = Vec::new();

    for combination in combinations {
        let mut result = operands[0];
        for (idx, _) in operands.iter().enumerate().skip(1) {
            let mask = 1 << (idx - 1);
            result = match combination & mask == 0 {
                true => add(result, operands[idx]),
                false => mul(result, operands[idx])
            };

            if result > *test_value {
                break;
            }
        }

        if result == *test_value {
            valid_combinations.push((combination, result));
            return true;
        }
    }

    false
}

fn mul(a: usize, b: usize) -> usize {
    a * b
}

fn add(a: usize, b: usize) -> usize {
    a + b
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
        assert_eq!("3749", process(input)?);
        Ok(())
    }
}
