use std::collections::{HashMap, HashSet};
use std::fmt;

use miette::{miette, Context, Result};
use nom::{
    character::complete::{newline, satisfy},
    multi::{many1, separated_list1},
    IResult, Parser,
};
use nom_locate::LocatedSpan;
use petgraph::graph::{DiGraph, NodeIndex};
use tracing::{debug, info};

mod constants {
    pub const TRAILHEAD: u8 = 0;
    pub const PEAK: u8 = 9;
    pub const MIN_VALUE: u8 = TRAILHEAD;
    pub const MAX_VALUE: u8 = PEAK;
}

use constants::*;

/// Represents a node in the climbing grid with position and height value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Node {
    x: usize,
    y: usize,
    value: u8,
}

/// Represents the climbing grid with dimensions and node values
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
    grid: Vec<Vec<Node>>,
    xdim: usize,
    ydim: usize,
}

impl Map {
    fn add_node(&mut self, node: Node) {
        self.grid[node.y][node.x] = node;
    }

    fn get(&self, x: usize, y: usize) -> Option<&Node> {
        self.grid.get(y).and_then(|row| row.get(x))
    }

    fn dimensions(&self) -> (usize, usize) {
        (self.xdim, self.ydim)
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.grid {
            for node in row {
                write!(f, "{}", node.value)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Processes a climbing grid and returns the total number of reachable peaks from all trailheads
/// 
/// # Arguments
/// * `input` - String containing the grid of numbers representing heights
/// 
/// # Returns
/// * `Result<String>` - The sum of reachable peaks from each trailhead
/// 
/// # Errors
/// * If the input is empty or malformed
/// * If no peaks or trailheads are found
#[tracing::instrument]
pub fn process(input: &str) -> Result<String> {
    info!("Processing climbing grid");
    let map = parse_input(input)
        .context("Failed to parse input grid")?;
    
    debug!("Created map with dimensions {:?}", map.dimensions());
    
    let graph = create_graph(&map)
        .context("Failed to create graph representation")?;
    
    debug!("Created graph with {} nodes and {} edges", 
           graph.node_count(), graph.edge_count());

    let result = count_reachable_peaks(&graph)
        .context("Failed to count reachable peaks")?;

    let total = result.iter().fold(0, |total, (_, count)| total + count);
    debug!("Found total of {} reachable peaks", total);

    Ok(total.to_string())
}

fn parse_input(input: &str) -> Result<Map> {
    // Input validation
    let xdim = input.lines().next()
        .ok_or_else(|| miette!("Input is empty"))?
        .len();
    let ydim = input.lines().count();
    
    if ydim == 0 {
        return Err(miette!("Input has no lines"));
    }

    if input.lines().any(|line| line.len() != xdim) {
        return Err(miette!("Input grid is not rectangular"));
    }

    let mut map = Map {
        grid: vec![
            vec![
                Node {
                    x: 0,
                    y: 0,
                    value: 0
                };
                xdim
            ];
            ydim
        ],
        xdim,
        ydim,
    };

    let result = parse_grid(LocatedSpan::new(input.as_bytes()))
        .map_err(|e| miette!("Failed to parse grid: {}", e))?;

    // Validate parsed values
    for node in result.1.iter() {
        if node.value > MAX_VALUE {
            return Err(miette!("Invalid height value {} at line {}, column {}", 
                node.value,
                node.position.location_line(),
                node.position.get_column()));
        }
    }

    result.1.iter().for_each(|node| {
        map.add_node(Node {
            x: node.position.get_column().saturating_sub(1),
            y: (node.position.location_line() as usize).saturating_sub(1),
            value: node.value,
        });
    });

    Ok(map)
}

/// Creates a directed graph representation of the climbing map
/// 
/// Edges are created between adjacent nodes where the destination
/// is exactly one value higher than the source.
fn create_graph(map: &Map) -> Result<DiGraph<Node, ()>> {
    let mut graph = DiGraph::<Node, ()>::new();
    let mut indices = HashMap::new();

    // First pass: add all nodes
    for y in 0..map.ydim {
        for x in 0..map.xdim {
            let node = map.grid[y][x];
            let idx = graph.add_node(node);
            indices.insert((x, y), idx);
        }
    }

    // Second pass: add edges according to rules
    let deltas = [(0, 1), (1, 0), (0, -1), (-1, 0)]; // Down, Right, Up, Left
    
    for y in 0..map.ydim {
        for x in 0..map.xdim {
            let current = indices[&(x, y)];
            let current_node = graph[current];

            for (dx, dy) in deltas {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx < 0 || ny < 0 || nx >= map.xdim as i32 || ny >= map.ydim as i32 {
                    continue;
                }

                let nx = nx as usize;
                let ny = ny as usize;

                let neighbor = indices[&(nx, ny)];
                let neighbor_node = graph[neighbor];

                if neighbor_node.value == current_node.value + 1 {
                    graph.add_edge(current, neighbor, ());
                }
            }
        }
    }

    Ok(graph)
}

/// Counts how many peaks each trailhead can reach
/// 
/// Returns a vector of tuples (trailhead_node_index, number_of_reachable_peaks)
fn count_reachable_peaks(graph: &DiGraph<Node, ()>) -> Result<Vec<(NodeIndex, usize)>> {
    let peaks: HashSet<_> = graph
        .node_indices()
        .filter(|idx| graph[*idx].value == PEAK)
        .collect();

    let trailheads: Vec<_> = graph
        .node_indices()
        .filter(|idx| graph[*idx].value == TRAILHEAD)
        .collect();

    if peaks.is_empty() {
        return Err(miette!("No peaks found in the grid"));
    }

    if trailheads.is_empty() {
        return Err(miette!("No trailheads found in the grid"));
    }

    debug!("Found {} peaks and {} trailheads", peaks.len(), trailheads.len());

    let mut result = Vec::new();
    
    // Pre-calculate all reachable nodes from each trailhead
    for &start in &trailheads {
        let mut visited = HashSet::new();
        let mut reachable_peaks = 0;
        
        // Stack-based DFS to avoid recursion overhead
        let mut stack = vec![start];
        visited.insert(start);
        
        while let Some(current) = stack.pop() {
            // If we found a peak, increment counter
            if peaks.contains(&current) {
                reachable_peaks += 1;
            }
            
            // Add all unvisited neighbors to the stack
            for neighbor in graph.neighbors(current) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    stack.push(neighbor);
                }
            }
        }
        
        result.push((start, reachable_peaks));
    }

    Ok(result)
}

// region: parser module
mod parser {
    use super::*;

    type Span<'a> = LocatedSpan<&'a [u8]>;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct LocatedNode<'a> {
        pub value: u8,
        pub position: Span<'a>,
    }

    pub(crate) fn parse_node(input: Span) -> IResult<Span, LocatedNode> {
        satisfy(|c: char| c.is_ascii_digit())
            .map(|c| LocatedNode {
                value: (c as u8) - b'0',
                position: input,
            })
            .parse(input)
    }

    pub(crate) fn parse_grid(input: Span) -> IResult<Span, Vec<LocatedNode>> {
        let (input, lines) = separated_list1(newline, many1(parse_node))(input)?;
        Ok((input, lines.into_iter().flatten().collect()))
    }
}

use parser::*;

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::visit::EdgeRef;

    #[test]
    fn test_process() -> Result<()> {
        let input = "89010123
78121874
87430965
96549874
45678903
32019012
01329801
10456732";
        assert_eq!("36", process(input)?);
        Ok(())
    }

    #[test]
    fn test_graph_creation() -> Result<()> {
        let input = "12\n34";
        let parsed = parse_input(input)?;
        let graph = create_graph(&parsed)?;

        assert_eq!(4, graph.node_count(), "Should have 4 nodes");
        assert_eq!(2, graph.edge_count(), "Should have 2 edges");
        Ok(())
    }

    #[test]
    fn test_edge_directions() -> Result<()> {
        let input = "123\n654";
        let parsed = parse_input(input)?;
        let graph = create_graph(&parsed)?;

        assert_eq!(5, graph.edge_count(), "Expected 5 edges in the graph");

        let mut found_edges = [false; 5];

        for edge in graph.edge_references() {
            let from = graph[edge.source()].value;
            let to = graph[edge.target()].value;

            match (from, to) {
                (1, 2) => found_edges[0] = true,
                (2, 3) => found_edges[1] = true,
                (3, 4) => found_edges[2] = true,
                (4, 5) => found_edges[3] = true,
                (5, 6) => found_edges[4] = true,
                _ => panic!("Unexpected edge from {} to {}", from, to),
            }
        }

        assert!(found_edges.iter().all(|&x| x), "Not all expected edges were found");
        Ok(())
    }

    #[test]
    fn test_single_path() -> Result<()> {
        let input = "0123\n1234\n8765\n9876";
        let parsed = parse_input(input)?;
        let graph = create_graph(&parsed)?;
        let paths = count_reachable_peaks(&graph)?;

        assert_eq!(1, paths.len(), "Expected 1 trailhead");
        assert!(paths[0].1 > 0, "Trailhead should reach at least one peak");
        Ok(())
    }

    #[test]
    fn test_map_display() -> Result<()> {
        let input = "12\n34";
        let map = parse_input(input)?;
        let display = format!("{}", map);
        assert_eq!("12\n34\n", display);
        Ok(())
    }
}