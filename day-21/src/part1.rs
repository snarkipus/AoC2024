use crate::directional::create_directional_keypad;
use crate::numeric::create_numeric_keypad;
use rayon::prelude::*;
use std::collections::HashMap;

pub const ROBOT_LEVELS: usize = 1;

pub fn process(input: &str) -> miette::Result<(HashMap<String, String>, usize)> {
    let input_sequences: Vec<String> = input.lines().map(|s| s.to_string()).collect();

    // Process sequences in parallel
    let solutions: HashMap<_, _> = input_sequences
        .par_iter() // Parallel iterator
        .map(|sequence| {
            let numeric_keypad = create_numeric_keypad();
            let directional_keypad = create_directional_keypad();

            // Level 1: Initial encoding
            let initial = numeric_keypad.encode_sequence(sequence, None)?;

            // Process robot levels sequentially since each level depends on the previous
            let mut current = initial;
            let mut results = Vec::with_capacity(ROBOT_LEVELS);

            for _ in 0..ROBOT_LEVELS {
                let next = directional_keypad.encode_sequence(&current, None)?;
                results.push(next.clone());
                current = next;
            }

            // Join intermediate results
            let robot_output = results.join("");

            // Final encoding
            let final_sequence = directional_keypad.encode_sequence(&robot_output, None)?;
            Ok((sequence.clone(), final_sequence))
        })
        .collect::<miette::Result<HashMap<_, _>>>()?;

    // Calculate complexity in parallel
    let complexity = solutions
        .par_iter()
        .map(|(k, v)| {
            let key_nums = k
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .trim_start_matches('0')
                .parse::<usize>()
                .unwrap_or(0);
            key_nums * v.len()
        })
        .sum();

    Ok((solutions, complexity))
}

#[cfg(test)]
mod tests {
    use crate::{keypads::Key, numeric::NumericKey};

    use super::*;

    #[allow(dead_code)]
    fn validate_path(path: &str) -> bool {
        path.chars().all(|c| ['<', '>', '^', 'v', 'A'].contains(&c)) && path.ends_with('A')
    }

    // Paths are equivalent if they have the same length and the same number of 'A's
    fn validate_ouput(sequence: &str) -> Vec<usize> {
        vec![
            sequence.len(),
            sequence.chars().filter(|c| *c == 'A').count(),
        ]
    }

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "\
029A
980A
179A
456A
379A";
        let expected: HashMap<String, String> = vec![
            (
                "029A",
                "<vA<AA>>^AvAA<^A>A<v<A>>^AvA^A<vA>^A<v<A>^A>AAvA^A<v<A>A>^AAAvA<^A>A",
            ),
            (
                "980A",
                "<v<A>>^AAAvA^A<vA<AA>>^AvAA<^A>A<v<A>A>^AAAvA<^A>A<vA>^A<A>A",
            ),
            (
                "179A",
                "<v<A>>^A<vA<A>>^AAvAA<^A>A<v<A>>^AAvA^A<vA>^AA<A>A<v<A>A>^AAAvA<^A>A",
            ),
            (
                "456A",
                "<v<A>>^AA<vA<A>>^AAvAA<^A>A<vA>^A<A>A<vA>^A<A>A<v<A>A>^AAvA<^A>A",
            ),
            (
                "379A",
                "<v<A>>^AvA^A<vA<AA>>^AAvA<^A>AAvA^A<vA>^AA<A>A<v<A>A>^AAAvA<^A>A",
            ),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

        let (result, complexity) = process(input)?;
        for (k, v) in expected {
            assert_eq!(validate_ouput(result.get(&k).unwrap()), validate_ouput(&v));
        }

        assert_eq!(complexity, 126384);
        Ok(())
    }

    #[test]
    fn test_basic_numeric_keypad() -> miette::Result<()> {
        let numeric_keypad = create_numeric_keypad();
        let test_cases = vec![
            ("029A", "<A^A>^^AvvvA"),
            ("029A", "<A^A^>^AvvvA"),
            ("029A", "<A^A^^>AvvvA"),
        ];

        for (input, expected) in test_cases {
            let result = numeric_keypad.encode_sequence(input, None)?;
            assert_eq!(
                validate_ouput(&result),
                validate_ouput(&expected),
                "A position mismatch for input '{}'. Got: {} Expected: {}",
                input,
                result,
                expected
            );
        }
        Ok(())
    }

    #[test]
    fn test_basic_directional_keypad() -> miette::Result<()> {
        let directional_keypad = create_directional_keypad();
        let test_cases = vec![
            // The sequence that Robot 2 should type to make Robot 1 type "<A^A>^^AvvvA"
            ("<A^A>^^AvvvA", "v<<A>>^A<A>AvA<^AA>A<vAAA>^A"),
        ];

        for (input, expected) in test_cases {
            let result = directional_keypad.encode_sequence(input, None)?;
            assert_eq!(
                validate_ouput(&result),
                validate_ouput(&expected),
                "A position mismatch for input '{}'. Got: {} Expected: {}",
                input,
                result,
                expected
            );
        }
        Ok(())
    }

    #[test]
    fn test_full_encoding_chain() -> miette::Result<()> {
        let numeric_keypad = create_numeric_keypad();
        let directional_keypad = create_directional_keypad();

        let door_code = "029A";
        let robot1_sequence = "<A^A>^^AvvvA";
        let robot2_sequence = "v<<A>>^A<A>AvA<^AA>A<vAAA>^A";
        let robot3_sequence =
            "<vA<AA>>^AvAA<^A>A<v<A>>^AvA^A<vA>^A<v<A>^A>AAvA^A<v<A>A>^AAAvA<^A>A";

        let level1 = numeric_keypad.encode_sequence(door_code, None)?;
        println!("Level 1: {}", level1);
        assert_eq!(
            validate_ouput(&level1),
            validate_ouput(&robot1_sequence),
            "Robot 1 sequence mismatch\nGot: {}\nExpected: {}",
            level1,
            robot1_sequence
        );

        let level2 = directional_keypad.encode_sequence(&level1, None)?;
        println!("Level 2: {}", level2);
        assert_eq!(
            validate_ouput(&level2),
            validate_ouput(&robot2_sequence),
            "Robot 2 sequence mismatch\nGot: {}\nExpected: {}",
            level2,
            robot2_sequence
        );

        let level3 = directional_keypad.encode_sequence(&level2, None)?;
        println!("Level 3: {}", level3);
        assert_eq!(
            validate_ouput(&level3),
            validate_ouput(&robot3_sequence),
            "Robot 3 sequence mismatch\nGot: {}\nExpected: {}",
            level3,
            robot3_sequence
        );

        Ok(())
    }

    #[test]
    fn test_debug_chain() -> miette::Result<()> {
        let numeric_keypad = create_numeric_keypad();
        let directional_keypad = create_directional_keypad();

        // Simple test case
        let input = "02";
        let expected_l1 = "<A^A"; // From A->0->2
        let expected_l2 = "<A^A>^^AvvvA"; // Encode directions from l1

        let level1 = numeric_keypad.encode_sequence(input, None)?;
        println!("\nLevel 1:");
        println!("Input:    {}", input);
        println!("Expected: {}", expected_l1);
        println!("Got:      {}", level1);
        println!("Expected [len, cnt]: {:?}", validate_ouput(&expected_l1));
        println!("Got [len, cnt]: {:?}", validate_ouput(&level1));

        let level2 = directional_keypad.encode_sequence(&level1, None)?;
        println!("\nLevel 2:");
        println!("Input:    {}", level1);
        println!("Expected: {}", expected_l2);
        println!("Got:      {}", level2);
        println!("Expected [len, cnt]: {:?}", validate_ouput(&expected_l2));
        println!("Got [len, cnt]: {:?}", validate_ouput(&level2));

        Ok(())
    }

    #[test]
    fn test_debug_chain_2() -> miette::Result<()> {
        let numeric_keypad = create_numeric_keypad();
        let directional_keypad = create_directional_keypad();

        // Simple test case
        let input = "29";

        // Numeric keypad layout:
        // +---+---+---+
        // | 7 | 8 | 9 |
        // +---+---+---+
        // | 4 | 5 | 6 |
        // +---+---+---+
        // | 1 | 2 | 3 |
        // +---+---+---+
        //     | 0 | A |
        //     +---+---+

        let valid_l1_outputs = vec![
            "<^A^^>A", // (1) [A->2: left+up+A],[2->9: up+up+right+A]
            "<^A^>^A", // (2) [A->2: left+up+A],[2->9: up+right+up+A]
            "<^A>^^A", // (3) [A->2: left+up+A],[2->9: right+up+up+A]
            "^<A^^>A", // (4) [A->2: up+left+A],[2->9: up+up+right+A]
            "^<A^>^A", // (5) [A->2: up+left+A],[2->9: up+right+up+A]
            "^<A>^^A", // (6) [A->2: up+left+A],[2->9: right+up+up+A]
        ]; // From A->2->9+A

        // Directional keypad layout:
        //     +---+---+
        //     | ^ | A |
        // +---+---+---+
        // | < | v | > |
        // +---+---+---+

        // Family of shortest paths from A->2->9:
        // (1.1) [A->2: < ^ A],[2->9: ^ ^ > A]: [A -> <: left+down+left],[A],[< -> ^: right+up], [A],[^ -> A: right],         [A],[A -> ^: left],[A],[^ -> ^: None],      [A],[^ -> >: down+right],[A],[> -> A: up],   [A]
        // (1.2) [A->2: < ^ A],[2->9: ^ ^ > A]: [A -> <: down+left+left],[A],[< -> ^: right+up], [A],[^ -> A: right],         [A],[A -> ^: left],[A],[^ -> ^: None],      [A],[^ -> >: down+right],[A],[> -> A: up],   [A]
        // (2.1) [A->2: < ^ A],[2->9: ^ > ^ A]: [A -> <: left+down+left],[A],[< -> ^: right+up], [A],[^ -> A: right],         [A],[A -> ^: left],[A],[^ -> >: down+right],[A],[> -> ^: up+left],   [A],[^ -> A: right],[A]
        // (2.2) [A->2: < ^ A],[2->9: ^ > ^ A]: [A -> <: left+down+left],[A],[< -> ^: right+up], [A],[^ -> A: right],         [A],[A -> ^: left],[A],[^ -> >: down+right],[A],[> -> ^: left+up],   [A],[^ -> A: right],[A]
        // (2.3) [A->2: < ^ A],[2->9: ^ > ^ A]: [A -> <: left+down+left],[A],[< -> ^: right+up], [A],[^ -> A: right],         [A],[A -> ^: left],[A],[^ -> >: right+down],[A],[> -> ^: up+left],   [A],[^ -> A: right],[A]
        // (2.4) [A->2: < ^ A],[2->9: ^ > ^ A]: [A -> <: left+down+left],[A],[< -> ^: right+up], [A],[^ -> A: right],         [A],[A -> ^: left],[A],[^ -> >: right+down],[A],[> -> ^: left+up],   [A],[^ -> A: right],[A]
        // (2.5) [A->2: < ^ A],[2->9: ^ > ^ A]: [A -> <: down+left+left],[A],[< -> ^: right+up], [A],[^ -> A: right],         [A],[A -> ^: left],[A],[^ -> >: down+right],[A],[> -> ^: up+left],   [A],[^ -> A: right],[A]
        // (2.6) [A->2: < ^ A],[2->9: ^ > ^ A]: [A -> <: down+left+left],[A],[< -> ^: right+up], [A],[^ -> A: right],         [A],[A -> ^: left],[A],[^ -> >: down+right],[A],[> -> ^: left+up],   [A],[^ -> A: right],[A]
        // (2.7) [A->2: < ^ A],[2->9: ^ > ^ A]: [A -> <: down+left+left],[A],[< -> ^: right+up], [A],[^ -> A: right],         [A],[A -> ^: left],[A],[^ -> >: right+down],[A],[> -> ^: up+left],   [A],[^ -> A: right],[A]
        // (2.8) [A->2: < ^ A],[2->9: ^ > ^ A]: [A -> <: down+left+left],[A],[< -> ^: right+up], [A],[^ -> A: right],         [A],[A -> ^: left],[A],[^ -> >: right+down],[A],[> -> ^: left+up],   [A],[^ -> A: right],[A]
        // (3.1) [A->2: < ^ A],[2->9: > ^ ^ A]: [A -> <: left+down+left],[A],[< -> ^: right+up], [A],[^ -> A: right],         [A],[A -> >: down],[A],[> -> ^: up+left],   [A],[^ -> ^: None],      [A],[^ -> A: right],[A]
        // (3.2) [A->2: < ^ A],[2->9: > ^ ^ A]: [A -> <: left+down+left],[A],[< -> ^: right+up], [A],[^ -> A: right],         [A],[A -> >: down],[A],[> -> ^: left+up],   [A],[^ -> ^: None],      [A],[^ -> A: right],[A]
        // (3.3) [A->2: < ^ A],[2->9: > ^ ^ A]: [A -> <: down+left+left],[A],[< -> ^: right+up], [A],[^ -> A: right],         [A],[A -> >: down],[A],[> -> ^: up+left],   [A],[^ -> ^: None],      [A],[^ -> A: right],[A]
        // (3.4) [A->2: < ^ A],[2->9: > ^ ^ A]: [A -> <: down+left+left],[A],[< -> ^: right+up], [A],[^ -> A: right],         [A],[A -> >: down],[A],[> -> ^: left+up],   [A],[^ -> ^: None],      [A],[^ -> A: right],[A]
        // (4.1) [A->2: ^ < A],[2->9: ^ ^ > A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+up+right],[A],[A -> ^: left],[A],[^ -> ^: None],      [A],[^ -> >: down+right],[A],[> -> A: up],   [A]
        // (4.2) [A->2: ^ < A],[2->9: ^ ^ > A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+up+right],[A],[A -> ^: left],[A],[^ -> ^: None],      [A],[^ -> >: right+down],[A],[> -> A: up],   [A]
        // (4.3) [A->2: ^ < A],[2->9: ^ ^ > A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+right+up],[A],[A -> ^: left],[A],[^ -> ^: None],      [A],[^ -> >: down+right],[A],[> -> A: up],   [A]
        // (4.4) [A->2: ^ < A],[2->9: ^ ^ > A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+right+up],[A],[A -> ^: left],[A],[^ -> ^: None],      [A],[^ -> >: right+down],[A],[> -> A: up],   [A]
        // (5.1) [A->2: ^ < A],[2->9: ^ > ^ A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+up+right],[A],[A -> ^: left],[A],[^ -> >: down+right],[A],[> -> ^: up+left],   [A],[^ -> A: right],[A]
        // (5.2) [A->2: ^ < A],[2->9: ^ > ^ A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+up+right],[A],[A -> ^: left],[A],[^ -> >: down+right],[A],[> -> ^: left+up],   [A],[^ -> A: right],[A]
        // (5.3) [A->2: ^ < A],[2->9: ^ > ^ A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+up+right],[A],[A -> ^: left],[A],[^ -> >: right+down],[A],[> -> ^: up+left],   [A],[^ -> A: right],[A]
        // (5.4) [A->2: ^ < A],[2->9: ^ > ^ A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+up+right],[A],[A -> ^: left],[A],[^ -> >: right+down],[A],[> -> ^: left+up],   [A],[^ -> A: right],[A]
        // (5.5) [A->2: ^ < A],[2->9: ^ > ^ A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+right+up],[A],[A -> ^: left],[A],[^ -> >: down+right],[A],[> -> ^: up+left],   [A],[^ -> A: right],[A]
        // (5.6) [A->2: ^ < A],[2->9: ^ > ^ A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+right+up],[A],[A -> ^: left],[A],[^ -> >: down+right],[A],[> -> ^: left+up],   [A],[^ -> A: right],[A]
        // (5.7) [A->2: ^ < A],[2->9: ^ > ^ A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+right+up],[A],[A -> ^: left],[A],[^ -> >: right+down],[A],[> -> ^: up+left],   [A],[^ -> A: right],[A]
        // (5.8) [A->2: ^ < A],[2->9: ^ > ^ A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+right+up],[A],[A -> ^: left],[A],[^ -> >: right+down],[A],[> -> ^: left+up],   [A],[^ -> A: right],[A]
        // (6.1) [A->2: ^ < A],[2->9: > ^ ^ A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+up+right],[A],[A -> >: down],[A],[> -> ^: up+left],   [A],[^ -> ^: None],      [A],[^ -> A: right],[A]
        // (6.2) [A->2: ^ < A],[2->9: > ^ ^ A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+up+right],[A],[A -> >: down],[A],[> -> ^: left+up],   [A],[^ -> ^: None],      [A],[^ -> A: right],[A]
        // (6.3) [A->2: ^ < A],[2->9: > ^ ^ A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+right+up],[A],[A -> >: down],[A],[> -> ^: up+left],   [A],[^ -> ^: None],      [A],[^ -> A: right],[A]
        // (6.4) [A->2: ^ < A],[2->9: > ^ ^ A]: [A -> ^: left],          [A],[^ -> <: down+left],[A],[< -> A: right+right+up],[A],[A -> >: down],[A],[> -> ^: left+up],   [A],[^ -> ^: None],      [A],[^ -> A: right],[A]

        let valid_l2_outputs = vec![
            "<v<A>^A>A<AAv>A^A",   // (1.1)
            "v<<A>^A>A<AAv>A^A",   // (1.2)
            "<v<A>^A>A<Av>A^<A>A", // (2.1)
            "<v<A>^A>A<Av>A<^A>A", // (2.2)
            "<v<A>^A>A<A>vA^<A>A", // (2.3)
            "<v<A>^A>A<A>vA<^A>A", // (2.4)
            "v<<A>^A>A<Av>A^<A>A", // (2.5)
            "v<<A>^A>A<Av>A<^A>A", // (2.6)
            "v<<A>^A>A<A>vA^<A>A", // (2.7)
            "v<<A>^A>A<A>vA<^A>A", // (2.8)
            "<v<A>^A>AvA^<AA>A",   // (3.1)
            "<v<A>^A>AvA<^AA>A",   // (3.2)
            "v<<A>^A>AvA^<AA>A",   // (3.3)
            "v<<A>^A>AvA<^AA>A",   // (3.4)
            "<Av<A>^>A<AAv>A^A",   // (4.1)
            "<Av<A>^>A<AA>vA^A",   // (4.2)
            "<Av<A>>^A<AAv>A^A",   // (4.3)
            "<Av<A>>^A<AA>vA^A",   // (4.4)
            "<Av<A>^>A<Av>A^<A>A", // (5.1)
            "<Av<A>^>A<Av>A<^A>A", // (5.2)
            "<Av<A>^>A<A>vA^<A>A", // (5.3)
            "<Av<A>^>A<A>vA<^A>A", // (5.4)
            "<Av<A>>^A<Av>A^<A>A", // (5.5)
            "<Av<A>>^A<Av>A<^A>A", // (5.6)
            "<Av<A>>^A<A>vA^<A>A", // (5.7)
            "<Av<A>>^A<A>vA<^A>A", // (5.8)
            "<Av<A>^>AvA^<AA>A",   // (6.1)
            "<Av<A>^>AvA<^AA>A",   // (6.2)
            "<Av<A>>^AvA^<AA>A",   // (6.3)
            "<Av<A>>^AvA<^AA>A",   // (6.4)
        ];

        let expected_l2 = "<v<A>^A>A<AAv>A^A"; // shortest path encoding of best l1 output

        let level1 = numeric_keypad.encode_sequence(input, None)?;
        println!("\nLevel 1:");
        println!("Input:    {}", input);
        println!("Expected: {:#?}", valid_l1_outputs);
        println!("Got:      {}", level1);
        assert!(valid_l1_outputs.iter().any(|v| v == &level1));
        println!(
            "Expected [len, cnt]: {:?}",
            validate_ouput(&valid_l1_outputs[0])
        );
        println!("Got [len, cnt]: {:?}", validate_ouput(&level1));

        let level2 = directional_keypad.encode_sequence(&level1, None)?;
        println!("\nLevel 2:");
        println!("Input:    {}", level1);
        println!("Expected: {}", expected_l2);
        println!("Got:      {}", level2);
        println!("Expected [len, cnt]: {:?}", validate_ouput(&expected_l2));
        println!("Got [len, cnt]: {:?}", validate_ouput(&level2));

        // validates that we've chosen one of the valid shortest path encodings
        assert!(valid_l2_outputs.iter().any(|v| v == &level2));

        // validates that we've chosen the level1 ouput that results in the shortest path encoding
        assert_eq!(
            level2.len(),
            expected_l2.len(),
            "Non-optimal level 1 path selected: {}",
            level1
        );

        Ok(())
    }

    #[test]
    fn test_find_paths() -> miette::Result<()> {
        let numeric_keypad = create_numeric_keypad();

        let start = NumericKey::from_char('2').unwrap();
        let end = NumericKey::from_char('9').unwrap();

        let paths = numeric_keypad.find_paths(start, end)?;

        // Convert paths to character vectors for easier comparison
        let char_paths: Vec<Vec<char>> = paths
            .iter()
            .map(|path| {
                path.iter()
                    .map(|node| numeric_keypad.graph.as_ref().unwrap()[*node].to_char())
                    .collect()
            })
            .collect();

        println!("Paths Found: {:?}", char_paths);

        // Expected valid paths
        let expected_paths = vec![
            vec!['2', '5', '8', '9'],
            vec!['2', '3', '6', '9'],
            vec!['2', '5', '6', '9'],
        ];

        // Test path count
        assert_eq!(paths.len(), 3, "Expected 3 paths, found {}", paths.len());

        // Test path lengths
        assert!(
            paths.iter().all(|p| p.len() == 4),
            "All paths should have length 4"
        );

        // Test path contents
        assert!(
            expected_paths.iter().all(|exp| char_paths.contains(exp)),
            "Not all expected paths were found"
        );

        Ok(())
    }
}
