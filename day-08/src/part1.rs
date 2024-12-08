use miette::{Diagnostic, SourceSpan};
use nom::{
    character::complete::{newline, satisfy},
    multi::{many1, separated_list1},
    IResult, Parser,
};
use nom_locate::LocatedSpan;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};
use thiserror::Error;
use itertools::Itertools;

#[derive(Debug, Error, Diagnostic)]
#[error("Failed to parse grid")]
#[diagnostic(
    code(day8::parse_error),
    help("Input must contain only dots (.), digits (0-9), or uppercase letters (A-Z)")
)]
struct GridParseError {
    #[source_code]
    src: String,
    #[label("Parse error occurred here")]
    span: SourceSpan,
    kind: nom::error::ErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Antinode {
    x: isize,
    y: isize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Antenna(Location);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Location {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Map {
    xdim: usize,
    ydim: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AntennaSet(HashMap<char, Vec<Antenna>>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct AntinodeSet(HashSet<Antinode>);

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (map, antennas) = parse_input(input)?;
    let mut antinodes = calculate_antinodes(&antennas)?;

    antinodes.0.retain(|antinode| {
        antinode.x > 0 && 
        antinode.y > 0 && 
        antinode.x <= map.xdim as isize && 
        antinode.y <= map.ydim as isize
    });

    antinodes.0.iter().for_each(|antinode| {
        tracing::debug!("Antinode: {:?}", antinode);
    });

    Ok(antinodes.0.len().to_string())
}

fn parse_input(input: &str) -> miette::Result<(Map, AntennaSet)> {
    let mut antenna_set = AntennaSet(HashMap::new());
    let map = Map {
        xdim: input.lines().next().unwrap().len(),
        ydim: input.lines().count(),
    };

    tracing::debug!("Map dimensions: {}x{}", map.xdim, map.ydim);

    let result = parse_grid(LocatedSpan::new(input));

    match result {
        Ok((_, result)) => {
            for c in result.iter().filter(|c| c.character != EMPTY) {
                antenna_set
                    .0
                    .entry(c.character)
                    .or_default()
                    .push(Antenna(Location {
                        x: c.position.get_column(),
                        y: c.position.location_line() as usize,
                    }));
            }
            Ok((map, antenna_set))
        }
        Err(nom::Err::Error(e)) => {
            let offset = e.input.location_offset();
            let err = GridParseError {
                src: input.to_string(),
                span: (offset, 1).into(),
                kind: e.code,
            };
            Err(err.into())
        }
        Err(e) => {
            // Handle other error variants (Failure, Incomplete) if needed
            Err(miette::Error::msg(format!("Parse error: {:?}", e)))
        }
    }
}

fn calculate_antinodes(antennas: &AntennaSet) -> miette::Result<AntinodeSet> {
    let mut antinodes = AntinodeSet(HashSet::new());

    for antenna_locations  in antennas.0.values() {
        let antenna_pairs = antenna_locations
            .iter()
            .combinations(2)
            .map(|pair| (pair[0], pair[1]))
            .collect::<Vec<_>>();

        for (a, b) in antenna_pairs.iter() { 
            let (anti_a, anti_b) = calculate_antinode_pair(a, b);
            antinodes.0.insert(anti_a);
            antinodes.0.insert(anti_b);
        }
    }
    
    Ok(antinodes)
}

fn calculate_slope(a: &Antenna, b: &Antenna) -> (isize, isize) {
    let rise = b.0.y as isize - a.0.y as isize;
    let run = b.0.x as isize - a.0.x as isize;

    (rise, run)
}

fn calculate_antinode_pair(a: &Antenna, b: &Antenna) -> (Antinode, Antinode) {
    let (rise, run) = calculate_slope(a, b);

    let antinode_a = Antinode {
        x: a.0.x as isize - run,
        y: a.0.y as isize - rise,
    };

    let antinode_b = Antinode {
        x: b.0.x as isize + run,
        y: b.0.y as isize + rise,
    };

    (antinode_a, antinode_b)
}

// region: nom parser
const EMPTY: char = '.';

type CharSpan<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LocatedChar<'a> {
    character: char,
    position: CharSpan<'a>,
}

fn parse_alphanumeric(input: CharSpan) -> IResult<CharSpan, LocatedChar> {
    satisfy(|c: char| c.is_ascii_alphanumeric() || c == EMPTY)
        .map(|c| LocatedChar {
            character: c,
            position: input,
        })
        .parse(input)
}

fn parse_grid(input: CharSpan) -> IResult<CharSpan, Vec<LocatedChar>> {
    let (input, lines) = separated_list1(newline, many1(parse_alphanumeric))(input)?;
    Ok((input, lines.into_iter().flatten().collect()))
}
// endregion

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test]
    fn test_process() -> miette::Result<()> {
        let input = "............
........0...
.....0......
.......0....
....0.......
......A.....
............
............
........A...
.........A..
............
............";
        assert_eq!("14", process(input)?);
        Ok(())
    }

    #[test_log::test]
    fn test_parse_grid() -> miette::Result<()> {
        let input = LocatedSpan::new(
            "............
........0...
.....0......
.......0....
....0.......
......A.....
............
............
........A...
.........A..
............
............",
        );

        let result = parse_grid(input);
        let mut antenna_count = 0;
        match &result {
            Ok((_, result)) => {
                for c in result.iter() {
                    if c.character != EMPTY {
                        antenna_count += 1;
                        tracing::debug!(
                            "char: {}, line: {}, col: {}",
                            c.character,
                            c.position.location_line(),
                            c.position.get_column()
                        );
                    }
                }
            }
            Err(e) => {
                dbg!(e);
            }
        }

        assert_eq!(antenna_count, 7);
        Ok(())
    }

    #[test_log::test]
    fn test_parse_grid_small() -> miette::Result<()> {
        let input = LocatedSpan::new(
            "...\n.0.\n..A", // Simplified 3x3 grid for clear testing
        );

        match parse_grid(input) {
            Ok((_, chars)) => {
                // Total length should be 9 (3x3 grid)
                assert_eq!(chars.len(), 9);

                // Check some specific positions
                let zero = chars.iter().find(|c| c.character == '0').unwrap();
                let a = chars.iter().find(|c| c.character == 'A').unwrap();

                // Zero should be on line 2, position 1
                assert_eq!(zero.position.location_line(), 2);
                assert_eq!(zero.position.get_column(), 2);

                // 'A' should be on line 3, position 2
                assert_eq!(a.position.location_line(), 3);
                assert_eq!(a.position.get_column(), 3);

                // Count non-empty characters
                let non_empty = chars.iter().filter(|c| c.character != '.').count();
                assert_eq!(non_empty, 2);
            }
            Err(e) => {
                panic!("Failed to parse grid: {:?}", e);
            }
        }

        Ok(())
    }

    #[test_log::test]
    fn test_parse_input() -> miette::Result<()> {
        let input = "...\n.0.\n..A";
        let (_map, antennas) = parse_input(input).unwrap();

        // Check the map
        // assert_eq!(map.locations.len(), 3);
        // assert_eq!(map.locations[0].len(), 3);

        // Check the antennas
        assert_eq!(antennas.0.len(), 2);

        Ok(())
    }

    #[test_log::test]
    fn test_calculate_slope() -> miette::Result<()> {
        let a = Antenna(Location { x: 0, y: 0 });
        let b = Antenna(Location { x: 3, y: 4 });
        assert_eq!(calculate_slope(&a, &b), (4, 3));

        // negative slope
        let a = Antenna(Location { x: 0, y: 4 });
        let b = Antenna(Location { x: 3, y: 0 });
        assert_eq!(calculate_slope(&a, &b), (-4, 3));

        Ok(())
    }

    #[test_log::test]
    fn test_calculate_antinode_pair() -> miette::Result<()> {
        let expected_antinodes = (
            Antinode { x: 0, y: 0 },
            Antinode { x: 3, y: 3 },
        );

        let antinode_pair = calculate_antinode_pair(
            &Antenna(Location { x: 1, y: 1 }),
            &Antenna(Location { x: 2, y: 2 }),
        );

        assert_eq!(antinode_pair, expected_antinodes);

        Ok(())
    }

    #[test_log::test]
    fn test_calculate_antinodes() -> miette::Result<()> {
        let antennas = HashMap::from([
            ('A', vec![
                Antenna(Location { x: 1, y: 1 }),
                Antenna(Location { x: 2, y: 2 }),
            ])
        ]);

        let expected_antinodes = HashSet::from([
            Antinode { x: 0, y: 0 },
            Antinode { x: 3, y: 3 },
        ]);

        let antinodes = calculate_antinodes(&AntennaSet(antennas))?;

        assert_eq!(antinodes.0, expected_antinodes);
        
        Ok(())
    }
}
