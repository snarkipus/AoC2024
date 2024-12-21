use miette::Result;
use types::{CellType, Direction, Position};

pub fn process(input: &str) -> Result<String> {
    let cells = parser::parse_grid(input)?;
    let (graph, position_to_node) = graph::build_graph(&cells)?;

    let start_pos = cells.find_special_cell(CellType::Start)?;
    let end_pos = cells.find_special_cell(CellType::End)?;

    // Start facing right (arbitrary but consistent choice)
    let start_node = position_to_node
        .get(&(start_pos, Direction::Right))
        .ok_or(error::PuzzleError::InvalidPosition(start_pos))?;

    // Find shortest path to any node at end position
    let result = petgraph::algo::astar(
        &graph,
        *start_node,
        |n| graph[n].cell_type == CellType::End,
        |e| *e.weight(),
        |n| {
            manhattan_distance(
                position_to_node
                    .iter()
                    .find(|(_, &node)| node == n)
                    .map(|((pos, _), _)| *pos)
                    .expect("All nodes must have positions"),
                end_pos,
            )
        },
    );

    result
        .map(|(cost, _)| cost.to_string())
        .ok_or_else(|| error::PuzzleError::NoPath.into())
}

fn manhattan_distance(pos1: Position, pos2: Position) -> u32 {
    ((pos1.x.abs_diff(pos2.x)) + (pos1.y.abs_diff(pos2.y))) as u32
}

mod types {
    use crate::part1::error;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Position {
        pub x: usize,
        pub y: usize,
    }

    impl Position {
        pub fn new(x: usize, y: usize) -> Self {
            Self { x, y }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Direction {
        Up,
        Down,
        Left,
        Right,
    }

    impl Direction {
        pub fn all() -> [Direction; 4] {
            [Self::Up, Self::Down, Self::Left, Self::Right]
        }

        pub fn turn_cost(&self, new_direction: Direction) -> u32 {
            if *self == new_direction {
                0
            } else {
                1000
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum CellType {
        Start,
        End,
        Wall,
        Empty,
    }

    impl TryFrom<char> for CellType {
        type Error = error::PuzzleError;

        fn try_from(c: char) -> Result<Self, Self::Error> {
            match c {
                'S' => Ok(Self::Start),
                'E' => Ok(Self::End),
                '#' => Ok(Self::Wall),
                '.' => Ok(Self::Empty),
                _ => Err(error::PuzzleError::InvalidCell(c)),
            }
        }
    }

    // A node in our graph represents a position and facing direction
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct NodeState {
        pub cell_type: CellType,
        pub direction: Direction,
    }
}

mod error {
    use crate::part1::types::{CellType, Position};
    use miette::Diagnostic;
    use thiserror::Error;

    #[derive(Debug, Error, Diagnostic)]
    pub enum PuzzleError {
        #[error("Failed to parse input: {0}")]
        Parser(String),

        #[error("Invalid cell character: {0}")]
        InvalidCell(char),

        #[error("Invalid position: {0:?}")]
        InvalidPosition(Position),

        #[error("Could not find cell of type {0:?}")]
        CellNotFound(CellType),

        #[error("No path found from start to end")]
        NoPath,
    }
}

mod parser {
    use crate::part1::{
        error::PuzzleError,
        types::{CellType, Position},
    };
    use nom::{
        character::complete::{line_ending, one_of},
        combinator::map_res,
        multi::{many1, separated_list1},
    };

    #[derive(Debug, Clone)]
    pub struct Grid {
        cells: Vec<Vec<CellType>>,
    }

    impl Grid {
        pub fn _get(&self, pos: Position) -> Option<CellType> {
            self.cells
                .get(pos.y)
                .and_then(|row| row.get(pos.x))
                .copied()
        }

        pub fn dimensions(&self) -> (usize, usize) {
            let height = self.cells.len();
            let width = self.cells.first().map_or(0, |row| row.len());
            (width, height)
        }

        pub fn find_special_cell(&self, target: CellType) -> Result<Position, PuzzleError> {
            for (y, row) in self.cells.iter().enumerate() {
                for (x, &cell) in row.iter().enumerate() {
                    if cell == target {
                        return Ok(Position::new(x, y));
                    }
                }
            }
            Err(PuzzleError::CellNotFound(target))
        }

        pub fn iter_positions(&self) -> impl Iterator<Item = (Position, CellType)> + '_ {
            self.cells.iter().enumerate().flat_map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .map(move |(x, &cell)| (Position::new(x, y), cell))
            })
        }
    }

    pub fn parse_grid(input: &str) -> Result<Grid, PuzzleError> {
        let (_, cells) = separated_list1::<_, _, _, nom::error::Error<&str>, _, _>(
            line_ending,
            many1(map_res(one_of("SE#."), CellType::try_from)),
        )(input)
        .map_err(|e| PuzzleError::Parser(e.to_string()))?;

        Ok(Grid { cells })
    }
}

mod graph {
    use petgraph::graph::{DiGraph, NodeIndex};
    use std::collections::HashMap;

    use crate::part1::{
        error::PuzzleError,
        types::{CellType, Direction, NodeState, Position},
    };

    pub type Graph = DiGraph<NodeState, u32>;
    pub type NodeLookup = HashMap<(Position, Direction), NodeIndex>;

    const MOVEMENT_COST: u32 = 1;

    pub fn build_graph(
        grid: &crate::part1::parser::Grid,
    ) -> Result<(Graph, NodeLookup), PuzzleError> {
        let mut graph = Graph::new();
        let mut position_to_node = NodeLookup::new();

        // Create nodes for each position/direction combination
        for (pos, cell_type) in grid.iter_positions() {
            if cell_type != CellType::Wall {
                for direction in Direction::all() {
                    let node = graph.add_node(NodeState {
                        cell_type,
                        direction,
                    });
                    position_to_node.insert((pos, direction), node);
                }
            }
        }

        // Add edges between nodes
        let (width, height) = grid.dimensions();
        for ((pos, from_dir), &from_idx) in position_to_node.iter() {
            // Check all possible moves from current position
            let possible_moves = get_possible_moves(*pos, width, height);

            for (next_pos, to_dir) in possible_moves {
                if let Some(&to_idx) = position_to_node.get(&(next_pos, to_dir)) {
                    let turn_cost = from_dir.turn_cost(to_dir);
                    graph.add_edge(from_idx, to_idx, MOVEMENT_COST + turn_cost);
                }
            }
        }

        Ok((graph, position_to_node))
    }

    fn get_possible_moves(
        pos: Position,
        width: usize,
        height: usize,
    ) -> Vec<(Position, Direction)> {
        let mut moves = Vec::new();

        // Try all possible moves, checking bounds
        if pos.x > 0 {
            moves.push((Position::new(pos.x - 1, pos.y), Direction::Left));
        }
        if pos.x + 1 < width {
            moves.push((Position::new(pos.x + 1, pos.y), Direction::Right));
        }
        if pos.y > 0 {
            moves.push((Position::new(pos.x, pos.y - 1), Direction::Up));
        }
        if pos.y + 1 < height {
            moves.push((Position::new(pos.x, pos.y + 1), Direction::Down));
        }

        moves
    }
}

#[cfg(test)]
mod tests {
    use crate::part1::process;

    #[test]
    fn test_simple_path() -> miette::Result<()> {
        let input = "\
##
SE";
        assert_eq!("1", process(input)?);
        Ok(())
    }

    #[test]
    fn test_with_turn() -> miette::Result<()> {
        let input = "\
###
#S#
#.#
#E#
###";
        assert_eq!("1002", process(input)?);
        Ok(())
    }

    #[test]
    fn test_complex_maze() -> miette::Result<()> {
        let input = "\
#################
#...#...#...#..E#
#.#.#.#.#.#.#.#.#
#.#.#.#...#...#.#
#.#.#.#.###.#.#.#
#...#.#.#.....#.#
#.#.#.#.#.#####.#
#.#...#.#.#.....#
#.#.#####.#.###.#
#.#.#.......#...#
#.#.###.#####.###
#.#.#...#.....#.#
#.#.#.#####.###.#
#.#.#.........#.#
#.#.#.#########.#
#S#.............#
#################";

        assert_eq!("11048", process(input)?);
        Ok(())
    }
}
