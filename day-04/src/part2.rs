use std::collections::HashMap;

type Matrix = Vec<Vec<u8>>;

/// Represents coordinates in the matrix
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct Position {
    row: usize,
    col: usize,
}

/// Represents a match found in the matrix
#[derive(Debug, Clone)]
struct Match {
    position: Position,
    direction: Direction,
}

/// Possible search directions in the matrix
#[derive(Debug, Copy, Clone)]
enum Direction {
    SWtoNE, // ↗
    NEtoSW, // ↙
    NWtoSE, // ↘
    SEtoNW, // ↖
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

    let matches = find_all_matches(&data);
    let count = count_duplicate_positions(&matches, data.len());
    
    Ok(count.to_string())
}

/// Processes matrix transformations and pattern matching
fn find_all_matches(data: &Matrix) -> Vec<Match> {
    let directions = [
        Direction::SWtoNE,
        Direction::NEtoSW,
        Direction::NWtoSE,
        Direction::SEtoNW,
    ];

    directions
        .iter()
        .flat_map(|&dir| {
            let transformed = transform_matrix(data, dir);
            transformed
                .into_iter()
                .enumerate()
                .flat_map(move |(row_idx, row)| find_mas_a(row, row_idx, dir))
        })
        .collect()
}

/// Finds 'A' characters that are part of "MAS" sequences in the given row
fn find_mas_a(row: Vec<u8>, row_idx: usize, direction: Direction) -> Vec<Match> {
    const PATTERN: &[u8] = b"MAS";
    
    row.windows(PATTERN.len())
        .enumerate()
        .filter(|(_, window)| window == &PATTERN)
        .map(|(i, _)| Match {
            position: Position {
                row: row_idx,
                col: i + 1, // Points to 'A'
            },
            direction,
        })
        .collect()
}

/// Counts positions that appear exactly twice in the results
fn count_duplicate_positions(matches: &[Match], matrix_size: usize) -> usize {
    matches
        .iter()
        .filter_map(|m| transform_coords_back(m, matrix_size))
        .fold(HashMap::new(), |mut acc, pos| {
            *acc.entry(pos).or_insert(0) += 1;
            acc
        })
        .values()
        .filter(|&&count| count == 2)
        .count()
}

/// Transform matrix to read in specified direction
#[must_use]
fn transform_matrix(matrix: &[Vec<u8>], direction: Direction) -> Matrix {
    match direction {
        Direction::SWtoNE => transpose_matrix(&pad_diagonal(matrix, false)),
        Direction::NEtoSW => reverse_matrix(&transpose_matrix(&pad_diagonal(matrix, false))),
        Direction::NWtoSE => transpose_matrix(&pad_diagonal(matrix, true)),
        Direction::SEtoNW => reverse_matrix(&transpose_matrix(&pad_diagonal(matrix, true))),
    }
}

/// Add diagonal padding to matrix
#[must_use]
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
    matrix
        .iter()
        .map(|row| {
            let mut rev = row.clone();
            rev.reverse();
            rev
        })
        .collect()
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

/// Helper to map coordinates after un-transposing
fn untranspose_coords(row: usize, col: usize) -> (usize, usize) {
    (col, row) // Swap row and column
}

/// Helper to map coordinates after un-reversing
fn unreverse_coords(row: usize, col: usize, width: usize) -> (usize, usize) {
    (row, width - 1 - col) // Flip column position
}

/// Helper to map coordinates after un-padding diagonal
fn unpad_diagonal_coords(
    row: usize,
    col: usize,
    size: usize,
    reverse: bool,
) -> Option<(usize, usize)> {
    let padding = if reverse {
        size - row - 1 // Left padding for reversed diagonals
    } else {
        row // Left padding for normal diagonals
    };

    // Adjust for padding and bounds check
    let real_col = col.checked_sub(padding)?;
    if real_col >= size {
        return None;
    }

    Some((row, real_col))
}

/// Transform match coordinates back to original matrix positions
fn transform_coords_back(match_pos: &Match, size: usize) -> Option<(usize, usize)> {
    let (mut row, mut col) = (match_pos.position.row, match_pos.position.col);

    match match_pos.direction {
        Direction::SWtoNE => {
            // 1. Untranspose
            (row, col) = untranspose_coords(row, col);
            // 2. Unpad diagonal
            (row, col) = unpad_diagonal_coords(row, col, size, false)?;
        }
        Direction::NEtoSW => {
            // 1. Unreverse
            (row, col) = unreverse_coords(row, col, size);
            // 2. Untranspose
            (row, col) = untranspose_coords(row, col);
            // 3. Unpad diagonal
            (row, col) = unpad_diagonal_coords(row, col, size, false)?;
        }
        Direction::NWtoSE => {
            // 1. Untranspose
            (row, col) = untranspose_coords(row, col);
            // 2. Unpad diagonal
            (row, col) = unpad_diagonal_coords(row, col, size, true)?;
        }
        Direction::SEtoNW => {
            // 1. Unreverse
            (row, col) = unreverse_coords(row, col, size);
            // 2. Untranspose
            (row, col) = untranspose_coords(row, col);
            // 3. Unpad diagonal
            (row, col) = unpad_diagonal_coords(row, col, size, true)?;
        }
    }

    Some((row, col))
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
        let matches = find_mas_a(row, 0, Direction::SWtoNE);
        assert_eq!(matches.len(), 1);
    }
}
