use std::collections::HashMap;

type Matrix = Vec<Vec<u8>>;
type Coordinate = (usize, usize);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct Position {
    row: usize,
    col: usize,
}

impl Position {
    fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    fn to_coordinate(self) -> Coordinate {
        (self.row, self.col)
    }
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    SWtoNE, // ↗
    NEtoSW, // ↙
    NWtoSE, // ↘
    SEtoNW, // ↖
}

impl Direction {
    fn all() -> &'static [Direction] {
        &[
            Direction::SWtoNE,
            Direction::NEtoSW,
            Direction::NWtoSE,
            Direction::SEtoNW,
        ]
    }

    fn transform_coords(&self, pos: Position, matrix_size: usize) -> Option<Coordinate> {
        let (mut row, mut col) = pos.to_coordinate();

        match self {
            Direction::SWtoNE => {
                (row, col) = MatrixOps::untranspose_coords(row, col);
                (row, col) = MatrixOps::unpad_diagonal_coords(row, col, matrix_size, false)?;
            }
            Direction::NEtoSW => {
                (row, col) = MatrixOps::unreverse_coords(row, col, matrix_size);
                (row, col) = MatrixOps::untranspose_coords(row, col);
                (row, col) = MatrixOps::unpad_diagonal_coords(row, col, matrix_size, false)?;
            }
            Direction::NWtoSE => {
                (row, col) = MatrixOps::untranspose_coords(row, col);
                (row, col) = MatrixOps::unpad_diagonal_coords(row, col, matrix_size, true)?;
            }
            Direction::SEtoNW => {
                (row, col) = MatrixOps::unreverse_coords(row, col, matrix_size);
                (row, col) = MatrixOps::untranspose_coords(row, col);
                (row, col) = MatrixOps::unpad_diagonal_coords(row, col, matrix_size, true)?;
            }
        }

        Some((row, col))
    }
}

#[derive(Debug, Clone)]
struct Match {
    position: Position,
    direction: Direction,
}

impl Match {
    fn new(row: usize, col: usize, direction: Direction) -> Self {
        Self {
            position: Position::new(row, col),
            direction,
        }
    }

    fn transform_coords_back(&self, matrix_size: usize) -> Option<Coordinate> {
        self.direction.transform_coords(self.position, matrix_size)
    }
}

struct MatrixOps;

impl MatrixOps {
    const PATTERN: &'static [u8] = b"MAS";

    fn transform_matrix(matrix: &[Vec<u8>], direction: Direction) -> Matrix {
        match direction {
            Direction::SWtoNE => Self::transpose_matrix(&Self::pad_diagonal(matrix, false)),
            Direction::NEtoSW => Self::reverse_matrix(&Self::transpose_matrix(&Self::pad_diagonal(matrix, false))),
            Direction::NWtoSE => Self::transpose_matrix(&Self::pad_diagonal(matrix, true)),
            Direction::SEtoNW => Self::reverse_matrix(&Self::transpose_matrix(&Self::pad_diagonal(matrix, true))),
        }
    }

    fn pad_diagonal(matrix: &[Vec<u8>], reverse: bool) -> Matrix {
        let size = matrix.len();
        matrix
            .iter()
            .enumerate()
            .map(|(i, row)| {
                let (left, right) = if reverse {
                    (size - i - 1, i)
                } else {
                    (i, size - i - 1)
                };
                [vec![b' '; left], row.to_vec(), vec![b' '; right]].concat()
            })
            .collect()
    }

    fn reverse_matrix(matrix: &[Vec<u8>]) -> Matrix {
        matrix.iter().map(|row| row.iter().rev().copied().collect()).collect()
    }

    fn transpose_matrix(matrix: &[Vec<u8>]) -> Matrix {
        if matrix.is_empty() {
            return vec![];
        }

        let cols = matrix[0].len();
        (0..cols)
            .map(|col| matrix.iter().map(|row| row[col]).collect())
            .collect()
    }

    fn untranspose_coords(row: usize, col: usize) -> (usize, usize) {
        (col, row)
    }

    fn unreverse_coords(row: usize, col: usize, width: usize) -> (usize, usize) {
        (row, width - 1 - col)
    }

    fn unpad_diagonal_coords(
        row: usize,
        col: usize,
        size: usize,
        reverse: bool,
    ) -> Option<(usize, usize)> {
        let padding = if reverse {
            size - row - 1
        } else {
            row
        };

        let real_col = col.checked_sub(padding)?;
        if real_col >= size {
            return None;
        }

        Some((row, real_col))
    }
}

struct PatternMatcher;

impl PatternMatcher {
    fn find_all_matches(data: &Matrix) -> Vec<Match> {
        Direction::all()
            .iter()
            .flat_map(|&dir| {
                let transformed = MatrixOps::transform_matrix(data, dir);
                transformed
                    .into_iter()
                    .enumerate()
                    .flat_map(move |(row_idx, row)| Self::find_mas_a(row, row_idx, dir))
            })
            .collect()
    }

    fn find_mas_a(row: Vec<u8>, row_idx: usize, direction: Direction) -> Vec<Match> {
        row.windows(MatrixOps::PATTERN.len())
            .enumerate()
            .filter(|(_, window)| window == &MatrixOps::PATTERN)
            .map(|(i, _)| Match::new(row_idx, i + 1, direction))
            .collect()
    }

    fn count_duplicate_positions(matches: &[Match], matrix_size: usize) -> usize {
        matches
            .iter()
            .filter_map(|m| m.transform_coords_back(matrix_size))
            .fold(HashMap::new(), |mut acc, pos| {
                *acc.entry(pos).or_insert(0) += 1;
                acc
            })
            .values()
            .filter(|&&count| count == 2)
            .count()
    }
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let data: Matrix = input
        .lines()
        .map(|line| line.bytes().collect())
        .collect();

    if data.is_empty() {
        return Ok("0".to_string());
    }

    let matches = PatternMatcher::find_all_matches(&data);
    let count = PatternMatcher::count_duplicate_positions(&matches, data.len());
    
    Ok(count.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "MMMSXXMASM\n\
                    MSAMXMSMSA\n\
                    AMXSXMAAMM\n\
                    MSAMASMSMX\n\
                    XMASAMXAMM\n\
                    XXAMMXXAMA\n\
                    SMSMSASXSS\n\
                    SAXAMASAAA\n\
                    MAMMMXMMMM\n\
                    MXMXAXMASX";
        assert_eq!("9", process(input)?);
        Ok(())
    }

    #[test]
    fn test_find_mas_a() {
        let row = b"MMASAS".to_vec();
        let matches = PatternMatcher::find_mas_a(row, 0, Direction::SWtoNE);
        assert_eq!(matches.len(), 1);
    }
}