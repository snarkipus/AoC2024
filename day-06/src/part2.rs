use std::collections::HashSet;

use miette::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
enum Direction {
    #[default]
    North, // ^
    South, // v
    East,  // >
    West,  // <
}

impl Direction {
    fn turn_right(&self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
pub struct Location {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Guard {
    location: Location,
    direction: Direction,
    steps: usize,
}

impl Guard {
    fn walk(&mut self, path: &mut HashSet<PathEntry>) -> bool {
        match self.direction {
            Direction::North => self.location.y -= 1,
            Direction::South => self.location.y += 1,
            Direction::East => self.location.x += 1,
            Direction::West => self.location.x -= 1,
        }

        self.steps += 1;
        !path.insert(PathEntry {
            location: self.location,
            direction: self.direction.clone(),
        })
    }

    fn check_obstacle(&self, obstacles: &[Obstacle]) -> bool {
        // Calculate next position based on current direction
        let next = match self.direction {
            Direction::North => Location {
                x: self.location.x,
                y: self.location.y.saturating_sub(1),
            },
            Direction::South => Location {
                x: self.location.x,
                y: self.location.y + 1,
            },
            Direction::East => Location {
                x: self.location.x + 1,
                y: self.location.y,
            },
            Direction::West => Location {
                x: self.location.x.saturating_sub(1),
                y: self.location.y,
            },
        };

        // Check if next position collides with any obstacle
        obstacles.iter().any(|o| o.location == next)
    }

    fn turn_right(&mut self) {
        self.direction = self.direction.turn_right();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Obstacle {
    location: Location,
}

impl Obstacle {
    fn new(x: usize, y: usize) -> Self {
        Self {
            location: Location { x, y },
        }
    }
}

type Grid = Vec<Vec<Location>>;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct PathEntry {
    location: Location,
    direction: Direction,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Map {
    guard: Guard,
    obstacles: Vec<Obstacle>,
    grid: Grid,
    path: HashSet<PathEntry>,
}

impl Map {
    fn new(input: &str) -> Self {
        let mut guard = Guard::default();
        let mut obstacles = vec![];

        let rows = input.lines().count();
        let cols = input.lines().next().map_or(0, |line| line.len());
        let mut path: HashSet<PathEntry> = HashSet::with_capacity(rows * cols);
        let mut grid = vec![vec![Location::default(); cols]; rows];

        for (y, line) in input.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                match c {
                    START_POS => {
                        guard = Guard {
                            location: Location { x, y },
                            direction: Direction::North,
                            steps: 0, // Start at 0
                        };
                        path.insert(PathEntry {
                            location: Location { x, y },
                            direction: guard.direction.clone(),
                        });
                    }
                    OBSTACLE => {
                        obstacles.push(Obstacle::new(x, y));
                    }
                    _ => {}
                }

                grid[y][x] = Location { x, y }; // Fix grid access
            }
        }

        Self {
            guard,
            obstacles,
            grid,
            path,
        }
    }

    fn unique_locations(&self) -> usize {
        self.path.len()
    }

    fn guard_location(&self) -> &Location {
        &self.guard.location
    }

    fn is_within_bounds(&self) -> bool {
        let location = self.guard_location();
        location.x > 0
            && location.y > 0
            && location.x < self.grid[0].len() - 1
            && location.y < self.grid.len() - 1
    }

    fn track_path(&mut self) -> Result<Option<Location>, miette::Error> {
        while self.is_within_bounds() {
            if self.walk() {
                return Ok(Some(self.guard.location));
            }
        }
        Ok(None)
    }

    fn walk(&mut self) -> bool {
        if self.guard.check_obstacle(&self.obstacles) {
            self.guard.turn_right();
            false
        } else {
            self.guard.walk(&mut self.path)
        }
    }
}

const OBSTACLE: char = '#';
const START_POS: char = '^';

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<(Vec<Location>, String)> {
    let mut original_map = Map::new(input);
    original_map.track_path()?;

    let mut loop_locations = HashSet::new();

    // Skip first location (start position)
    for step in original_map.path.iter().skip(1) {
        let mut test_map = Map::new(input);
        test_map.obstacles.push(Obstacle {
            location: step.location,
        });

        let mut steps = 0;
        const MAX_STEPS: usize = 1000; // Prevent infinite loops

        while test_map.is_within_bounds() {
            steps += 1;
            if steps > MAX_STEPS {
                // Likely stuck in pattern without true loop
                break;
            }

            if test_map.guard.walk(&mut test_map.path) {
                // Verify loop is real by checking path length
                if test_map.path.len() > 2 {
                    loop_locations.insert(step.location);
                }
                break;
            }

            if test_map.guard.check_obstacle(&test_map.obstacles) {
                test_map.guard.turn_right();
            }
        }
    }

    Ok((
        loop_locations.clone().into_iter().collect(),
        loop_locations.len().to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "....#.....
.........#
..........
..#.......
.......#..
..........
.#..^.....
........#.
#.........
......#...";
        assert_eq!("6", process(input)?.1);
        Ok(())
    }

    #[test]
    fn test_process2() -> miette::Result<()> {
        let input = "....#.....
.........#
..........
..#.......
.......#..
..........
.#..^.....
........#.
#.........
......#...";

        let answers: Vec<Location> = vec![
            Location { x: 3, y: 6 },
            Location { x: 6, y: 7 },
            Location { x: 7, y: 7 },
            Location { x: 1, y: 8 },
            Location { x: 3, y: 8 },
            Location { x: 7, y: 9 },
        ];

        let mut a_sorted = answers.to_vec();
        let mut b_sorted = process(input)?.0;

        a_sorted.sort();
        b_sorted.sort();

        assert_eq!(a_sorted, b_sorted);
        Ok(())
    }
}
