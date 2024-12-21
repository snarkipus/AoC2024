use crate::part2::robot::*;

#[tracing::instrument]
pub fn process(_input: &str) -> miette::Result<String> {
    let (mut grid, path) = parser::parse_input(_input)?;

    let (robot_x, robot_y) = grid
        .cells
        .iter()
        .enumerate()
        .find_map(|(y, row)| {
            row.iter()
                .enumerate()
                .find(|(_, cell)| cell.is_robot())
                .map(|(x, _)| (x as i32, y as i32))
        })
        .expect("Robot not found in grid");

    let mut robot = Robot::new(robot_x, robot_y);

    for direction in path.0.iter() {
        robot.execute_move(&mut grid, *direction)?;
    }

    Ok(grid.get_grid_gps().to_string())
}

mod error {
    use miette::{Diagnostic, SourceSpan};
    use thiserror::Error;

    #[derive(Debug, Error, Diagnostic)]
    #[diagnostic(code(game_error))]
    pub(crate) enum GameError {
        #[error("Failed to parse grid: {0}")]
        Parse(String),

        #[error("Invalid robot movement: {0}")]
        Movement(String),
    }

    impl<E> From<nom::Err<E>> for GameError
    where
        E: std::fmt::Debug,
    {
        fn from(err: nom::Err<E>) -> Self {
            GameError::Parse(format!("Parsing failed: {:?}", err))
        }
    }

    impl From<GridParseError> for GameError {
        fn from(err: GridParseError) -> Self {
            GameError::Parse(format!("Grid parse error at position {:?}", err.span))
        }
    }

    #[derive(Debug, Error, Diagnostic)]
    #[error("Failed to parse grid")]
    #[diagnostic(
        code(parse_error),
        help("Input must contain only dots (.), digits (0-9), or uppercase letters (A-Z)")
    )]
    pub(crate) struct GridParseError {
        #[source_code]
        pub src: String,
        #[label("Parse error occurred here")]
        pub span: SourceSpan,
        pub kind: nom::error::ErrorKind,
    }
}

mod grid {
    use crate::part2::parser::{BOX, EMPTY, ROBOT, WALL};
    use std::fmt::{self, Display, Formatter};

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub(crate) struct GridCell {
        pub(crate) x: i32,
        pub(crate) y: i32,
        pub(crate) cell: char,
    }

    impl GridCell {
        pub(crate) fn new(x: i32, y: i32, cell: char) -> Self {
            Self { x, y, cell }
        }

        pub(crate) fn is_robot(&self) -> bool {
            self.cell == ROBOT
        }

        pub(crate) fn is_wall(&self) -> bool {
            self.cell == WALL
        }

        pub(crate) fn is_box(&self) -> bool {
            self.cell == BOX
        }

        pub(crate) fn is_empty(&self) -> bool {
            self.cell == EMPTY
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub(crate) struct Grid {
        pub(crate) cells: Vec<Vec<GridCell>>,
        pub(crate) width: i32,
        pub(crate) height: i32,
    }

    impl Display for Grid {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            for row in &self.cells {
                for cell in row {
                    write!(f, "{}", cell.cell)?;
                }
                writeln!(f)?;
            }
            Ok(())
        }
    }

    impl Grid {
        #[allow(dead_code)]
        pub(crate) fn display_grid(&self) {
            for row in &self.cells {
                for cell in row {
                    print!("{}", cell.cell);
                }
                println!();
            }
            println!();
        }

        pub(crate) fn get_row(&mut self, y: i32) -> &mut Vec<GridCell> {
            &mut self.cells[y as usize]
        }

        fn _get_column(&mut self, x: i32) -> Vec<&mut GridCell> {
            self.cells
                .iter_mut()
                .map(|row| &mut row[x as usize])
                .collect()
        }

        pub(crate) fn transpose(&mut self) -> miette::Result<()> {
            let height = self.height as usize;
            let width = self.width as usize;

            let mut transposed = vec![vec![]; width];

            for (j, row) in transposed.iter_mut().enumerate().take(width) {
                for (i, cell) in self.cells.iter().enumerate().take(height) {
                    let mut new_cell = cell[j].clone();
                    new_cell.x = i as i32;
                    new_cell.y = j as i32;
                    row.push(new_cell);
                }
            }

            self.cells = transposed;
            std::mem::swap(&mut self.width, &mut self.height);
            Ok(())
        }

        pub(crate) fn reverse_rows(&mut self) -> miette::Result<()> {
            for row in self.cells.iter_mut() {
                row.reverse();
                let width = row.len();
                for (i, cell) in row.iter_mut().enumerate() {
                    cell.x = (width - 1 - i) as i32;
                }
            }
            Ok(())
        }

        pub(crate) fn get_grid_gps(&self) -> i32 {
            self.cells
                .iter()
                .enumerate()
                .flat_map(|(y, row)| {
                    row.iter()
                        .enumerate()
                        .filter(|(_, cell)| cell.is_box())
                        .map(move |(x, _)| {
                            let from_left = x as i32;
                            let from_top = y as i32;
                            from_left + (100 * from_top)
                        })
                })
                .sum()
        }
    }
}

mod robot {
    use crate::part2::{
        error::GameError,
        grid::{Grid, GridCell},
        parser::{EMPTY, ROBOT},
    };

    #[derive(Debug, Clone, Copy)]
    pub enum Direction {
        Up,
        Down,
        Left,
        Right,
    }

    #[derive(Debug, Clone)]
    pub(crate) struct Path(pub(crate) Vec<Direction>);

    #[derive(Debug, Clone)]
    pub(crate) struct Robot {
        pub(crate) current: GridCell,
    }

    impl Robot {
        pub(crate) fn new(x: i32, y: i32) -> Self {
            Self {
                current: GridCell::new(x, y, ROBOT),
            }
        }

        pub(crate) fn execute_move(
            &mut self,
            grid: &mut Grid,
            direction: Direction,
        ) -> miette::Result<()> {
            match direction {
                Direction::Right => self.execute_movement(grid),
                Direction::Left => {
                    grid.reverse_rows().map_err(|e| {
                        GameError::Movement(format!("Failed to reverse rows: {}", e))
                    })?;
                    self.current.x = grid.width - 1 - self.current.x;

                    let result = self.execute_movement(grid);

                    // Always reverse back, but preserve the original error if there was one
                    let reverse_result = grid.reverse_rows().map_err(|e| {
                        GameError::Movement(format!("Failed to reverse rows back: {}", e))
                    });
                    self.current.x = grid.width - 1 - self.current.x;

                    result.and(Ok(reverse_result?))
                }
                Direction::Up => {
                    grid.transpose()
                        .map_err(|e| GameError::Movement(format!("Failed to transpose: {}", e)))?;
                    std::mem::swap(&mut self.current.x, &mut self.current.y);

                    grid.reverse_rows().map_err(|e| {
                        GameError::Movement(format!("Failed to reverse rows: {}", e))
                    })?;
                    self.current.x = grid.width - 1 - self.current.x;

                    let result = self.execute_movement(grid);

                    // Always clean up transformations, but preserve the original error
                    let cleanup_result = grid
                        .reverse_rows()
                        .and_then(|_| {
                            self.current.x = grid.width - 1 - self.current.x;
                            grid.transpose()
                        })
                        .map_err(|e| GameError::Movement(format!("Failed to restore grid: {}", e)));
                    std::mem::swap(&mut self.current.x, &mut self.current.y);

                    result.and(Ok(cleanup_result?))
                }
                Direction::Down => {
                    grid.transpose()
                        .map_err(|e| GameError::Movement(format!("Failed to transpose: {}", e)))?;
                    std::mem::swap(&mut self.current.x, &mut self.current.y);

                    let result = self.execute_movement(grid);

                    // Always clean up, but preserve the original error
                    let cleanup_result = grid
                        .transpose()
                        .map_err(|e| GameError::Movement(format!("Failed to restore grid: {}", e)));
                    std::mem::swap(&mut self.current.x, &mut self.current.y);

                    result.and(Ok(cleanup_result?))
                }
            }
        }

        pub(crate) fn execute_movement(&mut self, grid: &mut Grid) -> miette::Result<()> {
            let row = grid.get_row(self.current.y);
            let current_x = self.current.x as usize;

            // Check bounds and wall
            if current_x + 1 >= row.len() || row[current_x + 1].is_wall() {
                return Ok(());
            }

            // If next space is empty, just move there
            if row[current_x + 1].is_empty() {
                row[current_x].cell = EMPTY;
                self.current.x += 1;
                row[current_x + 1].cell = ROBOT;
                return Ok(());
            }

            // Count contiguous boxes and check for empty space after them
            let mut box_count = 0;
            let mut x = current_x + 1;
            while x < row.len() && row[x].is_box() {
                box_count += 1;
                x += 1;
            }

            // If we found boxes and there's space after them
            if box_count > 0 && x < row.len() && row[x].is_empty() {
                // Move boxes one space right
                for i in (current_x + 1..=x).rev() {
                    row[i].cell = row[i - 1].cell;
                }
                // Place robot
                row[current_x].cell = EMPTY;
                self.current.x += 1;
                row[current_x + 1].cell = ROBOT;
            }

            Ok(())
        }
    }
}

mod parser {
    use miette::miette;

    use nom::{
        branch::alt,
        character::complete::{char, newline, satisfy},
        combinator::value,
        multi::{fold_many1, many0, many1, separated_list1},
        sequence::preceded,
        IResult,
    };

    use crate::part2::{
        error::GridParseError,
        grid::{Grid, GridCell},
        robot::{Direction, Path},
    };

    use nom_locate::LocatedSpan;

    pub(crate) const ROBOT: char = '@';
    pub(crate) const WALL: char = '#';
    pub(crate) const BOX: char = 'O';
    pub(crate) const EMPTY: char = '.';
    pub(crate) const UP: char = '^';
    pub(crate) const DOWN: char = 'v';
    pub(crate) const LEFT: char = '<';
    pub(crate) const RIGHT: char = '>';

    fn parse_direction(input: &str) -> IResult<&str, Direction> {
        alt((
            value(Direction::Up, char(UP)),
            value(Direction::Down, char(DOWN)),
            value(Direction::Left, char(LEFT)),
            value(Direction::Right, char(RIGHT)),
        ))(input)
    }

    fn parse_directions(input: &str) -> IResult<&str, Path> {
        // Instead of creating a new string, filter out newlines during parsing
        many1(preceded(
            many0(alt((char('\n'), char('\r')))),
            parse_direction,
        ))(input)
        .map(|(remaining, directions)| (remaining, Path(directions)))
    }

    type Span<'a> = LocatedSpan<&'a str>;

    #[derive(Debug, Clone)]
    struct LocatedCell<'a> {
        cell: char,
        position: Span<'a>,
    }

    fn parse_grid_cells(input: Span) -> IResult<Span, Vec<LocatedCell>> {
        fold_many1(
            satisfy(|c| [ROBOT, WALL, BOX, EMPTY].contains(&c)),
            Vec::new,
            |mut acc, c| {
                acc.push(LocatedCell {
                    cell: c,
                    position: input, // This gets the current position for each character
                });
                acc
            },
        )(input)
    }

    type LocatedGrid<'a> = Vec<Vec<LocatedCell<'a>>>;

    fn parse_grid(input: Span) -> IResult<Span, LocatedGrid> {
        separated_list1(newline, parse_grid_cells)(input)
    }

    pub(crate) fn parse_input(input: &str) -> miette::Result<(Grid, Path)> {
        // Parse grid
        let (input, grid) = match parse_grid(LocatedSpan::new(input)) {
            Ok((input, cells)) => {
                let height = cells.len() as i32;
                let width = cells.first().map_or(0, |row| row.len()) as i32;

                let cells = cells
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|cell| {
                                GridCell::new(
                                    cell.position.location_offset() as i32,
                                    cell.position.location_line() as i32,
                                    cell.cell,
                                )
                            })
                            .collect::<Vec<GridCell>>()
                    })
                    .collect::<Vec<Vec<GridCell>>>();

                (
                    input,
                    Grid {
                        height,
                        width,
                        cells,
                    },
                )
            }
            Err(nom::Err::Error(e)) => {
                let offset = e.input.location_offset();
                let err = GridParseError {
                    src: input.to_string(),
                    span: (offset, 1).into(),
                    kind: e.code,
                };
                return Err(err.into());
            }
            Err(e) => {
                return Err(miette!("Grid Parse error: {:?}", e));
            }
        };

        // Parse newline between grid and directions
        let Ok((remaining, _)) = many1(newline::<&str, nom::error::Error<&str>>)(input.fragment())
        else {
            return Err(miette!("Line ending Parse error"));
        };

        // Parse directions
        let path = match parse_directions(remaining) {
            Ok((_, path)) => path,
            Err(e) => {
                return Err(miette!("Direction Parse error: {:?}", e));
            }
        };

        Ok((grid, path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_large() -> miette::Result<()> {
        let input = "\
##########
#..O..O.O#
#......O.#
#.OO..O.O#
#..O@..O.#
#O#..O...#
#O..O..O.#
#.OO.O.OO#
#....O...#
##########

<vv>^<v^>v>^vv^v>v<>v^v<v<^vv<<<^><<><>>v<vvv<>^v^>^<<<><<v<<<v^vv^v>^
vvv<<^>^v^^><<>>><>^<<><^vv^^<>vvv<>><^^v>^>vv<>v<<<<v<^v>^<^^>>>^<v<v
><>vv>v^v^<>><>>>><^^>vv>v<^^^>>v^v^<^^>v^^>v^<^v>v<>>v^v^<v>v^^<^^vv<
<<v<^>>^^^^>>>v^<>vvv^><v<<<>^^^vv^<vvv>^>v<^^^^v<>^>vvvv><>>v^<<^^^^^
^><^><>>><>^^<<^^v>>><^<v>^<vv>>v>>>^v><>^v><<<<v>>v<v<v>vvv>^<><<>^><
^>><>^v<><^vvv<^^<><v<<<<<><^v<<<><<<^^<v<^^^><^>>^<v^><<<^>>^v<v^v<v^
>^>>^v>vv>^<<^v<>><<><<v<<v><>v<^vv<<<>^^v^>^^>>><<^v>>v^v><^^>>^<>vv^
<><^^>^^^<><vvvvv^v<v<<>^v<v>v<<^><<><<><<<^^<<<^<<>><<><^^^>^^<>^>v<>
^^>vv<^v^v<vv>^<><v<^v>^^^>>>^^vvv^>vvv<>>>^<^>>>>>^<<^v>^vvv<>^<><<v>
v^^>>><<^^<>>^v^<v^vv<>v^<<>^<^v^v><^<<<><<^<v><v<>vv>>v><v^<vv<>v^<<^";
        assert_eq!("10092", process(input)?);
        Ok(())
    }

    #[test]
    fn test_process_small() -> miette::Result<()> {
        let input = "\
########
#..O.O.#
##@.O..#
#...O..#
#.#.O..#
#...O..#
#......#
########

<^^>>>vv<v>>v<<";

        assert_eq!("2028", process(input)?);
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use crate::part2::{
            grid::{Grid, GridCell},
            robot::{Direction, Robot},
        };

        use rstest::rstest;

        #[rstest]
        #[case::right_basic(
            Direction::Right,
            vec![vec![
                GridCell::new(0, 0, '@'),
                GridCell::new(1, 0, 'O'),
                GridCell::new(2, 0, '.'),
                GridCell::new(3, 0, 'O'),
                GridCell::new(4, 0, '.'),
                GridCell::new(5, 0, '#'),
            ]],
            vec!['.','@','O','O','.','#']  // Robot should move past empty cells and stop before boxes
        )]
        #[case::right_wall_block(
            Direction::Right,
            vec![vec![
                GridCell::new(0, 0, '@'),
                GridCell::new(1, 0, '#'),
                GridCell::new(2, 0, '.'),
                GridCell::new(3, 0, '.'),
                GridCell::new(4, 0, '.'),
                GridCell::new(5, 0, '#'),
            ]],
            vec!['@','#','.','.','.','#']  // Robot blocked by wall, shouldn't move
        )]
        #[case::right_all_empty(
            Direction::Right,
            vec![vec![
                GridCell::new(0, 0, '@'),
                GridCell::new(1, 0, '.'),
                GridCell::new(2, 0, '.'),
                GridCell::new(3, 0, '.'),
                GridCell::new(4, 0, '.'),
                GridCell::new(5, 0, '#'),
            ]],
            vec!['.','@','.','.','.','#']  // Robot should move to last empty space
        )]
        #[case::left_basic(
            Direction::Left,
            vec![vec![
                GridCell::new(0, 0, '#'),
                GridCell::new(1, 0, '.'),
                GridCell::new(2, 0, 'O'),
                GridCell::new(3, 0, '.'),
                GridCell::new(4, 0, 'O'),
                GridCell::new(5, 0, '@'),
            ]],
            vec!['#','.','O','O','@','.']  // Boxes should move left, robot moves after them
        )]
        #[case::up_basic(
            Direction::Up,
            vec![
                vec![GridCell::new(0, 0, '#')],
                vec![GridCell::new(0, 1, '.')],
                vec![GridCell::new(0, 2, 'O')],
                vec![GridCell::new(0, 3, '.')],
                vec![GridCell::new(0, 4, '@')],
            ],
            vec!['#','.','O','@','.']  // Boxes should move up, robot moves after them
        )]
        #[case::down_basic(
            Direction::Down,
            vec![
                vec![GridCell::new(0, 0, '@')],
                vec![GridCell::new(0, 1, '.')],
                vec![GridCell::new(0, 2, 'O')],
                vec![GridCell::new(0, 3, '.')],
                vec![GridCell::new(0, 4, '#')],
            ],
            vec!['.','@','O','.','#']  // Boxes should move down, robot moves after them
        )]
        #[case::up_wall_block(
            Direction::Up,
            vec![
                vec![GridCell::new(0, 0, '#')],
                vec![GridCell::new(0, 1, '#')],
                vec![GridCell::new(0, 2, '.')],
                vec![GridCell::new(0, 3, '#')],
                vec![GridCell::new(0, 4, '@')],
            ],
            vec!['#','#','.','#','@']  // Robot blocked by wall, shouldn't move
        )]
        #[case::down_all_empty(
            Direction::Down,
            vec![
                vec![GridCell::new(0, 0, '@')],
                vec![GridCell::new(0, 1, '.')],
                vec![GridCell::new(0, 2, '.')],
                vec![GridCell::new(0, 3, '.')],
                vec![GridCell::new(0, 4, '#')],
            ],
            vec!['.','@','.','.','#']  // Robot moves to last empty space
        )]
        #[case::right_multi_box(
            Direction::Right,
            vec![vec![
                GridCell::new(0, 0, '@'),
                GridCell::new(1, 0, 'O'),
                GridCell::new(2, 0, 'O'),
                GridCell::new(3, 0, '.'),
                GridCell::new(4, 0, '#'),
            ]],
            vec!['.','@','O','O','#']  // Both boxes get pushed, robot follows
        )]
        #[case::right_multi_box_blocked(
            Direction::Right,
            vec![vec![
                GridCell::new(0, 0, '@'),
                GridCell::new(1, 0, 'O'),
                GridCell::new(2, 0, 'O'),
                GridCell::new(3, 0, '#'),
            ]],
            vec!['@','O','O','#']  // Can't push because no empty space after boxes
        )]
        #[case::left_multi_box(
            Direction::Left,
            vec![vec![
                GridCell::new(0, 0, '#'),
                GridCell::new(1, 0, '.'),
                GridCell::new(2, 0, 'O'),
                GridCell::new(3, 0, 'O'),
                GridCell::new(4, 0, '@'),
            ]],
            vec!['#','O','O','@','.']  // Both boxes get pushed left
        )]
        #[case::up_multi_box(
            Direction::Up,
            vec![
                vec![GridCell::new(0, 0, '.')],
                vec![GridCell::new(0, 1, 'O')],
                vec![GridCell::new(0, 2, 'O')],
                vec![GridCell::new(0, 3, '@')],
            ],
            vec!['O','O','@','.']  // Both boxes get pushed up
        )]
        #[case::down_multi_box(
            Direction::Down,
            vec![
                vec![GridCell::new(0, 0, '@')],
                vec![GridCell::new(0, 1, 'O')],
                vec![GridCell::new(0, 2, 'O')],
                vec![GridCell::new(0, 3, '.')],
            ],
            vec!['.','@','O','O']  // Both boxes get pushed down
        )]
        fn test_robot_movement(
            #[case] direction: Direction,
            #[case] initial_cells: Vec<Vec<GridCell>>,
            #[case] expected_cells: Vec<char>,
        ) {
            let width = initial_cells[0].len() as i32;
            let height = initial_cells.len() as i32;

            let mut grid = Grid {
                cells: initial_cells.clone(),
                width,
                height,
            };

            // Find robot's initial position
            let (robot_x, robot_y) = grid
                .cells
                .iter()
                .enumerate()
                .find_map(|(y, row)| {
                    row.iter()
                        .enumerate()
                        .find(|(_, cell)| cell.is_robot())
                        .map(|(x, _)| (x as i32, y as i32))
                })
                .expect("Robot not found in grid");

            let mut robot = Robot::new(robot_x, robot_y);

            println!("Initial grid:");
            grid.display_grid();

            robot.execute_move(&mut grid, direction).unwrap();

            println!("\nFinal grid:");
            grid.display_grid();

            // Compare final state with expected
            match direction {
                Direction::Up | Direction::Down => {
                    // For vertical movements, check the specified column
                    for (i, &expected) in expected_cells.iter().enumerate() {
                        assert_eq!(
                            grid.cells[i][0].cell, expected,
                            "Mismatch at position {}: expected '{}', got '{}'",
                            i, expected, grid.cells[i][0].cell
                        );
                    }
                }
                _ => {
                    // For horizontal movements, check the specified row
                    for (i, &expected) in expected_cells.iter().enumerate() {
                        assert_eq!(
                            grid.cells[0][i].cell, expected,
                            "Mismatch at position {}: expected '{}', got '{}'",
                            i, expected, grid.cells[0][i].cell
                        );
                    }
                }
            }
        }
    }
}
