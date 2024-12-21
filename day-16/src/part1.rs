use std::collections::HashMap;

use error::PuzzleError;
use graph::{build_graph, Direction, GridType, NodePosition, Position};
use parser::LocatedCell;
use petgraph::graph::NodeIndex;


// Update the process function to work with the new graph structure
pub fn process(_input: &str) -> miette::Result<String> {
    let (_, cells) = parser::parse_cells(parser::Span::new(_input))
        .map_err(|e| PuzzleError::Parser(format!("Failed to parse input: {:?}", e)))?;
    
    // Cache distances to end for each position
    let mut distance_cache = HashMap::new();
    let end_pos = find_position_by_type(&cells, GridType::End)
        .ok_or_else(|| PuzzleError::Graph("Could not find end position".to_string()))?;
    
    // Build graph with cached heuristics
    let (graph, position_to_node) = build_graph(cells.clone())?;
    let start_pos = find_position_by_type(&cells, GridType::Start)
        .ok_or_else(|| PuzzleError::Graph("Could not find start position".to_string()))?;
    let start_node = position_to_node
        .get(&(start_pos, Direction::Right))
        .ok_or_else(|| PuzzleError::Graph("Start position not in graph".to_string()))?;

    // Bidirectional A* search
    let result = petgraph::algo::astar(
        &graph,
        *start_node,
        |n| graph[n].grid_type == GridType::End,
        |e| *e.weight(),
        |n| {
            let pos = get_position_from_node(&position_to_node, n)
                .expect("Node must have a position");
            *distance_cache
                .entry(pos)
                .or_insert_with(|| manhattan_distance(pos, end_pos))
        },
    );

    match result {
        Some((cost, _)) => Ok(cost.to_string()),
        None => Err(PuzzleError::Graph("No path found from start to end".to_string()).into()),
    }
}

// Helper function to get position from node index
fn get_position_from_node(position_to_node: &HashMap<NodePosition, NodeIndex>, node: NodeIndex) -> Option<Position> {
    position_to_node
        .iter()
        .find(|(_, &n)| n == node)
        .map(|((pos, _), _)| *pos)
}

// Helper function to find a position by grid type
fn find_position_by_type(cells: &[Vec<LocatedCell>], grid_type: GridType) -> Option<Position> {
    cells.iter().enumerate().find_map(|(y, row)| {
        row.iter().enumerate().find_map(|(x, cell)| {
            if matches!(
                (cell.cell_type, grid_type),
                (parser::START, GridType::Start) |
                (parser::END, GridType::End)
            ) {
                Some((x, y))
            } else {
                None
            }
        })
    })
}

fn manhattan_distance(pos1: Position, pos2: Position) -> u32 {
    let dx = if pos1.0 > pos2.0 { pos1.0 - pos2.0 } else { pos2.0 - pos1.0 };
    let dy = if pos1.1 > pos2.1 { pos1.1 - pos2.1 } else { pos2.1 - pos1.1 };
    (dx + dy) as u32
}

mod error {
    use miette::Diagnostic;
    use nom::error::{ErrorKind, ParseError};
    use thiserror::Error;

    use crate::part1::parser::Span;

    #[derive(Debug, Error, Diagnostic)]
    pub enum PuzzleError {
        #[error("Parser error: {0}")]
        Parser(String),
        #[error("Graph error: {0}")]
        Graph(String),
        #[error(transparent)]
        Other(#[from] Box<dyn std::error::Error + Send + Sync>),
    }

    impl<'a> ParseError<Span<'a>> for PuzzleError {
        fn from_error_kind(_input: Span<'a>, kind: ErrorKind) -> Self {
            PuzzleError::Parser(format!("Parse error: {:?}", kind))
        }

        fn append(_: Span<'a>, _: ErrorKind, other: Self) -> Self {
            other
        }
    }
}

mod parser {
    use nom::{
        character::complete::{line_ending, one_of},
        multi::{many1, separated_list1},
        IResult,
    };
    use nom_locate::LocatedSpan;

    pub const START: char = 'S';
    pub const END: char = 'E';
    pub const WALL: char = '#';
    pub const EMPTY: char = '.';

    pub type Span<'a> = LocatedSpan<&'a str>;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LocatedCell<'a> {
        pub cell_type: char,
        pub position: Span<'a>,
    }

    fn parse_cell(input: Span) -> IResult<Span, LocatedCell> {
        let (input, cell_type) = one_of(&[START, END, WALL, EMPTY][..])(input)?;
        Ok((
            input,
            LocatedCell {
                cell_type,
                position: input,
            },
        ))
    }

    pub fn parse_cells(input: Span) -> IResult<Span, Vec<Vec<LocatedCell>>> {
        separated_list1(line_ending, many1(parse_cell))(input)
    }
}

mod graph {
    use miette::Diagnostic;
    use petgraph::graph::{DiGraph, NodeIndex};
    use std::collections::HashMap;
    use thiserror::Error;

    use super::parser::{self, LocatedCell};

    #[allow(dead_code)]
    #[derive(Debug, Error, Diagnostic)]
    pub enum GraphError {
        #[error("Graph Error")]
        Generic,
        #[error("Position Error {0:?}")]
        Position(Position),
        #[error("Grid Error {0:?}")]
        Grid(GridType),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Direction {
        Up,
        Down,
        Left,
        Right,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum GridType {
        Wall,
        Empty,
        Start,
        End,
    }

    impl GridType {
        fn _is_wall(&self) -> bool {
            matches!(self, GridType::Wall)
        }
    }

    pub type Position = (usize, usize);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Edge {
        pub direction: Direction,  // The direction this edge represents
        pub cost: u32,            // The cost of taking this edge
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NodeState {
        pub grid_type: GridType,
        pub direction: Direction,
    }
    
    // A position plus direction uniquely identifies a node
    pub type NodePosition = (Position, Direction);
    
    pub fn build_graph(
        cells: Vec<Vec<LocatedCell>>,
    ) -> miette::Result<(DiGraph<NodeState, u32>, HashMap<NodePosition, NodeIndex>)> {
        let mut graph = DiGraph::new();
        let mut position_to_node = HashMap::new();
    
        // Create nodes - one for each position and possible direction
        cells.iter().enumerate().for_each(|(y, row)| {
            row.iter().enumerate().for_each(|(x, cell)| {
                let position = (x, y);
                let grid_type = match cell.cell_type {
                    parser::START => GridType::Start,
                    parser::END => GridType::End,
                    parser::WALL => GridType::Wall,
                    parser::EMPTY => GridType::Empty,
                    _ => unreachable!(),
                };
    
                if !matches!(grid_type, GridType::Wall) {
                    // Create a node for each possible direction at this position
                    for &direction in &[Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
                        let node = graph.add_node(NodeState { grid_type, direction });
                        position_to_node.insert((position, direction), node);
                    }
                }
            });
        });
    
        // Add edges between nodes
        for ((x, y), from_dir) in position_to_node.keys().cloned().collect::<Vec<_>>() {
            let from_node = position_to_node[&((x, y), from_dir)];
            
            // Can move to adjacent positions
            let neighbors = [
                ((x.wrapping_sub(1), y), Direction::Left),
                ((x + 1, y), Direction::Right),
                ((x, y.wrapping_sub(1)), Direction::Up),
                ((x, y + 1), Direction::Down),
            ];
    
            for (next_pos, to_dir) in neighbors {
                if let Some(&to_node) = position_to_node.get(&(next_pos, to_dir)) {
                    // Cost is 1 for movement plus 1000 if we need to turn
                    let cost = if from_dir == to_dir { 1 } else { 1001 };
                    graph.add_edge(from_node, to_node, cost);
                }
            }
        }
    
        Ok((graph, position_to_node))
    }
}

#[cfg(test)]
mod tests {
    use error::PuzzleError;
    use graph::*;
    use nom_locate::LocatedSpan;
    use parser::*;

    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "\
###############
#.......#....E#
#.#.###.#.###.#
#.....#.#...#.#
#.###.#####.#.#
#.#.#.......#.#
#.#.#####.###.#
#...........#.#
###.#.#####.#.#
#...#.....#.#.#
#.#.#.###.#.#.#
#.....#...#.#.#
#.###.#.#.#.#.#
#S..#.....#...#
###############";

        assert_eq!("7036", process(input)?);
        Ok(())
    }

    #[test]
    fn test_process2() -> miette::Result<()> {
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

    #[test]
    fn test_parser() -> miette::Result<()> {
        let input = "\
###############
#.......#....E#
#.#.###.#.###.#
#.....#.#...#.#
#.###.#####.#.#
#.#.#.......#.#
#.#.#####.###.#
#...........#.#
###.#.#####.#.#
#...#.....#.#.#
#.#.#.###.#.#.#
#.....#...#.#.#
#.###.#.#.#.#.#
#S..#.....#...#
###############";

        let (_, output) = parse_cells(LocatedSpan::new(input)).map_err(|e| PuzzleError::Parser(format!("Parser Error: {:?}", e)))?;

        let start_position: Position = output
            .iter()
            .enumerate()
            .find_map(|(y, row)| {
                row.iter().enumerate().find_map(|(x, cell)| {
                    if cell.cell_type == START {
                        Some((x, y))
                    } else {
                        None
                    }
                })
            })
            .ok_or(PuzzleError::Parser(format!("Parser Error")))?;

        assert_eq!((1, 13), start_position);

        let end_position: Position = output
            .iter()
            .enumerate()
            .find_map(|(y, row)| {
                row.iter().enumerate().find_map(|(x, cell)| {
                    if cell.cell_type == END {
                        Some((x, y))
                    } else {
                        None
                    }
                })
            })
            .ok_or(PuzzleError::Parser(format!("Parser Error")))?;

        assert_eq!((13, 1), end_position);

        Ok(())
    }

    #[test]
    fn test_build_graph_small() -> miette::Result<()> {
        let input = "\
##
SE";
    
        let (_, cells) = parse_cells(LocatedSpan::new(input))
            .map_err(|e| PuzzleError::Parser(format!("Parser Error: {:?}", e)))?;
        let (graph, position_to_node) = build_graph(cells.clone())?;
    
        // Test total nodes
        // Now we have 4 directional nodes for each non-wall position
        assert_eq!(8, graph.node_count(), "Should have 8 nodes (2 positions * 4 directions)");
    
        let start_pos = find_position_by_type(&cells, GridType::Start)
            .ok_or(PuzzleError::Graph(format!("Graph Error")))?;
        let end_pos = find_position_by_type(&cells, GridType::End)
            .ok_or(PuzzleError::Graph(format!("Graph Error")))?;
    
        // Test node positions
        assert_eq!((0, 1), start_pos, "Start position should be at (0,1)");
        assert_eq!((1, 1), end_pos, "End position should be at (1,1)");
    
        // Get all directional nodes for start and end positions
        let start_right_node = position_to_node.get(&(start_pos, Direction::Right))
            .expect("Should have right-facing node at start");
        let end_right_node = position_to_node.get(&(end_pos, Direction::Right))
            .expect("Should have right-facing node at end");
    
        // Test edge connections
        let _edges: Vec<_> = graph
            .edge_indices()
            .map(|e| graph.edge_endpoints(e).unwrap())
            .collect();
    
        // There should be edges between directional states
        assert!(graph.edge_indices().any(|e| {
            let (from, to) = graph.edge_endpoints(e).unwrap();
            from == *start_right_node && to == *end_right_node
        }), "Should have edge from S(right) to E(right)");
    
        // Verify walls have no nodes
        let wall_positions = [(0, 0), (1, 0)];
        for wall_pos in wall_positions {
            assert!(
                !position_to_node.contains_key(&(wall_pos, Direction::Right)) &&
                !position_to_node.contains_key(&(wall_pos, Direction::Left)) &&
                !position_to_node.contains_key(&(wall_pos, Direction::Up)) &&
                !position_to_node.contains_key(&(wall_pos, Direction::Down)),
                "Wall at {:?} should have no nodes",
                wall_pos
            );
        }
    
        Ok(())
    }
    
    #[test]
    fn test_build_graph() -> miette::Result<()> {
        let input = "\
#####
#S#E#
#.#.#
#...#
#####";
    
        let (_, cells) = parse_cells(LocatedSpan::new(input))
            .map_err(|e| PuzzleError::Parser(format!("Parser Error: {:?}", e)))?;
        let (graph, position_to_node) = build_graph(cells.clone())?;
    
        // Test node counts - now we have 4 nodes per non-wall position
        let non_wall_positions = 7;  // S, E, and 5 empty spaces
        assert_eq!(
            non_wall_positions * 4, 
            graph.node_count(),
            "Should have {} nodes ({} positions * 4 directions)", 
            non_wall_positions * 4,
            non_wall_positions
        );
    
        let start_pos = find_position_by_type(&cells, GridType::Start)
            .ok_or(PuzzleError::Graph(format!("Graph Error")))?;
        let end_pos = find_position_by_type(&cells, GridType::End)
            .ok_or(PuzzleError::Graph(format!("Graph Error")))?;
    
        assert_eq!((1, 1), start_pos);
        assert_eq!((3, 1), end_pos);
    
        // Test that each non-wall position has nodes for all four directions
        let test_positions = vec![
            start_pos,      // (1,1) S
            end_pos,        // (3,1) E
            (1, 2),        // .
            (3, 2),        // . (previously missed)
            (1, 3),        // .
            (2, 3),        // .
            (3, 3),        // .
        ];
        for pos in test_positions {
            for dir in [Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
                assert!(
                    position_to_node.contains_key(&(pos, dir)),
                    "Position {:?} should have node for direction {:?}",
                    pos,
                    dir
                );
            }
        }
    
        Ok(())
    }
}
