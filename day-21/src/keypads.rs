use std::collections::{HashMap, VecDeque};
use std::fmt::Display;
use std::hash::Hash;

use miette::Result;
use petgraph::graph::{NodeIndex, UnGraph};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position(pub usize, pub usize);

impl Position {
    pub fn get_position(&self) -> (usize, usize) {
        (self.0, self.1)
    }
}

pub trait Key: Copy + Display + Eq + Hash + std::fmt::Debug {
    type Value: Display;
    fn value(&self) -> Self::Value;
    fn from_char(c: char) -> Option<Self>
    where
        Self: Sized;
    fn to_char(&self) -> char;
}

pub struct Keypad<K: Key> {
    keys: Vec<Vec<K>>,
    positions: HashMap<K, Position>,
    pub graph: Option<UnGraph<K, ()>>,
}

pub type Path = Vec<NodeIndex>;

impl<K: Key> Keypad<K> {
    pub fn new(keys: Vec<Vec<K>>, exclude: impl Fn(&K) -> bool) -> Self {
        let mut keypad = Self {
            keys: keys.clone(),
            positions: HashMap::new(),
            graph: None,
        };

        // Create position mapping
        keypad.positions = keypad.create_key_positions();

        // Build graph
        let graph = keypad.create_graph(keys, exclude);
        keypad.graph = Some(graph);

        keypad
    }

    fn create_graph<E: Fn(&K) -> bool>(&self, keys: Vec<Vec<K>>, exclude: E) -> UnGraph<K, ()> {
        let mut graph = UnGraph::new_undirected();
        let mut nodes = HashMap::new();

        // Add nodes
        for row in keys.iter() {
            for cell in row {
                let node = graph.add_node(*cell);
                nodes.insert(cell, node);
            }
        }

        // Add edges
        for (y, row) in keys.iter().enumerate() {
            for (x, key) in row.iter().enumerate() {
                let node = nodes[&key];
                if exclude(key) {
                    continue;
                }
                // Horizontal edges
                if x > 0 {
                    let left = &keys[y][x - 1];
                    if !exclude(left) {
                        graph.add_edge(node, nodes[left], ());
                    }
                }
                // Vertical edges
                if y > 0 {
                    let up = &keys[y - 1][x];
                    if !exclude(up) {
                        // Fixed condition
                        graph.add_edge(node, nodes[up], ());
                    }
                }
            }
        }

        graph
    }

    fn create_key_positions(&self) -> HashMap<K, Position> {
        let mut positions = HashMap::new();
        for (y, row) in self.keys.iter().enumerate() {
            for (x, key) in row.iter().enumerate() {
                positions.insert(*key, Position(x, y));
            }
        }
        positions
    }

    fn get_key_position(&self, key: K) -> Result<Position> {
        self.positions
            .get(&key)
            .copied()
            .ok_or_else(|| miette::miette!("Key not found"))
    }

    fn get_node_position(&self, node: NodeIndex) -> Result<Position> {
        let graph = self
            .graph
            .as_ref()
            .ok_or(miette::miette!("Graph not found"))?;
        let key = graph[node];
        self.get_key_position(key)
    }

    pub fn encode_path_direction(&self, path: Vec<NodeIndex>) -> Result<String> {
        let mut encoded_path = String::new();

        for (idx, node) in path.iter().skip(1).enumerate() {
            let cell_position = self.get_node_position(*node)?;
            let prev_position = self.get_node_position(path[idx])?;

            let dx = cell_position.0 as isize - prev_position.0 as isize;
            let dy = cell_position.1 as isize - prev_position.1 as isize;

            let encoded_direction = match (dx, dy) {
                (0, -1) => "^",
                (0, 1) => "v",
                (-1, 0) => "<",
                (1, 0) => ">",
                _ => return Err(miette::miette!("Invalid path!")),
            };

            encoded_path.push_str(encoded_direction);
        }

        Ok(encoded_path)
    }

    pub fn encode_sequence(&self, sequence: &str, current: Option<K>) -> Result<String> {
        if sequence.is_empty() {
            return Ok("A".to_string());
        }
    
        let mut result = String::new();
        let mut current_key = current.unwrap_or_else(|| K::from_char('A').expect("Invalid start character: A"));
        let mut chars = sequence.chars();
    
        while let Some(c) = chars.next() {
            let target = K::from_char(c)
                .ok_or_else(|| miette::miette!("Invalid character: {}", c))?;
    
            let path_options = self.find_paths(current_key, target)?;
    
            let mut scored_paths: Vec<(String, usize)> = path_options
                .into_iter()
                .filter_map(|path| {
                    self.encode_path_direction(path)
                        .ok()
                        .map(|encoded| (encoded.clone(), self.score_encoded_path(&encoded)))
                })
                .collect();
    
            scored_paths.sort_by_key(|(path, score)| (*score, path.len()));
    
            if let Some((best_path, _)) = scored_paths.last() {
                result.push_str(best_path);
            }
    
            result.push('A');
            current_key = target;
        }
    
        Ok(result)
    }

    fn score_encoded_path(&self, path: &str) -> usize {
        let patterns = ["^^", "vv", "<<", ">>", "AA"];
        patterns.iter().map(|p| path.matches(p).count()).sum()
    }

    pub fn find_paths(&self, start: K, end: K) -> Result<Vec<Path>> {
        let graph = self
            .graph
            .as_ref()
            .ok_or(miette::miette!("Graph not found"))?;

        let start_node = graph
            .node_indices()
            .find(|&idx| graph[idx] == start)
            .ok_or_else(|| miette::miette!("Start key not found"))?;

        let end_node = graph
            .node_indices()
            .find(|&idx| graph[idx] == end)
            .ok_or_else(|| miette::miette!("End key not found"))?;

        let mut paths = Vec::<Path>::new();
        let mut queue: VecDeque<(NodeIndex, Path)> = VecDeque::new();
        let mut shortest_distance = None;
        let mut distances = HashMap::new();

        queue.push_back((start_node, vec![start_node]));
        distances.insert(start_node, 0);

        while let Some((node, path)) = queue.pop_front() {
            let current_distance = distances[&node];

            if let Some(shortest) = shortest_distance {
                if current_distance > shortest {
                    continue;
                }
            }

            if node == end_node {
                paths.push(path.clone());
                shortest_distance = Some(current_distance);
                continue;
            }

            for neighbor in graph.neighbors(node) {
                let new_distance = current_distance + 1;

                // Allow paths of equal length
                if !path.contains(&neighbor)
                    && distances
                        .get(&neighbor)
                        .map_or(true, |&d| d >= new_distance)
                {
                    distances.insert(neighbor, new_distance);
                    let mut new_path = path.clone();
                    new_path.push(neighbor);
                    queue.push_back((neighbor, new_path));
                }
            }
        }

        Ok(paths)
    }

    #[allow(dead_code)]
    pub fn debug_print(&self) -> Result<()> {
        let graph = self.graph.as_ref().ok_or(miette::miette!("No graph"))?;

        // Print nodes
        for node in graph.node_indices() {
            let key = &graph[node];
            let pos = self.get_key_position(*key)?;
            println!("Node {:?}: {:?} at position {:?}", node, key, pos);
        }

        // Print edges
        for edge in graph.edge_indices() {
            let (a, b) = graph.edge_endpoints(edge).unwrap();
            println!("Edge: {:?} -> {:?}", graph[a], graph[b]);
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn debug_path(&self, path: &[NodeIndex]) -> Result<()> {
        println!("Path:");
        for (i, &node) in path.iter().enumerate() {
            let key = &self.graph.as_ref().unwrap()[node];
            let pos = self.get_key_position(*key)?;
            println!("  {}: {:?} at {:?}", i, key, pos);
        }
        Ok(())
    }
}
