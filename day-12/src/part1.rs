use std::collections::{HashMap, HashSet};

use miette::{miette, Result};
use nom::{
    character::complete::{newline, satisfy},
    multi::{many1, separated_list1},
    IResult, Parser,
};
use nom_locate::LocatedSpan;
use petgraph::graph::UnGraph;

type Position = (usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Plot {
    character: char,
    position: Position,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Map {
    xdim: usize,
    ydim: usize,
    grid: Vec<Vec<Plot>>,
}

impl Map {
    pub fn add_plot(&mut self, plot: Plot) {
        self.grid[plot.position.1 - 1][plot.position.0 - 1] = plot;
    }
}

#[derive(Debug, Clone)]
pub struct Region {
    area: usize,
    perimeter: usize,
}

impl Region {
    /// Creates a new Region from a graph of connected plots with the same character.
    /// Calculates the area (number of nodes) and perimeter (exposed edges) of the region.
    pub fn new(graph: UnGraph<Plot, ()>) -> Self {
        let area = graph.node_count();
        let perimeter = Self::calculate_perimeter(&graph);
        Self { area, perimeter }
    }

    fn calculate_perimeter(graph: &UnGraph<Plot, ()>) -> usize {
        // Extract perimeter calculation to its own function for clarity
        graph.node_indices().map(|node_idx| {
            let node_pos = graph[node_idx].position;
            let mut exposed_sides = 4;

            for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let neighbor_pos = (
                    node_pos.0 as i32 + dx,
                    node_pos.1 as i32 + dy,
                );
                
                if graph.neighbors(node_idx).any(|neighbor_idx| {
                    let neighbor = &graph[neighbor_idx];
                    neighbor.position == (neighbor_pos.0 as usize, neighbor_pos.1 as usize)
                }) {
                    exposed_sides -= 1;
                }
            }
            
            exposed_sides
        }).sum()
    }

    pub fn price(&self) -> usize {
        self.area * self.perimeter
    }
}

#[tracing::instrument]
pub fn process(input: &str) -> Result<String> {
    let map = parse_map(LocatedSpan::new(input))?;
    let graph = create_graph(&map)?;
    let subgraphs = extract_equal_value_subgraphs(&graph);
    let regions = subgraphs
        .iter()
        .map(|sg| Region::new(sg.clone()))
        .collect::<Vec<_>>();

    let price = regions.iter().fold(0, |acc, region| acc + region.price());
    Ok(price.to_string())
}

fn create_graph(map: &Map) -> Result<UnGraph<Plot, ()>> {
    let mut graph = UnGraph::<Plot, ()>::new_undirected();
    let mut indices = HashMap::new();

    // create nodes for grid
    for y in 0..map.ydim {
        for x in 0..map.xdim {
            let node = map.grid[y][x];
            let idx = graph.add_node(node);
            indices.insert((x, y), idx);
        }
    }

    // create edges for grid
    let deltas = [(0, 1), (1, 0)];

    for y in 0..map.ydim {
        for x in 0..map.xdim {
            let current = indices[&(x, y)];

            for (dx, dy) in deltas {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx < 0 || ny < 0 || nx >= map.xdim as i32 || ny >= map.ydim as i32 {
                    continue;
                }

                let nx = nx as usize;
                let ny = ny as usize;

                let neighbor = indices[&(nx, ny)];
                graph.add_edge(current, neighbor, ());
            }
        }
    }

    Ok(graph)
}

/// Extracts connected subgraphs where all nodes share the same character value.
/// Returns a vector of subgraphs, each containing nodes of a single character that
/// are connected in the original graph.
fn extract_equal_value_subgraphs<E: Clone>(graph: &UnGraph<Plot, E>) -> Vec<UnGraph<Plot, E>> {
    let mut visited = HashSet::new();
    let mut subgraphs = Vec::new();

    for start_node in graph.node_indices() {
        if visited.contains(&start_node) {
            continue;
        }

        let start_char = graph[start_node].character;
        let component = collect_connected_component(graph, start_node, start_char, &mut visited);
        
        if !component.is_empty() {
            subgraphs.push(create_subgraph(graph, &component));
        }
    }

    subgraphs
}

fn collect_connected_component<E>(
    graph: &UnGraph<Plot, E>,
    start: petgraph::graph::NodeIndex,
    target_char: char,
    visited: &mut HashSet<petgraph::graph::NodeIndex>,
) -> HashSet<petgraph::graph::NodeIndex> {
    let mut component = HashSet::new();
    let mut queue = vec![start];

    while let Some(current) = queue.pop() {
        if !visited.contains(&current) && graph[current].character == target_char {
            visited.insert(current);
            component.insert(current);

            queue.extend(
                graph.neighbors(current)
                    .filter(|&n| !visited.contains(&n) && graph[n].character == target_char)
            );
        }
    }

    component
}

fn create_subgraph<E: Clone>(
    graph: &UnGraph<Plot, E>,
    component: &HashSet<petgraph::graph::NodeIndex>,
) -> UnGraph<Plot, E> {
    let mut subgraph = UnGraph::new_undirected();
    let mut node_map = HashMap::new();

    // Add nodes
    for &node_idx in component {
        let new_idx = subgraph.add_node(graph[node_idx]);
        node_map.insert(node_idx, new_idx);
    }

    // Add edges between nodes in the component
    for &node_idx in component {
        for neighbor in graph.neighbors(node_idx) {
            if component.contains(&neighbor) {
                subgraph.add_edge(
                    node_map[&node_idx],
                    node_map[&neighbor],
                    graph.edge_weight(graph.find_edge(node_idx, neighbor).unwrap())
                        .unwrap()
                        .clone(),
                );
            }
        }
    }

    subgraph
}

// region: Nom parser
type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LocatedPlot<'a> {
    character: char,
    position: Span<'a>,
}

fn parse_alphanumeric(input: Span) -> IResult<Span, LocatedPlot> {
    satisfy(|c: char| c.is_ascii_alphanumeric())
        .map(|c| LocatedPlot {
            character: c,
            position: input,
        })
        .parse(input)
}

fn parse_grid(input: Span) -> IResult<Span, Vec<LocatedPlot>> {
    let (input, lines) = separated_list1(newline, many1(parse_alphanumeric))(input)?;
    Ok((input, lines.into_iter().flatten().collect()))
}

fn parse_map(input: Span) -> Result<Map> {
    let xdim = input
        .lines()
        .next()
        .ok_or_else(|| miette!("Failed to parse lines from input"))?
        .len();

    let ydim = input.lines().count();

    let mut map = Map {
        xdim,
        ydim,
        grid: vec![
            vec![
                Plot {
                    character: ' ',
                    position: (0, 0)
                };
                xdim
            ];
            ydim
        ],
    };

    let (_, plots) = parse_grid(input).map_err(|e| miette!("Failed to parse grid: {}", e))?;

    for plot in plots.iter() {
        map.add_plot({
            Plot {
                character: plot.character,
                position: (
                    plot.position.get_column(),
                    plot.position.location_line() as usize,
                ),
            }
        });
    }

    Ok(map)
}
// endregion

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "RRRRIICCFF
RRRRIICCCF
VVRRRCCFFF
VVRCCCJFFF
VVVVCJJCFE
VVIVCCJJEE
VVIIICJJEE
MIIIIIJJEE
MIIISIJEEE
MMMISSJEEE";
        assert_eq!("1930", process(input)?);
        Ok(())
    }

    #[test]
    fn test_process_example() -> miette::Result<()> {
        let input = "AAAA
BBCD
BBCC
EEEC";

        let map = parse_map(LocatedSpan::new(input))?;
        let graph = create_graph(&map)?;

        assert_eq!(graph.node_count(), 16);
        assert_eq!(graph.edge_count(), 24);

        let subgraphs = extract_equal_value_subgraphs(&graph);

        assert_eq!(subgraphs.len(), 5);
        let valid_subgraphs = subgraphs.iter().all(|sg| {
            sg.node_indices()
                .next()
                .map(|idx| "ABCDE".contains(sg[idx].character))
                .unwrap_or(false)
        });
        assert_eq!(valid_subgraphs, true);

        let regions = subgraphs
            .iter()
            .map(|sg| Region::new(sg.clone()))
            .collect::<Vec<_>>();

        assert_eq!(regions.len(), 5);

        let price = regions.iter().fold(0, |acc, region| acc + region.price());
        assert_eq!(price, 140);

        Ok(())
    }

    #[test]
    fn test_process_example_2() -> miette::Result<()> {
        let input = "OOOOO
OXOXO
OOOOO
OXOXO
OOOOO";

        let map = parse_map(LocatedSpan::new(input))?;
        let graph = create_graph(&map)?;

        assert_eq!(graph.node_count(), 25);
        assert_eq!(graph.edge_count(), 40);

        let subgraphs = extract_equal_value_subgraphs(&graph);

        assert_eq!(subgraphs.len(), 5);
        let valid_subgraphs = subgraphs.iter().all(|sg| {
            sg.node_indices()
                .next()
                .map(|idx| "OX".contains(sg[idx].character))
                .unwrap_or(false)
        });
        assert_eq!(valid_subgraphs, true);

        let count_o = subgraphs
            .iter()
            .filter(|sg| {
                sg.node_indices()
                    .next()
                    .map(|idx| sg[idx].character == 'O')
                    .unwrap_or(false)
            })
            .count();

        assert_eq!(count_o, 1);

        let count_x = subgraphs
            .iter()
            .filter(|sg| {
                sg.node_indices()
                    .next()
                    .map(|idx| sg[idx].character == 'X')
                    .unwrap_or(false)
            })
            .count();

        assert_eq!(count_x, 4);

        Ok(())
    }

    #[test]
    fn test_parse_map() -> miette::Result<()> {
        let input = "AB\nCD";
        let expected = Map {
            xdim: 2,
            ydim: 2,
            grid: vec![
                vec![
                    Plot {
                        character: 'A',
                        position: (1, 1),
                    },
                    Plot {
                        character: 'B',
                        position: (2, 1),
                    },
                ],
                vec![
                    Plot {
                        character: 'C',
                        position: (1, 2),
                    },
                    Plot {
                        character: 'D',
                        position: (2, 2),
                    },
                ],
            ],
        };

        let map = parse_map(LocatedSpan::new(input))?;

        assert_eq!(map, expected);
        Ok(())
    }

    #[test]
    fn test_parse_grid() -> miette::Result<()> {
        let input = LocatedSpan::new("AB\nCD");

        let expected = vec![
            Plot {
                character: 'A',
                position: (1, 1),
            },
            Plot {
                character: 'B',
                position: (2, 1),
            },
            Plot {
                character: 'C',
                position: (1, 2),
            },
            Plot {
                character: 'D',
                position: (2, 2),
            },
        ];

        let grid = parse_grid(input);
        match grid {
            Ok((_, parsed)) => {
                let result: Vec<Plot> = parsed
                    .iter()
                    .map(|plot| Plot {
                        character: plot.character,
                        position: (
                            plot.position.get_column(),
                            plot.position.location_line() as usize,
                        ),
                    })
                    .collect();
                assert_eq!(result, expected);
                Ok(())
            }
            Err(e) => Err(miette!("Error: {:?}", e)),
        }
    }
}
