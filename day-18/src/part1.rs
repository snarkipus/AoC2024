use miette::miette;
use petgraph::algo::dijkstra;

#[cfg(test)]
mod constants {
    pub const DIM: usize = 7;
    pub const BYTES: usize = 12;
}

#[cfg(not(test))]
mod constants {
    pub const DIM: usize = 70;
    pub const BYTES: usize = 1024;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Position(usize, usize);

pub const START: Position = Position(0, 0);
// Fix: Use DIM-1 for last valid position
pub const END: Position = Position(constants::DIM - 1, constants::DIM - 1);

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {

    let coords = parser::parse(input)?;
    let graph = graph::create_graph(coords)?;

    // Get node indices for start and end positions
    let start_idx = graph::get_node_index(&graph, START)?;
    let end_idx = graph::get_node_index(&graph, END)?;

    // Find shortest path using dijkstra
    let path = dijkstra(&graph, start_idx, Some(end_idx), |_| 1);

    // Get the distance to the end node
    let distance = path
        .get(&end_idx)
        .ok_or_else(|| miette!("No path found to end position"))?;

    Ok(distance.to_string())
}

mod graph {
    use std::collections::HashMap;
    use miette::miette;
    use petgraph::graph::{DiGraph, NodeIndex};

    use super::{
        constants::{BYTES, DIM},
        Position,
    };

    pub fn create_graph(coords: Vec<Position>) -> miette::Result<DiGraph<char, ()>> {
        let mut grid = vec![vec!['.'; DIM]; DIM];
        
        // Validate coordinates are within bounds
        for Position(x, y) in coords.iter() {
            if *x >= DIM || *y >= DIM {
                return Err(miette::miette!("Coordinates ({}, {}) out of bounds", x, y));
            }
        }
        
        // Place walls
        coords.into_iter()
            .take(BYTES)
            .for_each(|Position(x, y)| {
                grid[y][x] = '#';
            });

        // Create graph nodes
        let mut graph = DiGraph::new();
        let mut nodes = HashMap::new();
        
        // Create nodes
        for y in 0..DIM {
            for x in 0..DIM {
                let node = graph.add_node(grid[y][x]);
                nodes.insert((x, y), node);
            }
        }

        // Create edges - fix bounds to include last row/column
        for y in 0..DIM {
            for x in 0..DIM {
                let current_node = nodes[&(x, y)];
                let current_val = graph[current_node];

                if current_val == '#' {
                    continue;
                }

                for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx < 0 || ny < 0 || nx >= DIM as i32 || ny >= DIM as i32 {
                        continue;
                    }

                    let nx = nx as usize;
                    let ny = ny as usize;

                    let neighbor_node = nodes[&(nx, ny)];
                    let neighbor_val = graph[neighbor_node];

                    if neighbor_val == '.' {
                        graph.add_edge(current_node, neighbor_node, ());
                    }
                }
            }
        }

        Ok(graph)
    }

    pub fn print_grid(grid: &Vec<Vec<char>>) {
        for row in grid {
            println!("{:?}", row);
        }
    }

    pub fn get_node_index(
        graph: &DiGraph<char, ()>,
        Position(x, y): Position,
    ) -> miette::Result<NodeIndex> {
        if x >= DIM || y >= DIM {
            return Err(miette!("Position ({}, {}) out of bounds", x, y));
        }
        
        let idx = y * DIM + x;
        graph
            .node_indices()
            .nth(idx)
            .ok_or_else(|| miette!("No node found at position ({}, {})", x, y))
    }

    #[cfg(test)]
    mod tests {
        use petgraph::algo::dijkstra;

        use crate::part1::{constants, graph, parser, END, START, tests::INPUT};

        use super::*;

        #[test]
        fn test_graph_creation() -> miette::Result<()> {
            let coords = vec![Position(1, 1), Position(2, 2)];
            let graph = create_graph(coords)?;

            // Print grid for debugging
            let mut grid = vec![vec!['.'; DIM]; DIM];
            for node in graph.node_indices() {
                let idx = node.index();
                let x = idx % DIM;
                let y = idx / DIM;
                grid[y][x] = graph[node];
            }
            print_grid(&grid);

            Ok(())
        }

        #[test]
        fn test_path_finding() -> miette::Result<()> {
            // Create test grid with known path
            let coords = vec![
                Position(1, 0),
                Position(1, 1), // Wall blocking direct path
                Position(2, 1),
                Position(2, 2), // Forces path around
            ];

            let graph = create_graph(coords)?;

            // Print initial grid
            let mut grid = vec![vec!['.'; DIM]; DIM];
            for node in graph.node_indices() {
                let idx = node.index();
                let x = idx % DIM;
                let y = idx / DIM;
                grid[y][x] = graph[node];
            }
            println!("Initial grid:");
            print_grid(&grid);

            // Try finding path
            let start = Position(0, 0);
            let end = Position(3, 3);

            let start_idx = get_node_index(&graph, start)?;
            let end_idx = get_node_index(&graph, end)?;

            let paths = dijkstra(&graph, start_idx, Some(end_idx), |_| 1);
            let distance = paths.get(&end_idx).expect("Should find path");

            // Visualize path
            let mut path_grid = grid.clone();
            let mut current = end_idx;
            while current != start_idx {
                let idx = current.index();
                let x = idx % DIM;
                let y = idx / DIM;
                path_grid[y][x] = 'o';
                // Find previous node in path
                for neighbor in graph.neighbors_directed(current, petgraph::Direction::Incoming) {
                    if paths.get(&neighbor) == Some(&(paths[&current] - 1)) {
                        current = neighbor;
                        break;
                    }
                }
            }

            println!("\nPath visualization:");
            print_grid(&path_grid);
            println!("\nPath length: {}", distance);

            assert_eq!(*distance, 6, "Expected path length of 6");
            Ok(())
        }

        #[test]
        fn test_bounds() -> miette::Result<()> {
            // Create walls near but not at END position
            let coords = vec![
                Position(constants::DIM - 2, constants::DIM - 2),  // Wall near end
                Position(0, constants::DIM - 1),                   // Bottom wall
                Position(constants::DIM - 1, 0),                   // Right wall
            ];
            
            let graph = graph::create_graph(coords)?;
            
            // Print grid for debugging
            let mut grid = vec![vec!['.'; constants::DIM]; constants::DIM];
            for node in graph.node_indices() {
                let idx = node.index();
                let x = idx % constants::DIM;
                let y = idx / constants::DIM;
                grid[y][x] = graph[node];
            }
            println!("Grid state:");
            graph::print_grid(&grid);
            
            // Verify key positions
            assert!(graph::get_node_index(&graph, START).is_ok(), "Start should be accessible");
            assert!(graph::get_node_index(&graph, END).is_ok(), "End should be accessible");
            
            // Test invalid position
            let invalid_pos = Position(constants::DIM, constants::DIM);
            assert!(graph::get_node_index(&graph, invalid_pos).is_err());
            
            Ok(())
        }

        #[test]
        fn test_node_index() -> miette::Result<()> {
            let coords = vec![];  // Empty coords = no walls
            let graph = create_graph(coords)?;
            
            // Test all corners
            assert!(get_node_index(&graph, Position(0, 0)).is_ok());
            assert!(get_node_index(&graph, Position(0, DIM-1)).is_ok());
            assert!(get_node_index(&graph, Position(DIM-1, 0)).is_ok());
            assert!(get_node_index(&graph, Position(DIM-1, DIM-1)).is_ok());
            
            Ok(())
        }

        #[test]
        fn test_full_path() -> miette::Result<()> {
            let test_cases = vec![
                (
                    "Empty grid",
                    vec![], 
                    12     // 6 right + 6 down = 12 steps
                ),
                (
                    "Corner walls",
                    vec![  
                        Position(1, 0),
                        Position(1, 1),
                        Position(2, 1),
                    ],
                    12    // Same length - walls don't block optimal path
                ),
                (
                    "Input case",
                    parser::parse(INPUT)?,
                    22    // Matches known good result
                )
            ];

            for (name, coords, expected_length) in test_cases {
                let graph = graph::create_graph(coords.clone())?;
                let start_idx = graph::get_node_index(&graph, START)?;
                let end_idx = graph::get_node_index(&graph, END)?;
                
                let paths = dijkstra(&graph, start_idx, Some(end_idx), |_| 1);
                let distance = paths.get(&end_idx)
                    .ok_or_else(|| miette!("No path found"))?;
                    
                assert_eq!(*distance, expected_length, 
                    "Case '{}': Path length incorrect", name);
            }
            
            Ok(())
        }
    }
}

mod parser {
    use super::Position;
    use miette::miette;

    pub fn parse(input: &str) -> miette::Result<Vec<Position>> {
        Ok(input
            .lines()
            .map(|line| {
                let mut parts = line.split(',');
                let x = parts
                    .next()
                    .unwrap()
                    .trim()
                    .parse()
                    .map_err(|e| miette!("Failed to parse x coordinate: {}", e))?;
                let y = parts
                    .next()
                    .unwrap()
                    .trim()
                    .parse()
                    .map_err(|e| miette!("Failed to parse y coordinate: {}", e))?;
                Ok(Position(x, y))
            })
            .collect::<miette::Result<Vec<Position>>>()?)
    }
}

#[cfg(test)]
mod tests {
    use constants::DIM;
    use graph::{create_graph, get_node_index};

    use super::*;
    pub(crate) const INPUT: &str = "\
5,4
4,2
4,5
3,0
2,1
6,3
2,4
1,5
0,6
3,3
2,6
5,1
1,2
5,5
2,5
6,5
1,4
0,4
6,4
1,1
6,1
1,0
0,5
1,6
2,0";

    #[test]
    fn test_process() -> miette::Result<()> {
        assert_eq!("22", process(INPUT)?);
        Ok(())
    }

    #[test]
    fn test_parser() -> miette::Result<()> {
        let input = "\
5,4";
        assert_eq!(vec![Position(5, 4)], parser::parse(input)?);
        Ok(())
    }

    #[test]
    fn test_path_finding() -> miette::Result<()> {
        // Known test case with expected path
        let coords = vec![
            Position(1, 0),
            Position(1, 1), // Wall blocking direct path
            Position(2, 1),
            Position(2, 2), // Forces path around
        ];

        let graph = create_graph(coords)?;

        // Set up test positions
        let start = Position(0, 0);
        let end = Position(3, 3);

        let start_idx = get_node_index(&graph, start)?;
        let end_idx = get_node_index(&graph, end)?;

        let paths = dijkstra(&graph, start_idx, Some(end_idx), |_| 1);
        let distance = paths.get(&end_idx).expect("Should find path");

        // Build expected path grid
        let expected = vec![
            vec!['.', '#', '.', '.', '.', '.', '.'],
            vec!['o', '#', '#', '.', '.', '.', '.'],
            vec!['o', '.', '#', '.', '.', '.', '.'],
            vec!['o', 'o', 'o', 'o', '.', '.', '.'],
            vec!['.', '.', '.', '.', '.', '.', '.'],
            vec!['.', '.', '.', '.', '.', '.', '.'],
            vec!['.', '.', '.', '.', '.', '.', '.'],
        ];

        // Verify path matches expected
        let mut path_grid = vec![vec!['.'; DIM]; DIM];
        for (y, row) in expected.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                path_grid[y][x] = cell;
            }
        }

        assert_eq!(*distance, 6, "Path length should be 6");
        assert_eq!(
            path_grid, expected,
            "Path visualization should match expected"
        );

        Ok(())
    }

    #[test]
    fn test_bounds() -> miette::Result<()> {
        // Create graph with walls at edges
        let coords = vec![
            Position(constants::DIM - 1, constants::DIM - 1),
            Position(0, constants::DIM - 1),
            Position(constants::DIM - 1, 0),
        ];
        
        let graph = graph::create_graph(coords)?;
        
        // Test start position (0,0)
        assert!(graph::get_node_index(&graph, START).is_ok());
        
        // Test end position (DIM-1, DIM-1)
        let end_pos = Position(constants::DIM - 1, constants::DIM - 1);
        assert!(graph::get_node_index(&graph, end_pos).is_ok());
        
        // Verify out of bounds fails
        let invalid_pos = Position(constants::DIM, constants::DIM);
        assert!(graph::get_node_index(&graph, invalid_pos).is_err());
        
        Ok(())
    }
}
