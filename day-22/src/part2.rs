use std::collections::HashMap;
use miette::{Diagnostic, Result};
use rayon::prelude::*;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum PuzzleError {
    #[error("Failed to parse integer")]
    #[diagnostic(code(day22::parse_error))]
    Parse(#[from] std::num::ParseIntError),

    #[error("No valid pattern found")]
    #[diagnostic(code(day22::no_pattern))]
    NoPattern,
}

type Pattern = [isize; 4];

pub struct PatternMaps {
    value_patterns: Vec<Vec<Pattern>>,
    pattern_values: HashMap<Pattern, Vec<usize>>,
}

impl PatternMaps {
    fn new() -> Self {
        Self {
            value_patterns: (0..10).map(|_| Vec::with_capacity(100)).collect::<Vec<_>>(),
            pattern_values: HashMap::with_capacity(100),
        }
    }
}

struct SecretNumber(usize);

impl SecretNumber {
    const MASK: usize = 16777216;

    #[inline]
    fn mix(&mut self, value: usize) {
        self.0 ^= value;
    }

    #[inline]
    fn prune(&mut self) {
        self.0 %= Self::MASK;
    }

    #[inline]
    fn step_1(&mut self) {
        let product = self.0 << 6;
        self.mix(product);
        self.prune();
    }

    #[inline]
    fn step_2(&mut self) {
        let quotient = self.0 >> 5;
        self.mix(quotient);
        self.prune();
    }

    #[inline]
    fn step_3(&mut self) {
        let product = self.0 << 11;
        self.mix(product);
        self.prune();
    }

    #[inline]
    fn last_digit(&self) -> usize {
        self.0 % 10
    }
}

#[tracing::instrument(skip_all)]
pub fn process(input: &str) -> Result<String, PuzzleError> {
    let mut buyers = input
        .lines()
        .map(|line| line.parse().map_err(PuzzleError::Parse))
        .collect::<Result<Vec<usize>, _>>()?;

    let (max_value, _) = max_value_and_pattern(&mut buyers)?;
    Ok(max_value.to_string())
}

fn patterns_and_values(initial: usize, iterations: usize) -> Result<PatternMaps, PuzzleError> {
    let mut secret = SecretNumber(initial);
    let mut numbers = Vec::with_capacity(iterations + 1);
    numbers.push(secret.last_digit());

    for _ in 0..iterations {
        secret.step_1();
        secret.step_2();
        secret.step_3();
        numbers.push(secret.last_digit());
    }

    let mut deltas = Vec::with_capacity(iterations + 1);
    deltas.push(0);
    for i in 1..numbers.len() {
        deltas.push(numbers[i] as isize - numbers[i - 1] as isize);
    }

    let mut maps = PatternMaps::new();
    deltas.windows(4).enumerate().for_each(|(idx, pattern)| {
        if idx + 3 < numbers.len() {
            let change_pattern = [pattern[0], pattern[1], pattern[2], pattern[3]];
            let key = numbers[idx + 3];
            maps.value_patterns[key].push(change_pattern);
            maps.pattern_values
                .entry(change_pattern)
                .or_default()
                .push(key);
        }
    });

    Ok(maps)
}

fn max_value_and_pattern(buyers: &mut [usize]) -> Result<(usize, Pattern), PuzzleError> {
    let buyer_maps: Vec<PatternMaps> = buyers
        .par_iter_mut()
        .map(|&mut buyer| patterns_and_values(buyer, 2000))
        .collect::<Result<Vec<_>, _>>()?;

    let mut all_patterns = HashMap::with_capacity(100);
    for maps in &buyer_maps {
        for pattern in maps.pattern_values.keys() {
            all_patterns.insert(*pattern, ());
        }
    }

    all_patterns
        .into_par_iter()
        .map(|(pattern, _)| {
            let value = buyer_maps
                .iter()
                .map(|maps| {
                    maps.pattern_values
                        .get(&pattern)
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(0)
                })
                .sum();
            (value, pattern)
        })
        .max_by_key(|(value, _)| *value)
        .ok_or(PuzzleError::NoPattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_number_operations() {
        let mut secret = SecretNumber(42);
        secret.mix(15);
        assert_eq!(secret.0, 37);

        let mut secret = SecretNumber(100_000_000);
        secret.prune();
        assert_eq!(secret.0, 016_113_920);
    }

    #[test]
    fn test_pattern_detection() -> Result<(), PuzzleError> {
        let input = 123;
        let maps = patterns_and_values(input, 10)?;
        
        let expected_pattern = [-1, -1, 0, 2];
        assert!(maps.value_patterns[6].contains(&expected_pattern));
        Ok(())
    }

    #[test]
    fn test_max_value_calculation() -> Result<(), PuzzleError> {
        let input = "1\n2\n3\n2024";
        let mut buyers = input.lines()
            .map(|line| line.parse().map_err(PuzzleError::Parse))
            .collect::<Result<Vec<_>, _>>()?;

        let (max_value, pattern) = max_value_and_pattern(&mut buyers)?;
        
        assert_eq!(max_value, 23);
        assert_eq!(pattern, [-2, 1, -1, 3]);
        Ok(())
    }

    #[test]
    fn test_process() -> Result<(), PuzzleError> {
        let input = "1\n2\n3\n2024";
        assert_eq!(process(input)?, "23");
        Ok(())
    }
}