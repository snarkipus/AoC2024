use miette::miette;
use petgraph::{
    algo::dijkstra,
    graph::{DiGraph, NodeIndex},
};
use std::collections::HashMap;

#[cfg(test)]
mod constants {
    pub const DIM: usize = 7;
    pub const BYTES: usize = 12;
}

#[cfg(not(test))]
mod constants {
    pub const DIM: usize = 71;
    pub const BYTES: usize = 1024;
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
    let graph = graph::create_graph(&coords)?;
    
    let start_idx = graph::get_node_index(&graph, START)?;
    let end_idx = graph::get_node_index(&graph, END)?;
    
    let path = dijkstra(&graph, start_idx, Some(end_idx), |_| 1);
    let distance = path
        .get(&end_idx)
        .ok_or_else(|| miette!("No path found to end position"))?;
    
    Ok(distance.to_string())
}

mod graph {
    use super::*;

    pub fn create_graph(coords: &[Position]) -> miette::Result<Graph> {
        let mut grid = create_empty_grid();
        validate_coordinates(coords)?;
        place_walls(&mut grid, coords);
        
        let (mut graph, nodes) = create_nodes(&grid);
        add_edges(&grid, &mut graph, &nodes);
        
        Ok(graph)
    }

    fn create_empty_grid() -> Grid {
        vec![vec!['.'; constants::DIM]; constants::DIM]
    }

    fn validate_coordinates(coords: &[Position]) -> miette::Result<()> {
        for Position(x, y) in coords {
            if *x >= constants::DIM || *y >= constants::DIM {
                return Err(miette!("Coordinates ({}, {}) out of bounds (max: {})", 
                    x, y, constants::DIM - 1));
            }
        }
        Ok(())
    }

    fn place_walls(grid: &mut Grid, coords: &[Position]) {
        coords.iter()
            .take(constants::BYTES)
            .for_each(|Position(x, y)| {
                grid[*y][*x] = '#';
            });
    }

    fn create_nodes(grid: &Grid) -> (Graph, HashMap<(usize, usize), NodeIndex>) {
        let mut graph = Graph::new();
        let mut nodes = HashMap::new();
        
        for y in 0..constants::DIM {
            for x in 0..constants::DIM {
                let node = graph.add_node(grid[y][x]);
                nodes.insert((x, y), node);
            }
        }
        
        (graph, nodes)
    }

    fn add_edges(_grid: &Grid, graph: &mut Graph, nodes: &HashMap<(usize, usize), NodeIndex>) {
        const DIRECTIONS: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        
        for y in 0..constants::DIM {
            for x in 0..constants::DIM {
                let current_node = nodes[&(x, y)];
                if graph[current_node] == '#' {
                    continue;
                }

                for (dx, dy) in DIRECTIONS {
                    if let Some((nx, ny)) = get_neighbor_coords(x, y, dx, dy) {
                        let neighbor_node = nodes[&(nx, ny)];
                        if graph[neighbor_node] == '.' {
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

    const INPUT: &str = "\
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
        assert_eq!(vec![Position(5, 4)], parser::parse("5,4")?);
        Ok(())
    }

    mod graph_tests {
        use super::*;

        #[test]
        fn test_graph_creation() -> miette::Result<()> {
            let coords = vec![Position(1, 1), Position(2, 2)];
            let graph = graph::create_graph(&coords)?;
            assert!(graph.node_count() > 0);
            Ok(())
        }

        #[test]
        fn test_path_finding() -> miette::Result<()> {
            // Test path with known obstacles
            let coords = vec![
                Position(1, 0),
                Position(1, 1),
                Position(2, 1),
                Position(2, 2),
            ];
            
            let graph = graph::create_graph(&coords)?;
            let start_idx = graph::get_node_index(&graph, Position(0, 0))?;
            let end_idx = graph::get_node_index(&graph, Position(3, 3))?;
            
            let paths = dijkstra(&graph, start_idx, Some(end_idx), |_| 1);
            let distance = paths.get(&end_idx).expect("Should find path");
            
            assert_eq!(*distance, 6);
            Ok(())
        }

        #[test]
        fn test_bounds() -> miette::Result<()> {
            let coords = vec![
                Position(constants::DIM - 2, constants::DIM - 2),
                Position(0, constants::DIM - 1),
                Position(constants::DIM - 1, 0),
            ];
            
            let graph = graph::create_graph(&coords)?;
            
            assert!(graph::get_node_index(&graph, START).is_ok());
            assert!(graph::get_node_index(&graph, END).is_ok());
            assert!(graph::get_node_index(&graph, Position(constants::DIM, constants::DIM)).is_err());
            
            Ok(())
        }
    }
}