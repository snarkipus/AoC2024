use std::collections::HashSet;

use miette::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
struct Location {
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
    fn walk(&mut self, path: &mut HashSet<Location>) {
        match self.direction {
            Direction::North => self.location.y -= 1,
            Direction::South => self.location.y += 1,
            Direction::East => self.location.x += 1,
            Direction::West => self.location.x -= 1,
        }

        self.steps += 1;
        path.insert(self.location);
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

    fn steps(&self) -> usize {
        self.steps
    }
}

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

struct EmptyCell {
    location: Location,
}

impl EmptyCell {
    fn new(x: usize, y: usize) -> Self {
        Self {
            location: Location { x, y },
        }
    }
}

type Grid = Vec<Vec<Location>>;

struct Map {
    guard: Guard,
    obstacles: Vec<Obstacle>,
    grid: Grid,
    path: HashSet<Location>,
}

impl Map {
    fn new(input: &str) -> Self {
        let mut guard = Guard::default();
        let mut obstacles = vec![];
        let mut empty_cells = vec![];

        // Get dimensions from input
        let rows = input.lines().count();
        let cols = input.lines().next().map_or(0, |line| line.len());
        let mut path: HashSet<Location> = HashSet::with_capacity(rows * cols);

        // Initialize grid with correct dimensions
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
                        path.insert(Location { x, y });
                    }
                    OBSTACLE => {
                        obstacles.push(Obstacle::new(x, y));
                    }
                    EMPTY_SPACE => {
                        empty_cells.push(EmptyCell::new(x, y));
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

    fn _steps(&self) -> usize {
        self.guard.steps()
    }

    fn unique_locations(&self) -> usize {
        self.path.len()
    }

    fn guard_location(&self) -> &Location {
        &self.guard.location
    }

    // Add bounds checking as a Map method
    fn is_within_bounds(&self) -> bool {
        let location = self.guard_location();
        location.x > 0
            && location.y > 0
            && location.x < self.grid[0].len() - 1
            && location.y < self.grid.len() - 1
    }

    // Add method to track path
    fn track_path(&mut self) -> Result<(), miette::Error> {
        while self.is_within_bounds() {
            self.walk();
        }
        Ok(())
    }

    // Make walk private since it's an implementation detail
    fn walk(&mut self) {
        if self.guard.check_obstacle(&self.obstacles) {
            self.guard.turn_right();
        } else {
            self.guard.walk(&mut self.path);
        }
    }
}

const OBSTACLE: char = '#';
const START_POS: char = '^';
const EMPTY_SPACE: char = '.';

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let mut map = Map::new(input);
    map.track_path()?;

    Ok(map.unique_locations().to_string())
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
        assert_eq!("41", process(input)?);
        Ok(())
    }
}
