use pathfinding::grid::Grid as PathGrid;
use pathfinding::prelude::*;
use rayon::prelude::*;

mod types {
    pub type Position = (usize, usize);
}
use types::Position;

// Configuration constants
#[cfg(test)]
pub const SHORTCUT_THRESHOLD: usize = 10;

#[cfg(not(test))]
pub const SHORTCUT_THRESHOLD: usize = 100;

// Main processing function
#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let parsed_grid = parser::parse_input(input)?;
    let grid = graph::create_grid(&parsed_grid)?;
    let (start, end) = graph::find_endpoints(&parsed_grid)?;

    let path_grid = graph::create_pathfinding_grid(&grid);
    let original_path_length = pathing::find_shortest_path(&path_grid, start, end)?;

    let candidates = shortcuts::find_candidates(&path_grid)?;
    let improvements = shortcuts::evaluate_candidates(
        &path_grid,
        &candidates,
        start,
        end,
        original_path_length,
    )?;

    Ok(improvements.len().to_string())
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
    use rayon::prelude::*;
    use std::collections::{HashMap, HashSet};

    pub fn find_candidates(grid: &PathGrid) -> miette::Result<HashSet<Position>> {
        let mut candidates = HashSet::new();
        let path_vertices = get_path_vertices(grid);
        
        // Scale up radius based on grid size
        let max_radius = (grid.width.max(grid.height) / 2).min(20);
        
        for &pos in &path_vertices {
            for radius in 1..=max_radius {
                let points = get_points_at_radius(grid, pos, radius);
                let new_candidates: HashSet<_> = points
                    .into_iter()
                    .filter(|&p| is_valid_position(grid, p))
                    .collect();
                    
                candidates.extend(new_candidates);
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
        candidates
            .par_iter()
            .map(|&pos| -> miette::Result<Option<(Position, usize)>> {
                let improvement = evaluate_shortcut(grid, pos, start, end, original_length)?;
                Ok(if improvement >= SHORTCUT_THRESHOLD {
                    Some((pos, improvement))
                } else {
                    None
                })
            })
            .filter_map(|result| result.transpose())
            .collect()
    }

    pub(crate) fn evaluate_shortcut(
        grid: &PathGrid,
        shortcut: Position,
        start: Position,
        end: Position,
        original_length: usize,
    ) -> miette::Result<usize> {
        let mut test_grid = grid.clone();
        test_grid.add_vertex(shortcut);

        let new_length = pathing::find_shortest_path(&test_grid, start, end)?;
        if new_length < original_length {
            Ok(original_length - new_length)
        } else {
            Ok(0)
        }
    }

    // Core path finding functions
    fn find_endpoints(grid: &PathGrid) -> miette::Result<(Position, Position)> {
        // Find the "lowest" and "highest" vertices in the grid
        let vertices: Vec<Position> = get_path_vertices(grid);
        if vertices.is_empty() {
            return Err(miette::miette!("No vertices found in grid"));
        }

        let start = vertices
            .iter()
            .min_by_key(|&&(x, y)| (y, x))
            .copied()
            .ok_or(miette::miette!("No start position found"))?;

        let end = vertices
            .iter()
            .max_by_key(|&&(x, y)| (y, x))
            .copied()
            .ok_or(miette::miette!("No end position found"))?;

        Ok((start, end))
    }

    fn find_path_vertices(
        grid: &PathGrid,
        start: Position,
        end: Position,
    ) -> miette::Result<Vec<Position>> {
        let (path, _) = astar(
            &start,
            |p| grid.neighbours(*p).into_iter().map(|n| (n, 1)),
            |p| manhattan_distance(*p, end),
            |p| *p == end,
        )
        .ok_or(miette::miette!("No path found"))?;

        Ok(path)
    }

    fn find_shortcuts_from_point(
        grid: &PathGrid,
        point: Position,
        start: Position,
        end: Position,
    ) -> miette::Result<HashSet<Position>> {
        let mut shortcuts = HashSet::new();
        let mut visited = HashSet::new();
        
        // Get original path length
        let original_length = pathing::find_shortest_path(grid, start, end)?;
        
        // Check shortcuts at increasing distances
        for radius in 1..=20 {
            let points_at_radius = get_points_at_radius(grid, point, radius);
            
            for pos in points_at_radius {
                if visited.contains(&pos) {
                    continue;
                }
                visited.insert(pos);
                
                // Only consider positions that aren't walls
                if !grid.has_vertex(pos) {
                    // Test if this shortcut actually improves the path
                    let mut test_grid = grid.clone();
                    test_grid.add_vertex(pos);
                    
                    if let Ok(new_length) = pathing::find_shortest_path(&test_grid, start, end) {
                        let improvement = original_length - new_length;
                        if improvement >= SHORTCUT_THRESHOLD {
                            shortcuts.insert(pos);
                        }
                    }
                }
            }
        }
        
        Ok(shortcuts)
    }

    pub(crate) fn get_points_at_radius(grid: &PathGrid, center: Position, radius: usize) -> HashSet<Position> {
        let mut points = HashSet::new();
        let (cx, cy) = (center.0 as i32, center.1 as i32);
        let width = grid.width as i32;
        let height = grid.height as i32;

        // Generate points at exact manhattan distance
        for dx in -(radius as i32)..=radius as i32 {
            let y_offset = radius as i32 - dx.abs();
            if y_offset == 0 {
                // Points on horizontal axis at radius distance
                let x = cx + dx;
                let y = cy;
                if x >= 0 && x < width && y >= 0 && y < height {
                    points.insert((x as usize, y as usize));
                }
            } else {
                // Points above and below at remaining distance
                let x = cx + dx;
                let y1 = cy + y_offset;
                let y2 = cy - y_offset;
                
                if x >= 0 && x < width {
                    if y1 >= 0 && y1 < height {
                        points.insert((x as usize, y1 as usize));
                    }
                    if y2 >= 0 && y2 < height {
                        points.insert((x as usize, y2 as usize));
                    }
                }
            }
        }
        
        points
    }

    // Helper functions
    fn get_path_vertices(grid: &PathGrid) -> Vec<Position> {
        (0..grid.width)
            .flat_map(|x| (0..grid.height).map(move |y| (x, y)))
            .filter(|&pos| grid.has_vertex(pos))
            .collect()
    }

    fn is_valid_position(grid: &PathGrid, pos: Position) -> bool {
        if grid.has_vertex(pos) {
            return false;
        }
        
        // Check if position has adjacent paths
        let neighbors = [
            (pos.0.wrapping_sub(1), pos.1),
            (pos.0 + 1, pos.1),
            (pos.0, pos.1.wrapping_sub(1)),
            (pos.0, pos.1 + 1),
        ];
        
        neighbors.iter()
            .filter(|&&(x, y)| x < grid.width && y < grid.height)
            .any(|&pos| grid.has_vertex(pos))
    }

    pub(crate) fn manhattan_distance(a: Position, b: Position) -> usize {
        ((a.0 as i32 - b.0 as i32).abs() + (a.1 as i32 - b.1 as i32).abs()) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashSet, time::Instant};

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

    const EXAMPLE_MEDIUM: &str = "\
###########
#S..#.....#
#.#.#.###.#
#.#...#...#
#.#####.#.#
#.......#E#
###########";

    #[test]
    fn test_basic_shortcut_discovery() -> miette::Result<()> {
        let start_time = Instant::now();
        println!("\nStarting basic shortcut discovery test");

        // Setup grid
        let parsed_grid = parser::parse_input(EXAMPLE_SMALL)?;
        let grid = graph::create_grid(&parsed_grid)?;
        let path_grid = graph::create_pathfinding_grid(&grid);
        println!("Grid setup complete in {:?}", start_time.elapsed());

        // Find the path and its length
        let (start, end) = graph::find_endpoints(&parsed_grid)?;
        let original_length = pathing::find_shortest_path(&path_grid, start, end)?;
        println!("Original path length: {}", original_length);

        // Find candidates
        let candidates = shortcuts::find_candidates(&path_grid)?;
        println!("Found {} candidate positions", candidates.len());

        // Print first few candidates
        for pos in candidates.iter().take(5) {
            println!("Candidate at {:?}", pos);
        }

        Ok(())
    }

    #[test]
    fn test_shortcut_evaluation() -> miette::Result<()> {
        let start_time = Instant::now();
        println!("\nStarting shortcut evaluation test");

        // Setup
        let parsed_grid = parser::parse_input(EXAMPLE_LARGE)?;
        let grid = graph::create_grid(&parsed_grid)?;
        let path_grid = graph::create_pathfinding_grid(&grid);
        let (start, end) = graph::find_endpoints(&parsed_grid)?;

        // Get original path
        let original_length = pathing::find_shortest_path(&path_grid, start, end)?;
        println!("Original path length: {}", original_length);

        // Find and evaluate candidates
        let candidates = shortcuts::find_candidates(&path_grid)?;
        println!(
            "Found {} candidates in {:?}",
            candidates.len(),
            start_time.elapsed()
        );

        let improvements =
            shortcuts::evaluate_candidates(&path_grid, &candidates, start, end, original_length)?;
        println!(
            "Evaluated {} improvements in {:?}",
            improvements.len(),
            start_time.elapsed()
        );

        // Analyze improvements
        let mut improvements_vec: Vec<_> = improvements.iter().collect();
        improvements_vec.sort_by_key(|(_, &improvement)| std::cmp::Reverse(improvement));

        println!("\nTop 10 improvements:");
        for (pos, improvement) in improvements_vec.iter().take(10) {
            println!("Position {:?} improves by {} steps", pos, improvement);
        }

        Ok(())
    }

    #[test]
    fn test_specific_shortcuts() -> miette::Result<()> {
        println!("\nTesting specific known shortcuts");

        let parsed_grid = parser::parse_input(EXAMPLE_LARGE)?;
        let grid = graph::create_grid(&parsed_grid)?;
        let path_grid = graph::create_pathfinding_grid(&grid);
        let (start, end) = graph::find_endpoints(&parsed_grid)?;

        // Known shortcuts and their expected improvements
        let test_cases = [
            ((8, 1), 12), // Known to save 12 steps
                          // Add more known cases
        ];

        for (pos, expected) in test_cases {
            let original_length = pathing::find_shortest_path(&path_grid, start, end)?;
            let improvement =
                shortcuts::evaluate_shortcut(&path_grid, pos, start, end, original_length)?;

            println!("Shortcut at {:?}:", pos);
            println!("  Expected improvement: {}", expected);
            println!("  Actual improvement: {}", improvement);
            assert_eq!(
                improvement, expected,
                "Unexpected improvement for shortcut at {:?}",
                pos
            );
        }

        Ok(())
    }

    #[test]
    fn test_manhattan_radius() -> miette::Result<()> {
        let parsed_grid = parser::parse_input(EXAMPLE_SMALL)?;
        let grid = graph::create_grid(&parsed_grid)?;
        let path_grid = graph::create_pathfinding_grid(&grid);

        // Test points at various radii from a center point
        let center = (3, 3);
        println!(
            "\nTesting points at different Manhattan distances from {:?}",
            center
        );

        for radius in 1..=3 {
            let points = shortcuts::get_points_at_radius(&path_grid, center, radius);
            println!("\nRadius {}: found {} points", radius, points.len());
            println!("Points: {:?}", points);

            // Verify all points are actually at the correct Manhattan distance
            for pos in &points {
                let actual_distance = shortcuts::manhattan_distance(*pos, center);
                assert_eq!(
                    actual_distance, radius,
                    "Point {:?} is at distance {} but should be at distance {}",
                    pos, actual_distance, radius
                );
            }
        }

        Ok(())
    }

    #[test]
    fn test_process_large() -> miette::Result<()> {
        let start_time = Instant::now();
        println!("\nStarting large example test");

        let result = process(EXAMPLE_LARGE)?;

        // Expected results from the problem description
        let expected_counts = [
            (50, 32),
            (52, 31),
            (54, 29),
            (56, 39),
            (58, 25),
            (60, 23),
            (62, 20),
            (64, 19),
            (66, 12),
            (68, 14),
            (70, 12),
            (72, 22),
            (74, 4),
            (76, 3),
        ];

        println!("Processing complete in {:?}", start_time.elapsed());
        println!("Found {} total shortcuts", result);

        // TODO: Add detailed verification of improvement counts
        // for (improvement, expected_count) in expected_counts {
        //     println!("Shortcuts saving {} steps: {}", improvement, expected_count);
        // }

        assert_eq!(result, "285");
        Ok(())
    }

    #[test]
    fn test_medium_grid() -> miette::Result<()> {
        println!("\nTesting medium-sized grid");
        let parsed_grid = parser::parse_input(EXAMPLE_MEDIUM)?;
        let grid = graph::create_grid(&parsed_grid)?;
        let (start, end) = graph::find_endpoints(&parsed_grid)?;
        let path_grid = graph::create_pathfinding_grid(&grid);

        // Get original path
        let original_length = pathing::find_shortest_path(&path_grid, start, end)?;
        println!("Original path length: {}", original_length);

        // Find candidates
        let candidates = shortcuts::find_candidates(&path_grid)?;
        println!("Found {} candidates", candidates.len());

        // Debug each candidate
        let improvements = shortcuts::evaluate_candidates(&path_grid, &candidates, start, end, original_length)?;
        
        println!("\nSignificant improvements:");
        for (pos, improvement) in improvements.iter() {
            println!("Position {:?} improves by {} steps", pos, improvement);
        }

        Ok(())
    }

    fn visualize_grid(grid: &PathGrid, candidates: &HashSet<Position>) -> String {
        let mut output = String::new();
        for y in 0..grid.height {
            for x in 0..grid.width {
                let pos = (x, y);
                if grid.has_vertex(pos) {
                    output.push('#');
                } else if candidates.contains(&pos) {
                    output.push('*');
                } else {
                    output.push('.');
                }
            }
            output.push('\n');
        }
        output
    }

    #[test]
    fn test_process_large_debug() -> miette::Result<()> {
        let start = Instant::now();
        println!("\nStarting large example debug test");
        
        let parsed_grid = parser::parse_input(EXAMPLE_LARGE)?;
        let grid = graph::create_grid(&parsed_grid)?;
        let (start_pos, end_pos) = graph::find_endpoints(&parsed_grid)?;
        let path_grid = graph::create_pathfinding_grid(&grid);
        
        println!("Grid dimensions: {}x{}", path_grid.width, path_grid.height);
        
        let candidates = shortcuts::find_candidates(&path_grid)?;
        println!("Found {} candidates", candidates.len());
        
        let original_length = pathing::find_shortest_path(&path_grid, start_pos, end_pos)?;
        println!("Original path length: {}", original_length);
        
        let improvements = shortcuts::evaluate_candidates(
            &path_grid,
            &candidates,
            start_pos,
            end_pos,
            original_length
        )?;
        
        println!("\nFound {} improvements:", improvements.len());
        for (pos, improvement) in improvements.iter().take(10) {
            println!("Position {:?} improves by {} steps", pos, improvement);
        }
        
        println!("\nProcessing time: {:?}", start.elapsed());
        Ok(())
    }
}
