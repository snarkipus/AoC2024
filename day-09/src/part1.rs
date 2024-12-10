use miette::{miette, Result};
use miette::{Diagnostic, SourceSpan};
use std::fs::write;
use thiserror::Error;

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

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid free space")]
#[diagnostic(
    code(parse::invalid_free_space),
    help("Free space must be a single digit 0-9")
)]
struct InvalidFreeSizeError {
    #[source_code]
    src: String,

    #[label("invalid free space")]
    span: SourceSpan,

    digit: char,
}

impl InvalidFreeSizeError {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct Map {
    blocks: Vec<char>,
    free_space: Vec<char>,
}

impl Map {
    fn new(block_size: usize, free_size: usize, id: usize) -> Result<Self> {
        // Convert id to digit char with proper error context
        let digit = char::from_digit(id as u32, 10).ok_or_else(|| {
            InvalidBlockSizeError::new(
                &id.to_string(), // Use id as source string
                0,               // Position at start
                id.to_string().chars().next().unwrap_or('0'),
            )
        })?;

        // Create vectors with validated sizes
        let blocks = std::iter::repeat(digit).take(block_size).collect();
        let free_space = std::iter::repeat('.').take(free_size).collect();

        Ok(Self { blocks, free_space })
    }

    fn push_block(&mut self, block: char) -> Result<()> {
        if self.free_space.is_empty() {
            return Err(miette!("No free space left"));
        }
        self.blocks.push(block);
        self.free_space.pop();
        Ok(())
    }

    fn pop_block(&mut self) -> Result<char> {
        if self.blocks.is_empty() {
            return Err(miette!("No blocks left"));
        }
        self.free_space.push('.');
        Ok(self.blocks.pop().unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Region {
    block_size: usize,
    free_size: usize,
    region_id: usize,
    map: Map,
}

impl Region {
    fn new(block_size: usize, free_size: usize, id: usize) -> Result<Self> {
        Ok(Self {
            block_size,
            free_size,
            region_id: id,
            map: Map::new(block_size, free_size, id)?,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct DiskMap(Vec<DiskBlock>);

impl DiskMap {
    #[tracing::instrument]
    fn pack(&mut self) -> Result<()> {
        let total_regions: usize = self.0.iter().map(|block| block.regions.len()).sum();
        let mut forward_idx = 0;
        let mut backward_idx = total_regions - 1;

        while forward_idx < backward_idx {
            // Convert global indices to (block_idx, region_idx)
            let (forward_block_idx, forward_region_idx) =
                self.get_block_and_region_idx(forward_idx);
            let (backward_block_idx, backward_region_idx) =
                self.get_block_and_region_idx(backward_idx);

            // Check space availability
            let has_free_space = !self.0[forward_block_idx].regions[forward_region_idx]
                .map
                .free_space
                .is_empty();
            let has_blocks = !self.0[backward_block_idx].regions[backward_region_idx]
                .map
                .blocks
                .is_empty();

            if !has_free_space {
                forward_idx += 1;
                continue;
            }
            if !has_blocks {
                backward_idx -= 1;
                continue;
            }

            // Safe to perform block move
            let block = self.0[backward_block_idx].regions[backward_region_idx]
                .map
                .pop_block()?;
            self.0[forward_block_idx].regions[forward_region_idx]
                .map
                .push_block(block)?;
        }

        dbg!(format!("{}", self));

        Ok(())
    }

    fn get_block_and_region_idx(&self, global_idx: usize) -> (usize, usize) {
        let mut remaining = global_idx;
        for (block_idx, block) in self.0.iter().enumerate() {
            if remaining < block.regions.len() {
                return (block_idx, remaining);
            }
            remaining -= block.regions.len();
        }
        panic!("Index out of bounds")
    }

    fn checksum(&self) -> Result<u64> {
        // Convert the disk map to a string of file IDs with dots for free space
        let packed_state = format!("{}", self);

        // Calculate checksum by multiplying each position by its file ID
        packed_state
            .char_indices()
            .filter(|(_, c)| *c != '.') // Skip free space
            .try_fold(0_u64, |acc, (pos, c)| {
                let file_id =
                    c.to_digit(10)
                        .ok_or_else(|| miette!("Invalid digit: {c}"))? as u64;

                let product = (pos as u64)
                    .checked_mul(file_id)
                    .ok_or_else(|| miette!("Checksum multiplication overflow"))?;

                acc.checked_add(product)
                    .ok_or_else(|| miette!("Checksum addition overflow"))
            })
    }
}

impl std::fmt::Display for DiskMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // First collect all blocks
        let mut all_blocks = String::new();
        for block in &self.0 {
            for region in &block.regions {
                all_blocks.push_str(&region.map.blocks.iter().collect::<String>());
            }
        }

        // Then all free space
        let total_free_space: usize = self
            .0
            .iter()
            .flat_map(|block| &block.regions)
            .map(|region| region.map.free_space.len())
            .sum();

        write!(f, "{}{}", all_blocks, ".".repeat(total_free_space))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct DiskBlock {
    regions: Vec<Region>,
    block_id: usize,
}

impl DiskBlock {
    fn new(regions: Vec<Region>, id: usize) -> Self {
        Self {
            regions,
            block_id: id,
        }
    }
}

#[tracing::instrument]
pub fn process(input: &str) -> Result<String> {
    let mut disk_map = parse(input.trim())?;
    disk_map.pack()?;

    // Write packed state to file
    write("packed_output.txt", format!("{}", disk_map))
        .map_err(|e| miette!("Failed to write output: {}", e))?;

    Ok(disk_map.checksum()?.to_string())
}

fn parse(input: &str) -> Result<DiskMap> {
    if input.is_empty() {
        return Err(miette!("Empty input"));
    }

    // Find non-digit characters with their positions
    if let Some((pos, c)) = input.chars().enumerate().find(|(_, c)| !c.is_ascii_digit()) {
        return Err(InvalidCharError::new(input, pos, c).into());
    }

    // First convert input into pairs
    let pairs: Vec<(char, Option<char>)> = input
        .chars()
        .enumerate()
        .fold(Vec::new(), |mut acc, (i, c)| {
            if i % 2 == 0 {
                acc.push((c, None));
            } else if let Some(last) = acc.last_mut() {
                last.1 = Some(c);
            }
            acc
        });

    // Then create blocks with exactly 10 pairs each (except possibly the last block)
    let blocks = pairs
        .chunks(10)
        .enumerate()
        .map(|(block_id, chunk)| {
            let regions = chunk
                .iter()
                .enumerate()
                .map(|(local_id, (block_size, maybe_free_size))| {
                    // Wrap IDs around 0-9
                    let id = (block_id * chunk.len() + local_id) % 10;
                    
                    let block_size = block_size
                        .to_digit(10)
                        .ok_or_else(|| InvalidBlockSizeError::new(input, local_id * 2, *block_size))?
                        as usize;

                    let free_size = maybe_free_size
                        .map(|c| c.to_digit(10))
                        .unwrap_or(Some(0))
                        .ok_or_else(|| {
                            InvalidFreeSizeError::new(input, local_id * 2 + 1, maybe_free_size.unwrap())
                        })? as usize;

                    Region::new(block_size, free_size, id)
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(DiskBlock::new(regions, block_id))
        })
        .collect::<Result<Vec<_>>>()?;

    tracing::debug!("Parsed {} blocks from {} pairs", blocks.len(), pairs.len());

    Ok(DiskMap(blocks))
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
    fn test_parser() -> Result<()> {
        let input = "12345";
        let expected = DiskMap(vec![DiskBlock {
            regions: vec![
                Region::new(1, 2, 0)?,
                Region::new(3, 4, 1)?,
                Region::new(5, 0, 2)?,
            ],
            block_id: 0,
        }]);

        assert_eq!(expected, parse(input)?);

        Ok(())
    }

    #[test]
    fn test_parser_long() -> Result<()> {
        let input = "123456789";
        let expected = DiskMap(vec![DiskBlock {
            regions: vec![
                Region::new(1, 2, 0)?,
                Region::new(3, 4, 1)?,
                Region::new(5, 6, 2)?,
                Region::new(7, 8, 3)?,
                Region::new(9, 0, 4)?,
            ],
            block_id: 0,
        }]);

        assert_eq!(expected, parse(input)?);

        Ok(())
    }

    #[test]
    fn test_parser_long2() -> Result<()> {
        let input = "12345678901234567890";
        let expected = DiskMap(vec![
            (DiskBlock {
                regions: vec![
                    Region::new(1, 2, 0)?,
                    Region::new(3, 4, 1)?,
                    Region::new(5, 6, 2)?,
                    Region::new(7, 8, 3)?,
                    Region::new(9, 0, 4)?,
                    Region::new(1, 2, 5)?,
                    Region::new(3, 4, 6)?,
                    Region::new(5, 6, 7)?,
                    Region::new(7, 8, 8)?,
                    Region::new(9, 0, 9)?,
                ],
                block_id: 0,
            }),
        ]);

        assert_eq!(expected, parse(input)?);

        Ok(())
    }

    #[test]
    fn test_parser_long3() -> Result<()> {
        let input = "1234567890123456789012345678901234567890";
        let expected = DiskMap(vec![
            (DiskBlock {
                regions: vec![
                    Region::new(1, 2, 0)?,
                    Region::new(3, 4, 1)?,
                    Region::new(5, 6, 2)?,
                    Region::new(7, 8, 3)?,
                    Region::new(9, 0, 4)?,
                    Region::new(1, 2, 5)?,
                    Region::new(3, 4, 6)?,
                    Region::new(5, 6, 7)?,
                    Region::new(7, 8, 8)?,
                    Region::new(9, 0, 9)?,
                ],
                block_id: 0,
            }),
            (DiskBlock {
                regions: vec![
                    Region::new(1, 2, 0)?,
                    Region::new(3, 4, 1)?,
                    Region::new(5, 6, 2)?,
                    Region::new(7, 8, 3)?,
                    Region::new(9, 0, 4)?,
                    Region::new(1, 2, 5)?,
                    Region::new(3, 4, 6)?,
                    Region::new(5, 6, 7)?,
                    Region::new(7, 8, 8)?,
                    Region::new(9, 0, 9)?,
                ],
                block_id: 1,
            }),
        ]);

        assert_eq!(expected, parse(input)?);

        Ok(())
    }

    #[test_log::test]
    fn test_parser_invalid_input() -> Result<()> {
        let input = "123A45";
        assert!(parse(input).is_err());
        Ok(())
    }
}
