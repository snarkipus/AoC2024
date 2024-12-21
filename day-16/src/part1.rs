use graph::FastGraph;
use types::{CellType, Direction, Position};

pub fn process(input: &str) -> miette::Result<String> {
    let grid = parser::parse_grid(input)?;
    let (width, height) = grid.dimensions();
    let mut fast_graph = FastGraph::new(width, height);

    // Create nodes
    for (pos, cell_type) in grid.iter_positions() {
        if cell_type != CellType::Wall {
            for dir in Direction::all() {
                fast_graph.add_node(pos, cell_type, dir);
            }
        }
    }

    // Add edges
    fast_graph.add_edges();

    let start_pos = grid.find_special_cell(CellType::Start)?;
    let end_pos = grid.find_special_cell(CellType::End)?;

    // Get starting node (facing right)
    let start_node = fast_graph
        .get_node(start_pos, Direction::Right)
        .ok_or(error::PuzzleError::InvalidPosition(start_pos))?;

    // Use A* to find shortest path
    let result = petgraph::algo::astar(
        &fast_graph.graph,
        start_node,
        |n| fast_graph.graph[n].cell_type == CellType::End,
        |e| *e.weight(),
        |n| manhattan_distance(fast_graph.graph[n].pos, end_pos),
    );

    result
        .map(|(cost, _)| cost.to_string())
        .ok_or_else(|| error::PuzzleError::NoPath.into())
}

fn manhattan_distance(pos1: Position, pos2: Position) -> u32 {
    (pos1.x().abs_diff(pos2.x()) + pos1.y().abs_diff(pos2.y())) as u32
}

mod types {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Position(u32);

    impl Position {
        pub fn new(x: usize, y: usize) -> Self {
            debug_assert!(
                x < (1 << 16) && y < (1 << 16),
                "Position coordinates must fit in 16 bits"
            );
            Self(((y as u32) << 16) | (x as u32))
        }

        pub fn x(&self) -> usize {
            (self.0 & 0xFFFF) as usize
        }

        pub fn y(&self) -> usize {
            (self.0 >> 16) as usize
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Direction {
        Up,
        Down,
        Left,
        Right,
    }

    impl Direction {
        pub const fn as_index(self) -> usize {
            match self {
                Self::Up => 0,
                Self::Down => 1,
                Self::Left => 2,
                Self::Right => 3,
            }
        }

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

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CellType {
        Start,
        End,
        Wall,
        Empty,
    }

    impl TryFrom<char> for CellType {
        type Error = crate::part1::error::PuzzleError;

        fn try_from(c: char) -> Result<Self, Self::Error> {
            match c {
                'S' => Ok(Self::Start),
                'E' => Ok(Self::End),
                '#' => Ok(Self::Wall),
                '.' => Ok(Self::Empty),
                _ => Err(crate::part1::error::PuzzleError::InvalidCell(c)),
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct NodeState {
        pub pos: Position,
        pub cell_type: CellType,
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
    use crate::part1::types::*;
    use petgraph::graph::{DiGraph, NodeIndex};

    const MOVEMENT_COST: u32 = 1;

    pub struct FastGraph {
        // Core graph for pathfinding
        pub graph: DiGraph<NodeState, u32>,
        // Fast lookup from position+direction to node index
        nodes: Vec<Option<NodeIndex>>,
        width: usize,
        height: usize,
    }

    impl FastGraph {
        pub fn new(width: usize, height: usize) -> Self {
            let size = width * height * 4; // 4 directions per position
            Self {
                graph: DiGraph::new(),
                nodes: vec![None; size],
                width,
                height,
            }
        }

        fn get_index(&self, pos: Position, dir: Direction) -> usize {
            (pos.y() * self.width + pos.x()) * 4 + dir.as_index()
        }

        pub fn add_node(
            &mut self,
            pos: Position,
            cell_type: CellType,
            direction: Direction,
        ) -> NodeIndex {
            let state = NodeState { pos, cell_type };
            let node_idx = self.graph.add_node(state);
            let idx = self.get_index(pos, direction);
            self.nodes[idx] = Some(node_idx);
            node_idx
        }

        pub fn get_node(&self, pos: Position, dir: Direction) -> Option<NodeIndex> {
            let idx = self.get_index(pos, dir);
            self.nodes.get(idx).copied().flatten()
        }

        pub fn add_edges(&mut self) {
            let mut edges = Vec::new();

            // Collect all edges first
            for y in 0..self.height {
                for x in 0..self.width {
                    let pos = Position::new(x, y);
                    for from_dir in Direction::all() {
                        if let Some(from_idx) = self.get_node(pos, from_dir) {
                            // Try all possible moves from this position
                            let possible_moves = get_possible_moves(pos, self.width, self.height);
                            for (next_pos, to_dir) in possible_moves {
                                if let Some(to_idx) = self.get_node(next_pos, to_dir) {
                                    let cost = MOVEMENT_COST + from_dir.turn_cost(to_dir);
                                    edges.push((from_idx, to_idx, cost));
                                }
                            }
                        }
                    }
                }
            }

            // Add all edges at once
            for (from, to, cost) in edges {
                self.graph.add_edge(from, to, cost);
            }
        }
    }

    fn get_possible_moves(
        pos: Position,
        width: usize,
        height: usize,
    ) -> Vec<(Position, Direction)> {
        let mut moves = Vec::with_capacity(4);

        if pos.x() > 0 {
            moves.push((Position::new(pos.x() - 1, pos.y()), Direction::Left));
        }
        if pos.x() + 1 < width {
            moves.push((Position::new(pos.x() + 1, pos.y()), Direction::Right));
        }
        if pos.y() > 0 {
            moves.push((Position::new(pos.x(), pos.y() - 1), Direction::Up));
        }
        if pos.y() + 1 < height {
            moves.push((Position::new(pos.x(), pos.y() + 1), Direction::Down));
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
