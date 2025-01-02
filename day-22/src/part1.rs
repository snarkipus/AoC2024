use miette::{Diagnostic, Result};
use rayon::prelude::*;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("Failed to parse integer")]
#[diagnostic(code(day22::parse_error))]
struct ParseError(#[from] std::num::ParseIntError);

pub const VALUE_COUNT: usize = 2000;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let mut input = input
        .lines()
        .map(|line| line.parse::<usize>().map_err(ParseError))
        .collect::<Result<Vec<usize>, _>>()?;

    let result = input
        .par_iter_mut()
        .map(|secret_number| {
            evolution_process(secret_number, VALUE_COUNT);
            *secret_number
        })
        .sum::<usize>();

    Ok(result.to_string())
}

pub fn mix(value: usize, secret_number: &mut usize) {
    *secret_number ^= value
}

pub fn prune(secret_number: &mut usize) {
    *secret_number %= 16777216
}

pub fn step_1(secret_number: &mut usize) {
    let product = *secret_number << 6;
    mix(product, secret_number);
    prune(secret_number);
}

pub fn step_2(secret_number: &mut usize) {
    let quotient = *secret_number >> 5;
    mix(quotient, secret_number);
    prune(secret_number);
}

pub fn step_3(secret_number: &mut usize) {
    let product = *secret_number << 11;
    mix(product, secret_number);
    prune(secret_number);
}

pub fn evolution_process(secret_number: &mut usize, iterations: usize) {
    for _ in 0..iterations {
        step_1(secret_number);
        step_2(secret_number);
        step_3(secret_number);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "\
1
10
100
2024";
        assert_eq!("37327623", process(input)?);
        Ok(())
    }

    #[test]
    fn test_mix() {
        let mut secret_number = 42;
        mix(15, &mut secret_number);
        assert_eq!(37, secret_number);
    }

    #[test]
    fn test_prune() {
        let mut secret_number = 100000000;
        prune(&mut secret_number);
        assert_eq!(016113920, secret_number);
    }

    #[test]
    fn test_evolution_process() {
        let mut secret_number = 123;
        let expected = "\
15887950
16495136
527345
704524
1553684
12683156
11100544
12249484
7753432
5908254";

        for (i, value) in expected.lines().enumerate() {
            evolution_process(&mut secret_number, 1);
            assert_eq!(
                value.parse::<usize>().unwrap(),
                secret_number,
                "Failed at step {}",
                i + 1
            );
        }
    }
}
