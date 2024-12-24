use std::collections::HashSet;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, (patterns, designs)) =
        parser::parse(input).map_err(|e| miette::miette!("Failed to parse input: {}", e))?;

    let pattern_set = HashSet::<&str>::from_iter(patterns.into_iter());

    // Count how many designs can be fully broken down
    let valid_count = designs
        .iter()
        .filter(|&design| can_break_down(design, &pattern_set))
        .count();

    Ok(valid_count.to_string())
}

fn can_break_down(design: &str, patterns: &HashSet<&str>) -> bool {
    if design.is_empty() {
        return true;
    }
    if patterns.contains(design) {
        return true;
    }

    for split_index in 1..=design.len() {
        let (left, right) = design.split_at(split_index);
        if patterns.contains(left) && can_break_down(right, patterns) {
            return true;
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
        assert_eq!("6", process(input)?);
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
