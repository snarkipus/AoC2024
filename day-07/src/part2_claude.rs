use rayon::prelude::*;

use nom::{
    bytes::complete::tag,
    character::complete::{digit1, space1},
    combinator::map_res,
    multi::separated_list1,
    IResult,
};

type TestEquation = (usize, Vec<usize>);

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    // Pre-allocate with capacity based on rough line count
    let line_count = input.bytes().filter(|&b| b == b'\n').count() + 1;
    let mut equations = Vec::with_capacity(line_count);

    // Process lines without collecting into intermediate Vec
    for line in input.lines() {
        if let Ok((_, result)) = parse_line(line) {
            equations.push(result);
        }
    }

    // Process in parallel with chunk size optimization
    let chunk_size = (equations.len() / rayon::current_num_threads()).max(1);
    let total: usize = equations
        .par_iter()
        .with_min_len(chunk_size)
        .filter(|equation| process_equation(equation))
        .map(|(test_value, _)| test_value)
        .sum();

    Ok(total.to_string())
}

#[inline]
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

fn process_equation(equation: &TestEquation) -> bool {
    let (test_value, operands) = equation;

    // Early return for single operand case
    if operands.len() == 1 {
        return operands[0] == *test_value;
    }

    // Pre-calculate powers of 3 up to max needed size
    let powers = (0..operands.len() - 1)
        .map(|i| 3usize.pow(i as u32))
        .collect::<Vec<_>>();

    // Calculate total combinations needed
    let max_combinations = 3usize.pow(operands.len() as u32 - 1);

    // Use chunks for better cache utilization
    let chunk_size = 1024;
    (0..max_combinations)
        .collect::<Vec<_>>()
        .par_chunks(chunk_size)
        .any(|chunk| {
            chunk.iter().any(|&combination| {
                let mut result = operands[0];

                // Use pre-calculated powers instead of repeated division
                for (idx, power) in powers.iter().enumerate() {
                    let operation = (combination / power) % 3;

                    // Short circuit if we're already over the target
                    if result > *test_value && operation != 2 {
                        // Don't short circuit for concat
                        return false;
                    }

                    result = match operation {
                        0 => add(result, operands[idx + 1]),
                        1 => mul(result, operands[idx + 1]),
                        2 => {
                            // Only convert to string if absolutely necessary
                            if result > 999_999_999 || operands[idx + 1] > 999_999_999 {
                                concat(result, operands[idx + 1])
                            } else {
                                // Fast path for smaller numbers
                                fast_concat(result, operands[idx + 1])
                            }
                        }
                        _ => unreachable!(),
                    };
                }

                result == *test_value
            })
        })
}

#[inline]
fn mul(a: usize, b: usize) -> usize {
    a * b
}

#[inline]
fn add(a: usize, b: usize) -> usize {
    a + b
}

// Fast path for concatenation of smaller numbers
#[inline]
fn fast_concat(a: usize, b: usize) -> usize {
    // Determine number of digits in b
    let digits = if b < 10 {
        1
    } else if b < 100 {
        2
    } else if b < 1000 {
        3
    } else if b < 10000 {
        4
    } else if b < 100000 {
        5
    } else if b < 1000000 {
        6
    } else if b < 10000000 {
        7
    } else if b < 100000000 {
        8
    } else {
        9
    };

    a * 10_usize.pow(digits as u32) + b
}

// Fallback for very large numbers
#[inline]
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

    #[test]
    fn test_fast_concat() {
        assert_eq!(fast_concat(1, 2), 12);
        assert_eq!(fast_concat(12, 34), 1234);
        assert_eq!(fast_concat(123, 456), 123456);
    }
}
