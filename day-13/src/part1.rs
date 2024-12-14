use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, newline},
    combinator::value,
    multi::{many1, separated_list1},
    IResult,
};

use miette::miette;

use itertools::Itertools;

#[derive(Debug, Clone, PartialEq)]
struct SolutionPairs {
    a: i32,
    b: i32,
    cost: i32,
}

impl SolutionPairs {
    fn new(a: i32, b: i32) -> Self {
        Self {
            a,
            b,
            cost: 3 * a + b,
        }
    }
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, cases) =
        parse_multiple_entries(input).map_err(|e| miette!("Failed to parse input: {}", e))?;

    let a = 1..=100;
    let b = 1..=100;

    let pairs = a
        .cartesian_product(b)
        .map(|pair| SolutionPairs::new(pair.0, pair.1))
        .collect::<Vec<_>>();

    fn test_solution(pair: &SolutionPairs, case: &DataEntry) -> bool {
        case.button_a.dx * pair.a + case.button_b.dx * pair.b == case.prize.x
            && case.button_a.dy * pair.a + case.button_b.dy * pair.b == case.prize.y
    }

    let mut cost = 0;

    cases.iter().for_each(|case| {
        if let Some(case_cost) = pairs
            .iter()
            .filter(|pair| test_solution(pair, case))
            .map(|pair| pair.cost)
            .min()
        {
            cost += case_cost;
        }
    });

    Ok(cost.to_string())
}

// region: nom parser
#[derive(Debug, Clone, PartialEq)]
enum ButtonType {
    A,
    B,
}

#[derive(Debug, PartialEq)]
struct Coordinate {
    x: i32,
    y: i32,
}

#[derive(Debug, PartialEq)]
struct ButtonSlope {
    dx: i32,
    dy: i32,
}

#[derive(Debug, PartialEq)]
struct ButtonEntry {
    button_type: ButtonType,
    coordinate: ButtonSlope,
}

#[derive(Debug, PartialEq)]
struct DataEntry {
    button_a: ButtonSlope,
    button_b: ButtonSlope,
    prize: Coordinate,
}

fn parse_button_number(input: &str) -> IResult<&str, i32> {
    let (input, _) = char('+')(input)?;
    let (input, num_str) = digit1(input)?;
    let num = num_str.parse::<i32>().unwrap();
    Ok((input, num))
}

fn parse_prize_number(input: &str) -> IResult<&str, i32> {
    let (input, num_str) = digit1(input)?;
    let num = num_str.parse::<i32>().unwrap();
    Ok((input, num))
}

fn parse_prize_coordinate(input: &str) -> IResult<&str, Coordinate> {
    let (input, _) = tag("X=")(input)?;
    let (input, x) = parse_prize_number(input)?;
    let (input, _) = tag(", Y=")(input)?;
    let (input, y) = parse_prize_number(input)?;

    Ok((input, Coordinate { x, y }))
}

fn parse_button_coordinate(input: &str) -> IResult<&str, ButtonSlope> {
    let (input, _) = tag("X")(input)?;
    let (input, x) = parse_button_number(input)?;
    let (input, _) = tag(", Y")(input)?;
    let (input, y) = parse_button_number(input)?;

    Ok((input, ButtonSlope { dx: x, dy: y }))
}

fn parse_button_type(input: &str) -> IResult<&str, ButtonType> {
    let (input, _) = tag("Button ")(input)?;
    let (input, button_type) = alt((
        value(ButtonType::A, char('A')),
        value(ButtonType::B, char('B')),
    ))(input)?;
    let (input, _) = tag(": ")(input)?;

    Ok((input, button_type))
}

fn parse_button_line(input: &str) -> IResult<&str, ButtonEntry> {
    let (input, button_type) = parse_button_type(input)?;
    let (input, coordinate) = parse_button_coordinate(input)?;

    Ok((
        input,
        ButtonEntry {
            button_type,
            coordinate,
        },
    ))
}

fn parse_prize_line(input: &str) -> IResult<&str, Coordinate> {
    let (input, _) = tag("Prize: ")(input)?;
    parse_prize_coordinate(input)
}

fn parse_data_entry(input: &str) -> IResult<&str, DataEntry> {
    let (input, button_a_entry) = parse_button_line(input)?;
    let (input, _) = newline(input)?;
    let (input, button_b_entry) = parse_button_line(input)?;
    let (input, _) = newline(input)?;
    let (input, prize) = parse_prize_line(input)?;

    Ok((
        input,
        DataEntry {
            button_a: button_a_entry.coordinate,
            button_b: button_b_entry.coordinate,
            prize,
        },
    ))
}

// Optional: parse multiple entries separated by newlines
fn parse_multiple_entries(input: &str) -> IResult<&str, Vec<DataEntry>> {
    let (remaining, entries) = separated_list1(many1(newline), parse_data_entry)(input)?;

    Ok((remaining, entries))
}
// endregion

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "Button A: X+94, Y+34
Button B: X+22, Y+67
Prize: X=8400, Y=5400

Button A: X+26, Y+66
Button B: X+67, Y+21
Prize: X=12748, Y=12176

Button A: X+17, Y+86
Button B: X+84, Y+37
Prize: X=7870, Y=6450

Button A: X+69, Y+23
Button B: X+27, Y+71
Prize: X=18641, Y=10279";
        assert_eq!("480", process(input)?);
        Ok(())
    }

    #[test]
    fn test_button_type() {
        assert_eq!(parse_button_type("Button A: "), Ok(("", ButtonType::A)));
        assert_eq!(parse_button_type("Button B: "), Ok(("", ButtonType::B)));
        assert!(parse_button_type("Button C: ").is_err());
    }

    #[test]
    fn test_button_slope() {
        assert_eq!(
            parse_button_coordinate("X+94, Y+34"),
            Ok(("", ButtonSlope { dx: 94, dy: 34 }))
        );
    }

    #[test]
    fn test_prize_coordinate() {
        assert_eq!(
            parse_prize_coordinate("X=8400, Y=5400"),
            Ok(("", Coordinate { x: 8400, y: 5400 }))
        );
    }

    #[test]
    fn test_complete_entry() {
        let input = "\
Button A: X+94, Y+34
Button B: X+22, Y+67
Prize: X=8400, Y=5400";

        let result = parse_data_entry(input);
        assert!(result.is_ok());

        let (remaining, entry) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(entry.button_a, ButtonSlope { dx: 94, dy: 34 });
        assert_eq!(entry.button_b, ButtonSlope { dx: 22, dy: 67 });
        assert_eq!(entry.prize, Coordinate { x: 8400, y: 5400 });
    }

    #[test]
    fn test_multiple_entries() {
        let input = "\
Button A: X+94, Y+34
Button B: X+22, Y+67
Prize: X=8400, Y=5400

Button A: X+26, Y+66
Button B: X+67, Y+21
Prize: X=12748, Y=12176";

        let result = parse_multiple_entries(input);
        assert!(result.is_ok());

        let (remaining, entries) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(entries.len(), 2);

        // Verify first entry
        assert_eq!(entries[0].button_a, ButtonSlope { dx: 94, dy: 34 });
        assert_eq!(entries[0].prize, Coordinate { x: 8400, y: 5400 });

        // Verify second entry
        assert_eq!(entries[1].button_a, ButtonSlope { dx: 26, dy: 66 });
        assert_eq!(entries[1].prize, Coordinate { x: 12748, y: 12176 });
    }
}
