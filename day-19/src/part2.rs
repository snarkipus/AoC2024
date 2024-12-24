use std::collections::{HashMap, HashSet};

#[cfg(not(feature = "debug"))]
#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, (patterns, designs)) =
        parser::parse(input).map_err(|e| miette::miette!("Failed to parse input: {}", e))?;

    let pattern_set = HashSet::<&str>::from_iter(patterns.into_iter());

    let total = designs
        .iter()
        .map(|design| {
            let mut memo = HashMap::new();
            find_combinations(design, &pattern_set, &mut memo)
        })
        .sum::<usize>();

    Ok(total.to_string())
}

#[cfg(not(feature = "debug"))]
fn find_combinations<'a>(
    input: &'a str,
    patterns: &HashSet<&str>,
    memo: &mut HashMap<&'a str, usize>,
) -> usize {
    // Check memoized result first
    if let Some(&count) = memo.get(input) {
        return count;
    }

    // Base cases
    if input.is_empty() {
        return 1;
    }

    // Early return if this string has no valid patterns within it
    if !has_any_pattern(input, patterns) {
        memo.insert(input, 0);
        return 0;
    }

    let mut total = 0;
    for split_index in 1..=input.len() {
        let (current, remaining) = input.split_at(split_index);

        if patterns.contains(current) {
            total += find_combinations(remaining, patterns, memo);
        }
    }

    memo.insert(input, total);
    total
}

#[cfg(feature = "debug")]
#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, (patterns, designs)) =
        parser::parse(input).map_err(|e| miette::miette!("Failed to parse input: {}", e))?;

    let pattern_set = HashSet::<&str>::from_iter(patterns.into_iter());

    let total = designs
        .iter()
        .map(|design| {
            println!("\nProcessing new design: {}", design);
            let mut memo = HashMap::new();
            let result = find_combinations(design, &pattern_set, &mut memo);
            println!("\nFinal memo table for {}:", design);
            for (substring, count) in memo.iter() {
                println!("  {} => {}", substring, count);
            }
            result
        })
        .sum::<usize>();

    Ok(total.to_string())
}

#[cfg(feature = "debug")]
fn find_combinations<'a>(
    input: &'a str,
    patterns: &HashSet<&str>,
    memo: &mut HashMap<&'a str, usize>,
) -> usize {
    println!("Checking: {}", input);

    // Check memoized result first
    if let Some(&count) = memo.get(input) {
        println!("  Found in memo! {} => {}", input, count);
        return count;
    }

    // Base cases
    if input.is_empty() {
        println!("  Empty string - valid path!");
        return 1;
    }

    // Early return if this string has no valid patterns within it
    if !has_any_pattern(input, patterns) {
        println!("  No valid patterns possible in: {}", input);
        memo.insert(input, 0);
        return 0;
    }

    let mut total = 0;
    for split_index in 1..=input.len() {
        let (current, remaining) = input.split_at(split_index);

        if patterns.contains(current) {
            println!("  Found pattern: {} | remaining: {}", current, remaining);
            let combinations = find_combinations(remaining, patterns, memo);
            total += combinations;
            println!(
                "  Adding {} combinations from {} | {}",
                combinations, current, remaining
            );
        }
    }

    println!("  Storing in memo: {} => {}", input, total);
    memo.insert(input, total);
    total
}

fn has_any_pattern(input: &str, patterns: &HashSet<&str>) -> bool {
    for i in 0..input.len() {
        for j in (i + 1)..=input.len() {
            if patterns.contains(&input[i..j]) {
                return true;
            }
        }
    }
    false
}

mod parser {
    use nom::{
        character::complete::{alpha1, char, newline, space0},
        multi::{many1, separated_list1},
        sequence::{delimited, separated_pair},
        IResult,
    };

    pub fn parse_patterns(input: &str) -> IResult<&str, Vec<&str>> {
        separated_list1(delimited(space0, char(','), space0), alpha1)(input)
    }

    pub fn parse_designs(input: &str) -> IResult<&str, Vec<&str>> {
        separated_list1(newline, alpha1)(input)
    }

    pub fn parse(input: &str) -> IResult<&str, (Vec<&str>, Vec<&str>)> {
        separated_pair(parse_patterns, many1(newline), parse_designs)(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "\
r, wr, b, g, bwu, rb, gb, br

brwrr
bggr
gbbr
rrbgbr
ubwu
bwurrg
brgr
bbrgwb";
        assert_eq!("16", process(input)?);
        Ok(())
    }

    // NOTE: For print output run:
    // `cargo test --package day-19 --features debug --lib -- part2::tests::test_process_single --exact --show-output`
    #[test]
    fn test_process_single() -> miette::Result<()> {
        let input = "\
r, wr, b, g, bwu, rb, gb, br

brwrr";
        assert_eq!("2", process(input)?);
        Ok(())
    }

    #[test]
    fn test_parser() {
        assert_eq!(
            parser::parse_patterns("r, wr, b, g, bwu, rb, gb, br"),
            Ok(("", vec!["r", "wr", "b", "g", "bwu", "rb", "gb", "br"]))
        );
        assert_eq!(
            parser::parse_designs("brwrr\nbggr\ngbbr\nrrbgbr\nubwu\nbwurrg\nbrgr\nbbrgwb"),
            Ok((
                "",
                vec!["brwrr", "bggr", "gbbr", "rrbgbr", "ubwu", "bwurrg", "brgr", "bbrgwb"]
            ))
        );
        assert_eq!(
            parser::parse(
                "\
r, wr, b, g, bwu, rb, gb, br

brwrr
bggr
gbbr
rrbgbr
ubwu
bwurrg
brgr
bbrgwb"
            ),
            Ok((
                "",
                (
                    vec!["r", "wr", "b", "g", "bwu", "rb", "gb", "br"],
                    vec!["brwrr", "bggr", "gbbr", "rrbgbr", "ubwu", "bwurrg", "brgr", "bbrgwb"]
                )
            ))
        );
    }
}
