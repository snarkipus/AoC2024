use miette::{miette, Result};
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

// region: miette error handling
#[derive(Debug, Error, Diagnostic)]
#[error("Invalid character in input")]
#[diagnostic(code(parse::invalid_char), help("Input must contain only digits 0-9"))]
struct InvalidCharError {
    #[source_code]
    src: String,

    #[label("invalid character found here")]
    span: SourceSpan,

    character: char,
}

impl InvalidCharError {
    fn new(input: &str, pos: usize, c: char) -> Self {
        let start = pos.saturating_sub(5);
        let end = (pos + 1).min(input.len());
        let context_end = (pos + 6).min(input.len());

        Self {
            src: input[start..context_end].to_string(),
            span: ((pos - start)..(end - start)).into(),
            character: c,
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid block size")]
#[diagnostic(
    code(parse::invalid_block_size),
    help("Block size must be a single digit 0-9")
)]
struct InvalidBlockSizeError {
    #[source_code]
    src: String,

    #[label("invalid block size")]
    span: SourceSpan,

    digit: char,
}

impl InvalidBlockSizeError {
    fn new(input: &str, pos: usize, digit: char) -> Self {
        let start = pos.saturating_sub(5);
        let end = (pos + 1).min(input.len());
        let context_end = (pos + 6).min(input.len());

        Self {
            src: input[start..context_end].to_string(),
            span: ((pos - start)..(end - start)).into(),
            digit,
        }
    }
}

// endregion
#[derive(Debug, Clone, PartialEq, Eq)]
struct FileBlock {
    id: usize,
    size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskState {
    // Each entry represents a file block or free space
    // Files are FileBlock instances, free spaces are None
    blocks: Vec<Option<FileBlock>>,
}

impl DiskState {
    pub fn new(input: &str) -> Result<Self> {
        if input.is_empty() {
            return Err(miette!("Empty input"));
        }

        // Find non-digit characters with their positions
        if let Some((pos, c)) = input.chars().enumerate().find(|(_, c)| !c.is_ascii_digit()) {
            return Err(InvalidCharError::new(input, pos, c).into());
        }

        let mut blocks = Vec::new();
        let mut file_id = 0;

        // Parse alternating digits as file sizes and free spaces
        for (i, size_char) in input.chars().enumerate() {
            let size = size_char
                .to_digit(10)
                .ok_or_else(|| InvalidBlockSizeError::new(input, i, size_char))?
                as usize;

            if i % 2 == 0 {
                // File blocks
                for _ in 0..size {
                    blocks.push(Some(FileBlock { id: file_id, size }));
                }
                file_id += 1;
            } else {
                // Free space
                for _ in 0..size {
                    blocks.push(None);
                }
            }
        }

        Ok(Self { blocks })
    }

    pub fn pack(&mut self) -> Result<()> {
        let len = self.blocks.len();
        if len == 0 {
            return Ok(());
        }

        let mut right = len - 1;
        let mut left = 0;

        // Find initial right pointer position (rightmost block)
        while right > 0 && self.blocks[right].is_none() {
            right -= 1;
        }

        while right > left {
            // Find next gap
            while left < right && self.blocks[left].is_some() {
                left += 1;
            }
            // If no more gaps before right pointer, we're done
            if left >= right {
                break;
            }

            // Found a gap at left and a block at right, swap them
            self.blocks.swap(left, right);

            // Find new rightmost block
            while right > left && self.blocks[right].is_none() {
                right -= 1;
            }
        }

        Ok(())
    }
    // Helper method for debugging
    fn debug_state(&self) -> String {
        format!("{}", self)
    }

    pub fn checksum(&self) -> Result<u64> {
        self.blocks
            .iter()
            .enumerate()
            .filter_map(|(pos, maybe_block)| {
                maybe_block.as_ref().map(|block| {
                    // Multiply position by file ID for each block
                    (pos as u64)
                        .checked_mul(block.id as u64)
                        .ok_or_else(|| miette!("Checksum multiplication overflow"))
                })
            })
            .try_fold(0_u64, |acc, res| {
                let product = res?;
                acc.checked_add(product)
                    .ok_or_else(|| miette!("Checksum addition overflow"))
            })
    }
}

impl std::fmt::Display for DiskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for block in &self.blocks {
            match block {
                Some(file) => write!(f, "{}", file.id)?,
                None => write!(f, ".")?,
            }
        }
        Ok(())
    }
}

#[tracing::instrument]
pub fn process(input: &str) -> Result<String> {
    let mut disk_state = DiskState::new(input.trim())?;
    disk_state.pack()?;
    Ok(disk_state.checksum()?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log;

    #[test_log::test]
    fn test_process() -> Result<()> {
        let input = "2333133121414131402";
        assert_eq!("1928", process(input)?);
        Ok(())
    }

    #[test_log::test]
    fn test_process_small() -> Result<()> {
        let input = "12345";
        assert_eq!("60", process(input)?);
        Ok(())
    }

    #[test]
    fn test_disk_state_parser() -> Result<()> {
        let input = "12345";
        let expected = DiskState {
            blocks: vec![
                Some(FileBlock { id: 0, size: 1 }), // First file (size 1)
                None,
                None,                               // Free space (size 2)
                Some(FileBlock { id: 1, size: 3 }), // Second file (size 3)
                Some(FileBlock { id: 1, size: 3 }),
                Some(FileBlock { id: 1, size: 3 }),
                None,
                None,
                None,
                None,                               // Free space (size 4)
                Some(FileBlock { id: 2, size: 5 }), // Third file (size 5)
                Some(FileBlock { id: 2, size: 5 }),
                Some(FileBlock { id: 2, size: 5 }),
                Some(FileBlock { id: 2, size: 5 }),
                Some(FileBlock { id: 2, size: 5 }),
            ],
        };

        assert_eq!(expected, DiskState::new(input)?);
        Ok(())
    }

    #[test]
    fn test_disk_state_display() -> Result<()> {
        let input = "12345";
        let disk_state = DiskState::new(input)?;
        assert_eq!("0..111....22222", format!("{}", disk_state));
        Ok(())
    }

    #[test]
    fn test_disk_state_parser_long() -> Result<()> {
        let input = "2333133121";
        let disk_state = DiskState::new(input)?;
        // First pair: 2,3 -> file id 0 size 2, space 3
        // Second pair: 3,3 -> file id 1 size 3, space 3
        // Third pair: 1,3 -> file id 2 size 1, space 3
        // Fourth pair: 3,1 -> file id 3 size 3, space 1
        // Fifth pair: 2,1 -> file id 4 size 2, space 1
        assert_eq!("00...111...2...333.44.", format!("{}", disk_state));
        Ok(())
    }

    #[test_log::test]
    fn test_parser_invalid_input() -> Result<()> {
        let input = "123A45";
        assert!(DiskState::new(input).is_err());
        Ok(())
    }

    #[test]
    fn test_empty_input() -> Result<()> {
        let input = "";
        assert!(DiskState::new(input).is_err());
        Ok(())
    }
}