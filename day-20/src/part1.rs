use pathfinding::grid::Grid as PathGrid;
use pathfinding::prelude::*;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

mod types {
    pub type Position = (usize, usize);
}
use types::Position;

// Configuration constants
#[cfg(test)]
pub const SHORTCUT_THRESHOLD: usize = 30;

#[cfg(not(test))]
pub const SHORTCUT_THRESHOLD: usize = 100;

// Main processing function
#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    // Parse input and create initial grid
    let parsed_grid = parser::parse_input(input)?;
    let grid = graph::create_grid(&parsed_grid)?;
    let (start, end) = graph::find_endpoints(&parsed_grid)?;

    // Create pathfinding grid and get original path length
    let path_grid = graph::create_pathfinding_grid(&grid);
    let original_path_length = pathing::find_shortest_path(&path_grid, start, end)?;

    // Find and evaluate shortcut candidates
    let candidates = shortcuts::find_candidates(&path_grid)?;
    let improvements =
        shortcuts::evaluate_candidates(&path_grid, &candidates, start, end, original_path_length)?;

    // Count significant shortcuts
    let significant_shortcuts = improvements
        .iter()
        .filter(|(_, &improvement)| improvement >= SHORTCUT_THRESHOLD)
        .count();

    Ok(significant_shortcuts.to_string())
}

// Parser module - Handles input parsing
mod parser {
    use nom::{
        character::complete::{newline, satisfy},
        multi::{many1, separated_list1},
        IResult, Parser,
    };
    use nom_locate::LocatedSpan;

    pub type Span<'a> = LocatedSpan<&'a str>;

    const WALL: char = '#';
    const EMPTY: char = '.';
    pub(crate) const START: char = 'S';
    pub(crate) const END: char = 'E';

    #[derive(Debug, Clone, PartialEq)]
    pub struct Cell<'a> {
        pub value: char,
        pub position: Span<'a>,
    }

    pub type ParsedGrid<'a> = Vec<Vec<Cell<'a>>>;

    pub fn parse_input(input: &str) -> miette::Result<ParsedGrid> {
        let span = Span::new(input);
        let (_, grid) = parse(span).map_err(|e| miette::miette!("Failed to parse input: {}", e))?;
        Ok(grid)
    }

    pub(crate) fn parse(input: Span) -> IResult<Span, ParsedGrid> {
        separated_list1(
            newline,
            many1(
                satisfy(|c| matches!(c, WALL | EMPTY | START | END)).map(|c| Cell {
                    value: c,
                    position: input,
                }),
            ),
        )
        .parse(input)
    }
}

// Graph module - Handles grid creation and manipulation
mod graph {
    use super::*;

    pub fn create_grid(parsed_grid: &parser::ParsedGrid) -> miette::Result<PathGrid> {
        let wall_coords: Vec<Position> = find_cells(parsed_grid, |cell| cell.value == '#');
        PathGrid::from_coordinates(&wall_coords).ok_or(miette::miette!("Failed to create grid"))
    }

    pub fn find_endpoints(
        parsed_grid: &parser::ParsedGrid,
    ) -> miette::Result<(Position, Position)> {
        let start = find_cells(parsed_grid, |cell| cell.value == 'S')
            .into_iter()
            .next()
            .ok_or(miette::miette!("Start position not found"))?;

        let end = find_cells(parsed_grid, |cell| cell.value == 'E')
            .into_iter()
            .next()
            .ok_or(miette::miette!("End position not found"))?;

        Ok((start, end))
    }

    pub fn create_pathfinding_grid(grid: &PathGrid) -> PathGrid {
        let mut pathfinding_grid = grid.clone();
        pathfinding_grid.invert();
        pathfinding_grid
    }

    fn find_cells(
        grid: &parser::ParsedGrid,
        predicate: impl Fn(&parser::Cell) -> bool,
    ) -> Vec<Position> {
        grid.iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .filter(|(_, cell)| predicate(cell))
                    .map(move |(x, _)| (x, y))
            })
            .collect()
    }
}

// Pathfinding module - Handles path calculation
mod pathing {
    use super::*;

    pub fn find_shortest_path(
        grid: &PathGrid,
        start: Position,
        end: Position,
    ) -> miette::Result<usize> {
        let (_, path_length) = astar(
            &start,
            |p| grid.neighbours(*p).into_iter().map(|n| (n, 1)),
            |p| manhattan_distance(*p, end),
            |p| *p == end,
        )
        .ok_or(miette::miette!("No path found"))?;

        Ok(path_length)
    }

    fn manhattan_distance(pos: Position, target: Position) -> usize {
        ((pos.0 as i32 - target.0 as i32).abs() + (pos.1 as i32 - target.1 as i32).abs()) as usize
    }
}

// Shortcuts module - Handles finding and evaluating shortcuts
mod shortcuts {
    use super::*;

    pub fn find_candidates(grid: &PathGrid) -> miette::Result<HashSet<Position>> {
        let mut candidates = HashSet::new();

        for x in 1..grid.width - 1 {
            for y in 1..grid.height - 1 {
                if is_valid_shortcut(grid, x, y) {
                    candidates.insert((x, y));
                }
            }
        }

        Ok(candidates)
    }

    pub fn evaluate_candidates(
        grid: &PathGrid,
        candidates: &HashSet<Position>,
        start: Position,
        end: Position,
        original_length: usize,
    ) -> miette::Result<HashMap<Position, usize>> {
        let score_table = Arc::new(Mutex::new(HashMap::new()));

        candidates.par_iter().for_each(|&pos| {
            if let Ok(improvement) = evaluate_shortcut(grid, pos, start, end, original_length) {
                let mut table = score_table.lock().expect("Failed to lock score table");
                table.insert(pos, improvement);
            }
        });

        let result = Ok(score_table
            .lock()
            .expect("Failed to lock score table")
            .clone());
        result
    }

    fn is_valid_shortcut(grid: &PathGrid, x: usize, y: usize) -> bool {
        if grid.has_vertex((x, y)) {
            return false;
        }

        let horizontal = grid.has_vertex((x - 1, y)) && grid.has_vertex((x + 1, y));
        let vertical = grid.has_vertex((x, y - 1)) && grid.has_vertex((x, y + 1));

        horizontal || vertical
    }

    fn evaluate_shortcut(
        grid: &PathGrid,
        pos: Position,
        start: Position,
        end: Position,
        original_length: usize,
    ) -> miette::Result<usize> {
        let mut test_grid = grid.clone();
        test_grid.add_vertex(pos);

        let new_length = pathing::find_shortest_path(&test_grid, start, end)?;
        Ok(original_length - new_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    const EXAMPLE_LARGE: &str = "\
###############
#...#...#.....#
#.#.#.#.#.###.#
#S#...#.#.#...#
#######.#.#.###
#######.#.#...#
#######.#.###.#
###..E#...#...#
###.#######.###
#...###...#...#
#.#####.#.###.#
#.#...#.#.#...#
#.#.#.#.#.#.###
#...#...#...###
###############";

    const EXAMPLE_SMALL: &str = "\
#######
#S#...#
#.#.#.#
#...#E#
#######";

    #[test]
    fn test_process_large() -> miette::Result<()> {
        assert_eq!("4", process(EXAMPLE_LARGE)?);
        Ok(())
    }

    #[test]
    fn test_adding_shortcut() -> miette::Result<()> {
        // Parse and create initial grid
        let parsed_grid = parser::parse_input(EXAMPLE_LARGE)?;
        let grid = graph::create_grid(&parsed_grid)?;
        let (start, end) = graph::find_endpoints(&parsed_grid)?;

        // Get original path length
        let mut path_grid = graph::create_pathfinding_grid(&grid);
        let original_length = pathing::find_shortest_path(&path_grid, start, end)?;
        assert_eq!(original_length, 84);

        // Add a known shortcut and verify improvement
        path_grid.add_vertex((8, 1));
        let new_length = pathing::find_shortest_path(&path_grid, start, end)?;
        assert_eq!(new_length, 72);

        Ok(())
    }

    #[test]
    fn test_parser() -> miette::Result<()> {
        let parsed_grid = parser::parse_input(EXAMPLE_SMALL)?;

        assert_eq!(parsed_grid.len(), 5);
        assert_eq!(parsed_grid[0].len(), 7);
        assert_eq!(parsed_grid.iter().flatten().count(), 35);

        // Verify start and end cells are in correct positions
        assert_eq!(parsed_grid[1][1].value, parser::START);
        assert_eq!(parsed_grid[3][5].value, parser::END);

        Ok(())
    }

    #[test]
    fn test_grid_creation() -> miette::Result<()> {
        let parsed_grid = parser::parse_input(EXAMPLE_SMALL)?;
        let grid = graph::create_grid(&parsed_grid)?;

        // Verify grid dimensions and properties
        assert_eq!(grid.width, 7);
        assert_eq!(grid.height, 5);
        assert_eq!(grid.size(), 35);
        assert_eq!(grid.vertices_len(), 24);

        // Verify start and end positions
        let (start, end) = graph::find_endpoints(&parsed_grid)?;
        assert_eq!(start, (1, 1));
        assert_eq!(end, (5, 3));

        Ok(())
    }

    #[test]
    fn test_shortcut_detection() -> miette::Result<()> {
        // Setup
        let parsed_grid = parser::parse_input(EXAMPLE_SMALL)?;
        let grid = graph::create_grid(&parsed_grid)?;
        let path_grid = graph::create_pathfinding_grid(&grid);

        // Find candidates
        let candidates = shortcuts::find_candidates(&path_grid)?;

        // Verify expected candidates
        assert_eq!(candidates.len(), 4);
        assert!(candidates.contains(&(2, 1)));
        assert!(candidates.contains(&(2, 2)));
        assert!(candidates.contains(&(4, 2)));
        assert!(candidates.contains(&(4, 3)));

        // Optional: Print debug info
        println!("Path Grid:\n{:#?}\n", path_grid);
        println!("Edges:");
        path_grid.edges().sorted().for_each(|(a, b)| {
            println!("{:?} -> {:?}", a, b);
        });
        println!("\nCandidates: {:?}", candidates);

        Ok(())
    }

    #[test]
    fn test_shortcut_evaluation() -> miette::Result<()> {
        // Setup
        let parsed_grid = parser::parse_input(EXAMPLE_SMALL)?;
        let grid = graph::create_grid(&parsed_grid)?;
        let (start, end) = graph::find_endpoints(&parsed_grid)?;
        let path_grid = graph::create_pathfinding_grid(&grid);

        // Get original path length
        let original_length = pathing::find_shortest_path(&path_grid, start, end)?;

        // Find and evaluate candidates
        let candidates = shortcuts::find_candidates(&path_grid)?;
        let improvements =
            shortcuts::evaluate_candidates(&path_grid, &candidates, start, end, original_length)?;

        // Verify we found improvements
        assert!(!improvements.is_empty());
        println!("Shortcut improvements: {:?}", improvements);

        Ok(())
    }
}
