use memchr::memmem::Finder;
use rayon::prelude::*;

/// Represents a 2D matrix of bytes
type Matrix = Vec<Vec<u8>>;

/// Represents possible directions for word search
#[derive(Debug, Copy, Clone)]
enum Direction {
    WestToEast,   // →
    EastToWest,   // ←
    NorthToSouth, // ↓
    SouthToNorth, // ↑
    SWtoNE,       // ↗
    NEtoSW,       // ↙
    NWtoSE,       // ↘
    SEtoNW,       // ↖
}

/// Process input string to find occurrences of "XMAS" in all directions
/// Returns the total count as a string
#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    // Parse input into byte matrix
    let data: Matrix = input.lines().map(|line| line.bytes().collect()).collect();

    if data.is_empty() {
        return Ok("0".to_string());
    }

    let directions = [
        Direction::WestToEast,
        Direction::EastToWest,
        Direction::NorthToSouth,
        Direction::SouthToNorth,
        Direction::SWtoNE,
        Direction::NEtoSW,
        Direction::NWtoSE,
        Direction::SEtoNW,
    ];

    // Process all directions in parallel
    let total = directions
        .par_iter()
        .flat_map(|&dir| transform_matrix(&data, dir))
        .map(count_xmas)
        .sum::<usize>();

    Ok(total.to_string())
}

/// Transform matrix to read in specified direction
#[must_use]
fn transform_matrix(matrix: &[Vec<u8>], direction: Direction) -> Matrix {
    match direction {
        Direction::WestToEast => matrix.to_vec(),
        Direction::EastToWest => reverse_matrix(matrix),
        Direction::NorthToSouth => transpose_matrix(matrix),
        Direction::SouthToNorth => reverse_matrix(&transpose_matrix(matrix)),
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

/// Count occurrences of "XMAS" in byte vector
#[must_use]
fn count_xmas(input: Vec<u8>) -> usize {
    let finder = Finder::new("XMAS");
    let mut count = 0;
    let mut pos = 0;

    while let Some(idx) = finder.find(&input[pos..]) {
        count += 1;
        pos += idx + 1;
    }
    count
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
        assert_eq!("18", process(input)?);
        Ok(())
    }

    #[test]
    fn test_count_xmas() {
        assert_eq!(1, count_xmas("MMMSXXMASM".bytes().collect()));
        assert_eq!(0, count_xmas("MSAMXMSMSA".bytes().collect()));
        assert_eq!(0, count_xmas("AMXSXMAAMM".bytes().collect()));
        assert_eq!(0, count_xmas("MSAMASMSMX".bytes().collect()));
        assert_eq!(1, count_xmas("XMASAMXAMM".bytes().collect()));
        assert_eq!(0, count_xmas("XXAMMXXAMA".bytes().collect()));
        assert_eq!(0, count_xmas("SMSMSASXSS".bytes().collect()));
        assert_eq!(0, count_xmas("SAXAMASAAA".bytes().collect()));
        assert_eq!(0, count_xmas("MAMMMXMMMM".bytes().collect()));
        assert_eq!(1, count_xmas("MXMXAXMASX".bytes().collect()));
    }
}
