use graph::{add_wall_to_graph, build_initial_graph, node_to_position, would_block_all_paths};
use miette::miette;
use petgraph::{
    algo::astar,
    graph::{DiGraph, NodeIndex},
};
use std::collections::HashMap;

#[cfg(test)]
mod constants {
    pub const DIM: usize = 7;
    pub const INITIAL_BYTES: usize = 12;
}

#[cfg(not(test))]
mod constants {
    pub const DIM: usize = 71;
    pub const INITIAL_BYTES: usize = 1024;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Position(pub usize, pub usize);

pub const START: Position = Position(0, 0);
pub const END: Position = Position(constants::DIM - 1, constants::DIM - 1);

type Grid = Vec<Vec<char>>;
type Graph = DiGraph<char, ()>;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let coords = parser::parse(input)?;
    let blocking_coord = find_blocking_coordinate_optimized(&coords)?;
    Ok(format!("{},{}", blocking_coord.0, blocking_coord.1))
}

fn find_blocking_coordinate_optimized(coords: &[Position]) -> miette::Result<Position> {
    let mut bytes = constants::INITIAL_BYTES;
    let initial_coords: Vec<Position> = coords.iter().take(bytes).copied().collect();

    // Build initial graph
    let (mut graph, node_map) = build_initial_graph(&initial_coords)?;
    let mut last_valid = true;

    // Get indices once
    let start_idx = graph::get_node_index(&graph, START)?;
    let end_idx = graph::get_node_index(&graph, END)?;

    loop {
        if bytes >= coords.len() {
            return Err(miette!(
                "No blocking coordinate found - reached end of input"
            ));
        }

        let next_coord = coords[bytes];

        // Quick check if this wall would block all possible paths
        if would_block_all_paths(&graph, &node_map, next_coord, start_idx, end_idx)? {
            return Ok(next_coord);
        }

        // Add wall and update edges
        add_wall_to_graph(&mut graph, &node_map, next_coord)?;

        // Use A* instead of Dijkstra for potentially faster pathfinding
        let path_exists = astar(
            &graph,
            start_idx,
            |n| n == end_idx,
            |_| 1,
            |n| {
                let Position(x, y) = node_to_position(&graph, n);
                let Position(end_x, end_y) = END;
                ((x as i32 - end_x as i32).abs() + (y as i32 - end_y as i32).abs()) as u32
            },
        )
        .is_some();

        if !path_exists {
            if last_valid {
                return Ok(next_coord);
            }
            break;
        }

        last_valid = true;
        bytes += 1;
    }

    Err(miette!("No blocking coordinate found"))
}

// fn find_blocking_coordinate(coords: &[Position]) -> miette::Result<Position> {
//     let mut bytes = constants::INITIAL_BYTES;
//     let mut previous_coords: Vec<Position> = coords.iter().take(bytes).copied().collect();

//     loop {
//         if bytes >= coords.len() {
//             return Err(miette!("No blocking coordinate found - reached end of input"));
//         }

//         let next_coord = coords[bytes];

//         // Update grid with new coordinate
//         let graph = graph::create_graph(&[&previous_coords[..], &[next_coord]].concat())?;

//         // Check if path still exists
//         let start_idx = graph::get_node_index(&graph, START)?;
//         let end_idx = graph::get_node_index(&graph, END)?;

//         if dijkstra(&graph, start_idx, Some(end_idx), |_| 1).contains_key(&end_idx) {
//             previous_coords.push(next_coord);
//             bytes += 1;
//         } else {
//             return Ok(next_coord);
//         }
//     }
// }

mod graph {
    use petgraph::Direction;

    use super::*;

    pub fn build_initial_graph(
        coords: &[Position],
    ) -> miette::Result<(Graph, HashMap<(usize, usize), NodeIndex>)> {
        let mut grid = vec![vec!['.'; constants::DIM]; constants::DIM];

        // Place initial walls
        for &Position(x, y) in coords {
            grid[y][x] = '#';
        }

        let mut graph = Graph::new();
        let mut node_map = HashMap::new();

        // Create nodes
        for y in 0..constants::DIM {
            for x in 0..constants::DIM {
                let node = graph.add_node(grid[y][x]);
                node_map.insert((x, y), node);
            }
        }

        // Add initial edges
        add_all_edges(&mut graph, &grid, &node_map);

        Ok((graph, node_map))
    }

    pub fn add_wall_to_graph(
        graph: &mut Graph,
        node_map: &HashMap<(usize, usize), NodeIndex>,
        pos: Position,
    ) -> miette::Result<()> {
        let Position(x, y) = pos;
        let node = node_map[&(x, y)];

        // Update node value
        graph[node] = '#';

        // Remove all outgoing edges
        while let Some(edge) = graph.first_edge(node, Direction::Outgoing) {
            graph.remove_edge(edge);
        }
        // Remove all incoming edges
        while let Some(edge) = graph.first_edge(node, Direction::Incoming) {
            graph.remove_edge(edge);
        }

        Ok(())
    }

    pub fn would_block_all_paths(
        graph: &Graph,
        node_map: &HashMap<(usize, usize), NodeIndex>,
        pos: Position,
        start_idx: NodeIndex,
        end_idx: NodeIndex,
    ) -> miette::Result<bool> {
        let Position(x, y) = pos;

        // If the wall would block the only remaining path
        let current_paths = astar(
            graph,
            start_idx,
            |n| n == end_idx,
            |_| 1,
            |n| {
                let Position(px, py) = node_to_position(graph, n);
                let Position(end_x, end_y) = END;
                ((px as i32 - end_x as i32).abs() + (py as i32 - end_y as i32).abs()) as u32
            },
        );

        if let Some((_, path)) = current_paths {
            // Check if the new wall would block this path
            if path.iter().any(|&n| node_map[&(x, y)] == n) {
                // Check if there are alternative paths
                let mut temp_graph = graph.clone();
                add_wall_to_graph(&mut temp_graph, node_map, pos)?;

                return Ok(!astar(
                    &temp_graph,
                    start_idx,
                    |n| n == end_idx,
                    |_| 1,
                    |n| {
                        let Position(px, py) = node_to_position(&temp_graph, n);
                        let Position(end_x, end_y) = END;
                        ((px as i32 - end_x as i32).abs() + (py as i32 - end_y as i32).abs()) as u32
                    },
                )
                .is_some());
            }
        }

        Ok(false)
    }

    pub fn node_to_position(_graph: &Graph, node: NodeIndex) -> Position {
        let idx = node.index();
        Position(idx % constants::DIM, idx / constants::DIM)
    }

    fn add_all_edges(
        graph: &mut Graph,
        grid: &Grid,
        node_map: &HashMap<(usize, usize), NodeIndex>,
    ) {
        const DIRECTIONS: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        for y in 0..constants::DIM {
            for x in 0..constants::DIM {
                let current_node = node_map[&(x, y)];
                if grid[y][x] == '#' {
                    continue;
                }

                for (dx, dy) in DIRECTIONS {
                    if let Some((nx, ny)) = get_neighbor_coords(x, y, dx, dy) {
                        let neighbor_node = node_map[&(nx, ny)];
                        if grid[ny][nx] == '.' {
                            graph.add_edge(current_node, neighbor_node, ());
                        }
                    }
                }
            }
        }
    }

    fn get_neighbor_coords(x: usize, y: usize, dx: i32, dy: i32) -> Option<(usize, usize)> {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;

        if nx >= 0 && ny >= 0 && nx < constants::DIM as i32 && ny < constants::DIM as i32 {
            Some((nx as usize, ny as usize))
        } else {
            None
        }
    }

    pub fn get_node_index(graph: &Graph, Position(x, y): Position) -> miette::Result<NodeIndex> {
        if x >= constants::DIM || y >= constants::DIM {
            return Err(miette!("Position ({}, {}) out of bounds", x, y));
        }

        let idx = y * constants::DIM + x;
        graph
            .node_indices()
            .nth(idx)
            .ok_or_else(|| miette!("No node found at position ({}, {})", x, y))
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn print_grid(grid: &Grid) {
        for row in grid {
            println!("{:?}", row);
        }
    }
}

mod parser {
    use super::*;

    pub fn parse(input: &str) -> miette::Result<Vec<Position>> {
        input
            .lines()
            .map(|line| {
                let mut parts = line.split(',');
                let x = parts
                    .next()
                    .ok_or_else(|| miette!("Missing x coordinate"))?
                    .trim()
                    .parse()
                    .map_err(|e| miette!("Failed to parse x coordinate: {}", e))?;
                let y = parts
                    .next()
                    .ok_or_else(|| miette!("Missing y coordinate"))?
                    .trim()
                    .parse()
                    .map_err(|e| miette!("Failed to parse y coordinate: {}", e))?;
                Ok(Position(x, y))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod graph_tests {
        use super::*;

        // #[test]
        // fn test_graph_creation() -> miette::Result<()> {
        //     let coords = vec![Position(1, 1), Position(2, 2)];
        //     let graph = graph::create_graph(&coords)?;
        //     assert!(graph.node_count() > 0);
        //     Ok(())
        // }

        // #[test]
        // fn test_path_finding() -> miette::Result<()> {
        //     let coords = vec![
        //         Position(1, 0),
        //         Position(1, 1),
        //         Position(2, 1),
        //         Position(2, 2),
        //     ];

        //     let graph = graph::create_graph(&coords)?;
        //     let start_idx = graph::get_node_index(&graph, Position(0, 0))?;
        //     let end_idx = graph::get_node_index(&graph, Position(3, 3))?;

        //     let paths = astar(&graph, start_idx, Some(end_idx), |_| 1);
        //     let distance = paths.get(&end_idx).expect("Should find path");

        //     assert_eq!(*distance, 6);
        //     Ok(())
        // }

        // #[test]
        // fn test_bounds() -> miette::Result<()> {
        //     let coords = vec![
        //         Position(constants::DIM - 2, constants::DIM - 2),
        //         Position(0, constants::DIM - 1),
        //         Position(constants::DIM - 1, 0),
        //     ];

        //     let graph = graph::create_graph(&coords)?;

        //     assert!(graph::get_node_index(&graph, START).is_ok());
        //     assert!(graph::get_node_index(&graph, END).is_ok());
        //     assert!(graph::get_node_index(&graph, Position(constants::DIM, constants::DIM)).is_err());

        //     Ok(())
        // }
    }
}
