use std::collections::HashMap;

// #[tracing::instrument]
pub fn process(input: &str) -> miette::Result<HashMap<String, String>> {
    let mut solutions = HashMap::new();

    let input_sequence: Vec<Vec<char>> = input
        .lines()
        .map(|line| line.chars().collect::<Vec<char>>())
        .collect();

    let mut numeric_keypad = numeric_keypad::NumericKeypad::new();
    numeric_keypad.create_graph();

    let mut directional_keypad = directional_keypad::DirectionalKeypad::new();
    directional_keypad.create_graph();

    for sequence in input_sequence {
        let mut encoded_sequence = String::new();
        let mut start = 'A';

        // Encode the sequence using the numeric keypad
        for c in &sequence {
            let shortest_path = numeric_keypad.shortest_path(start, *c)?;
            encoded_sequence += &numeric_keypad.encode_path_direction(shortest_path)?;
            encoded_sequence.push('A');
            start = *c;
        }

        println!("Encoded Sequence #1: \t{}", encoded_sequence);

        // Encode the previous sequence using the directional keypad
        let mut encoded_sequence_2 = String::new();
        let mut start = 'A';

        for c in encoded_sequence.chars() {
            let shortest_path = directional_keypad.shortest_path(start, c)?;
            encoded_sequence_2 += &directional_keypad.encode_path_direction(shortest_path)?;
            encoded_sequence_2.push('A');
            start = c;
        }

        println!("Encoded Sequence #2: \t{}", encoded_sequence_2);

        // Encode the previous sequence using the directional keypad
        let mut encoded_sequence_3 = String::new();
        let mut start = 'A';

        for c in encoded_sequence_2.chars() {
            let shortest_path = directional_keypad.shortest_path(start, c)?;
            encoded_sequence_3 += &directional_keypad.encode_path_direction(shortest_path)?;
            encoded_sequence_3.push('A');
            start = c;
        }

        println!("Encoded Sequence #3: \t{}", encoded_sequence_3);

        let sequence_string = sequence.iter().collect::<String>();
        solutions.insert(sequence_string, encoded_sequence_3.clone());
    }

    Ok(solutions)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position(usize, usize);

mod numeric_keypad {
    use petgraph::algo::astar;
    use petgraph::graph::{NodeIndex, UnGraph};

    use super::*;
    use std::{collections::HashMap, fmt};

    // Numeric Kepyad
    // +---+---+---+
    // | 7 | 8 | 9 |
    // +---+---+---+
    // | 4 | 5 | 6 |
    // +---+---+---+
    // | 1 | 2 | 3 |
    // +---+---+---+
    //     | 0 | A |
    //     +---+---+

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Numeric {
        One,   // 1
        Two,   // 2
        Three, // 3
        Four,  // 4
        Five,  // 5
        Six,   // 6
        Seven, // 7
        Eight, // 8
        Nine,  // 9
        Zero,  // 0
        A,     // A
        Blank, // NO BUTTON
    }

    impl fmt::Display for Numeric {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let c = match self {
                Numeric::One => '1',
                Numeric::Two => '2',
                Numeric::Three => '3',
                Numeric::Four => '4',
                Numeric::Five => '5',
                Numeric::Six => '6',
                Numeric::Seven => '7',
                Numeric::Eight => '8',
                Numeric::Nine => '9',
                Numeric::Zero => '0',
                Numeric::A => 'A',
                Numeric::Blank => ' ',
            };
            write!(f, "{}", c)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct NumericKeypadCell {
        position: Position,
        pub value: Numeric,
    }

    impl NumericKeypadCell {
        fn new(value: Numeric, position: Position) -> Self {
            Self { value, position }
        }
    }

    impl fmt::Display for NumericKeypadCell {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.value)
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) struct NumericKeypad {
        cells: Vec<Vec<NumericKeypadCell>>,
        pub(crate) graph: Option<UnGraph<NumericKeypadCell, ()>>,
    }

    impl NumericKeypad {
        pub(crate) fn new() -> Self {
            let mut cells = Vec::new();
            // Row 1 - Buttons (width: 3)
            cells.push(vec![
                NumericKeypadCell::new(Numeric::Seven, Position(0, 0)),
                NumericKeypadCell::new(Numeric::Eight, Position(1, 0)),
                NumericKeypadCell::new(Numeric::Nine, Position(2, 0)),
            ]);
            // Row 2 - Buttons (width: 3)
            cells.push(vec![
                NumericKeypadCell::new(Numeric::Four, Position(0, 1)),
                NumericKeypadCell::new(Numeric::Five, Position(1, 1)),
                NumericKeypadCell::new(Numeric::Six, Position(2, 1)),
            ]);
            // Row 3 - Buttons (width: 3)
            cells.push(vec![
                NumericKeypadCell::new(Numeric::One, Position(0, 2)),
                NumericKeypadCell::new(Numeric::Two, Position(1, 2)),
                NumericKeypadCell::new(Numeric::Three, Position(2, 2)),
            ]);
            // Row 4 - Buttons (width: 3)
            cells.push(vec![
                NumericKeypadCell::new(Numeric::Blank, Position(0, 3)),
                NumericKeypadCell::new(Numeric::Zero, Position(1, 3)),
                NumericKeypadCell::new(Numeric::A, Position(2, 3)),
            ]);

            Self { cells, graph: None }
        }

        pub(crate) fn create_graph(&mut self) {
            let mut graph = UnGraph::<NumericKeypadCell, ()>::new_undirected();
            let mut nodes = HashMap::new();

            // Add nodes for each row and column
            for row in &self.cells {
                for cell in row {
                    let node = graph.add_node(*cell);
                    nodes.insert(cell.position, node);
                }
            }

            // Add edges between adjacent nodes
            for (y, row) in self.cells.iter().enumerate() {
                for (x, cell) in row.iter().enumerate() {
                    let node = nodes[&cell.position];
                    if cell.value == Numeric::Blank {
                        continue;
                    }
                    if x > 0 {
                        let left = nodes[&Position(x - 1, y)];
                        if graph[left].value != Numeric::Blank {
                            graph.add_edge(node, left, ());
                        }
                    }
                    if y > 0 {
                        let up = nodes[&Position(x, y - 1)];
                        if graph[up].value != Numeric::Blank {
                            graph.add_edge(node, up, ());
                        }
                    }
                }
            }

            self.graph = Some(graph);
        }

        pub(crate) fn shortest_path(
            &self,
            start: char,
            end: char,
        ) -> miette::Result<Vec<NodeIndex>> {
            let start_node = self.find_cell(start)?;
            let end_node = self.find_cell(end)?;

            if start == end {
                return Ok(vec![start_node]);
            }

            let graph = self
                .graph
                .as_ref()
                .ok_or(miette::miette!("Graph not found!"))?;

            let (_cost, path) = astar(graph, start_node, |n| n == end_node, |_| 1, |_| 0)
                .ok_or(miette::miette!("No path found!"))?;

            Ok(path)
        }

        pub(crate) fn encode_path_direction(&self, path: Vec<NodeIndex>) -> miette::Result<String> {
            let graph = self
                .graph
                .as_ref()
                .ok_or(miette::miette!("Graph not found!"))?;

            let mut encoded_path = String::new();

            for (idx, node) in path.iter().skip(1).enumerate() {
                let cell_position = &graph[*node].position;
                let prev_cell_position = &graph[path[idx]].position;

                let dx = cell_position.0 as isize - prev_cell_position.0 as isize;
                let dy = cell_position.1 as isize - prev_cell_position.1 as isize;

                let encoded_direction = match (dx, dy) {
                    (0, -1) => "^",
                    (0, 1) => "v",
                    (-1, 0) => "<",
                    (1, 0) => ">",
                    _ => return Err(miette::miette!("Invalid path!")),
                };

                encoded_path.push_str(encoded_direction);
            }

            Ok(encoded_path)
        }

        fn find_cell(&self, value: char) -> miette::Result<NodeIndex> {
            let value = match value {
                '1' => Numeric::One,
                '2' => Numeric::Two,
                '3' => Numeric::Three,
                '4' => Numeric::Four,
                '5' => Numeric::Five,
                '6' => Numeric::Six,
                '7' => Numeric::Seven,
                '8' => Numeric::Eight,
                '9' => Numeric::Nine,
                '0' => Numeric::Zero,
                'A' => Numeric::A,
                _ => return Err(miette::miette!("Invalid value: {}", value)),
            };

            let graph = self
                .graph
                .as_ref()
                .ok_or(miette::miette!("Graph not found"))?;

            if let Some(node) = graph.node_indices().find(|&n| graph[n].value == value) {
                Ok(node)
            } else {
                Err(miette::miette!("Node not found for value: {}", value))
            }
        }

        pub fn display(&self) {
            print!("{}", self);
        }
    }

    impl fmt::Display for NumericKeypad {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            for row in &self.cells {
                // Top border
                writeln!(f, "{}", "+---".repeat(row.len()) + "+")?;

                // Cell values
                for cell in row {
                    write!(f, "| {} ", cell)?;
                }
                writeln!(f, "|")?;
            }
            // Bottom border
            writeln!(f, "{}", "+---".repeat(self.cells[0].len()) + "+")?;
            Ok(())
        }
    }
}

mod directional_keypad {
    use petgraph::{
        algo::astar,
        graph::{NodeIndex, UnGraph},
    };

    use super::*;
    use std::{collections::HashMap, fmt};

    // Directional Keypad
    //     +---+---+
    //     | ^ | A |
    // +---+---+---+
    // | < | v | > |
    // +---+---+---+

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Directional {
        Up,    // ^
        Down,  // v
        Left,  // <
        Right, // >
        A,     // A
        Blank, // NO BUTTON
    }

    impl fmt::Display for Directional {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let c = match self {
                Directional::Up => '^',
                Directional::Down => 'v',
                Directional::Left => '<',
                Directional::Right => '>',
                Directional::A => 'A',
                Directional::Blank => ' ',
            };
            write!(f, "{}", c)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct DirectionKeypadCell {
        position: Position,
        pub value: Directional,
    }

    impl DirectionKeypadCell {
        fn new(value: Directional, position: Position) -> Self {
            Self { value, position }
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) struct DirectionalKeypad {
        cells: Vec<Vec<DirectionKeypadCell>>,
        pub(crate) graph: Option<UnGraph<DirectionKeypadCell, ()>>,
    }

    impl DirectionalKeypad {
        pub(crate) fn new() -> Self {
            let mut cells = Vec::new();
            // Row 1 - Buttons (width: 3)
            cells.push(vec![
                DirectionKeypadCell::new(Directional::Blank, Position(0, 0)),
                DirectionKeypadCell::new(Directional::Up, Position(1, 0)),
                DirectionKeypadCell::new(Directional::A, Position(2, 0)),
            ]);
            // Row 2 - Buttons (width: 3)
            cells.push(vec![
                DirectionKeypadCell::new(Directional::Left, Position(0, 1)),
                DirectionKeypadCell::new(Directional::Down, Position(1, 1)),
                DirectionKeypadCell::new(Directional::Right, Position(2, 1)),
            ]);
            Self { cells, graph: None }
        }

        pub(crate) fn create_graph(&mut self) {
            let mut graph = UnGraph::<DirectionKeypadCell, ()>::new_undirected();
            let mut nodes = HashMap::new();

            // Add nodes for each row and column
            for row in &self.cells {
                for cell in row {
                    let node = graph.add_node(*cell);
                    nodes.insert(cell.position, node);
                }
            }

            // Add edges between adjacent nodes
            for (y, row) in self.cells.iter().enumerate() {
                for (x, cell) in row.iter().enumerate() {
                    let node = nodes[&cell.position];
                    if cell.value == Directional::Blank {
                        continue;
                    }
                    if x > 0 {
                        let left = nodes[&Position(x - 1, y)];
                        if graph[left].value != Directional::Blank {
                            graph.add_edge(node, left, ());
                        }
                    }
                    if y > 0 {
                        let up = nodes[&Position(x, y - 1)];
                        if graph[up].value != Directional::Blank {
                            graph.add_edge(node, up, ());
                        }
                    }
                }
            }

            self.graph = Some(graph);
        }

        pub(crate) fn shortest_path(
            &self,
            start: char,
            end: char,
        ) -> miette::Result<Vec<NodeIndex>> {
            let start_node = self.find_cell(start)?;
            let end_node = self.find_cell(end)?;

            if start == end {
                return Ok(vec![start_node]);
            }

            let graph = self
                .graph
                .as_ref()
                .ok_or(miette::miette!("Graph not found!"))?;

            let (_cost, path) = astar(graph, start_node, |n| n == end_node, |_| 1, |_| 0)
                .ok_or(miette::miette!("No path found!"))?;

            Ok(path)
        }

        pub(crate) fn encode_path_direction(&self, path: Vec<NodeIndex>) -> miette::Result<String> {
            let graph = self
                .graph
                .as_ref()
                .ok_or(miette::miette!("Graph not found!"))?;

            let mut encoded_path = String::new();

            for (idx, node) in path.iter().skip(1).enumerate() {
                let cell_position = &graph[*node].position;
                let prev_cell_position = &graph[path[idx]].position;

                let dx = cell_position.0 as isize - prev_cell_position.0 as isize;
                let dy = cell_position.1 as isize - prev_cell_position.1 as isize;

                let encoded_direction = match (dx, dy) {
                    (0, -1) => "^",
                    (0, 1) => "v",
                    (-1, 0) => "<",
                    (1, 0) => ">",
                    _ => return Err(miette::miette!("Invalid path!")),
                };

                encoded_path.push_str(encoded_direction);
            }

            Ok(encoded_path)
        }

        fn find_cell(&self, value: char) -> miette::Result<NodeIndex> {
            let value = match value {
                '^' => Directional::Up,
                '<' => Directional::Left,
                '>' => Directional::Right,
                'v' => Directional::Down,
                'A' => Directional::A,
                _ => return Err(miette::miette!("Invalid value: {}", value)),
            };

            let graph = self
                .graph
                .as_ref()
                .ok_or(miette::miette!("Graph not found"))?;

            if let Some(node) = graph.node_indices().find(|&n| graph[n].value == value) {
                Ok(node)
            } else {
                Err(miette::miette!("Node not found for value: {}", value))
            }
        }

        pub fn display(&self) {
            for row in &self.cells {
                // Print top border
                println!("{}", "+---".repeat(row.len()) + "+");

                // Print cell values with vertical borders
                for cell in row {
                    print!("| {} ", cell.value);
                }
                println!("|");
            }
            // Print bottom border for last row
            println!("{}", "+---".repeat(self.cells[0].len()) + "+");
        }
    }

    impl fmt::Display for DirectionalKeypad {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            for row in &self.cells {
                for cell in row {
                    write!(f, "{}", cell.value)?;
                }
                writeln!(f)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "\
029A
980A
179A
456A
379A";

        let possible_output: HashMap<&str, &str> = HashMap::from([
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
        ]);

        let solutions = process(input)?;

        for (input, output) in possible_output {
            let solution = solutions.get(input).unwrap();
            let solution_len = solution.len();
            let solution_a_positions: Vec<_> = solution
                .chars()
                .enumerate()
                .filter(|(_, c)| *c == 'A')
                .map(|(i, _)| i)
                .collect();
            let solution_a_count = solution.chars().filter(|c| *c == 'A').count();

            let output_len = output.len();
            let output_a_positions: Vec<_> = output
                .chars()
                .enumerate()
                .filter(|(_, c)| *c == 'A')
                .map(|(i, _)| i)
                .collect();
            let output_a_count = output.chars().filter(|c| *c == 'A').count();

            // assert_eq!(solution_len, output_len);
            // assert_eq!(solution_a_positions, output_a_positions);
            assert_eq!(
                solution_a_count, output_a_count,
                "\nInput: \t{}\nExpected: \t{}\nBad Output: \t{}",
                input, solution, output
            );

            println!("Input: \t{}", &input);
            println!("Expected: \t{}", &solution);
            println!("Good Output: \t{}\n", &output);
        }
        Ok(())
    }

    #[test]
    fn test_process2() -> miette::Result<()> {
        let input = "029A";
        let possible_output =
            "<vA<AA>>^AvAA<^A>A<v<A>>^AvA^A<vA>^A<v<A>^A>AAvA^A<v<A>A>^AAAvA<^A>A";

        // Get solution for single sequence
        let solutions = process(&input)?;
        let actual = solutions.get(input).unwrap();

        // Split paths into segments between A's
        let expected_segments: Vec<_> = possible_output.split('A').collect();
        let actual_segments: Vec<_> = actual.split('A').collect();

        assert_eq!(
            expected_segments.len(),
            actual_segments.len(),
            "Expected {} segments, got {}",
            expected_segments.len(),
            actual_segments.len()
        );

        // Print segment comparisons
        for (i, (exp, act)) in expected_segments
            .iter()
            .zip(actual_segments.iter())
            .enumerate()
        {
            if exp != act {
                println!("Segment {} mismatch:", i);
                println!("  Expected: {}", exp);
                println!("  Actual  : {}", act);
            }
        }

        assert_eq!(
            possible_output.len(),
            actual.len(),
            "Expected length {}, got {}",
            possible_output.len(),
            actual.len()
        );

        Ok(())
    }

    #[test]
    fn test_process_single() -> miette::Result<()> {
        let input = "029A";
        // assert_eq!("", process(input)?);
        Ok(())
    }

    #[test]
    #[ignore = "display only"]
    fn test_numeric_keypad_display() {
        let keypad = numeric_keypad::NumericKeypad::new();
        keypad.display();
    }

    #[test]
    #[ignore = "display only"]
    fn test_direction_keypad_display() {
        let keypad = directional_keypad::DirectionalKeypad::new();
        keypad.display();
    }

    #[test]
    fn test_numeric_keypad_create_graph() {
        let mut keypad = numeric_keypad::NumericKeypad::new();
        keypad.create_graph();

        for edge in keypad.graph.as_ref().unwrap().edge_indices() {
            if let Some((source, target)) = keypad.graph.as_ref().unwrap().edge_endpoints(edge) {
                println!(
                    "{} <-> {}",
                    keypad.graph.as_ref().unwrap()[source].value,
                    keypad.graph.as_ref().unwrap()[target].value
                );
            }
        }

        assert_eq!(keypad.graph.is_some(), true);
        assert_eq!(keypad.graph.as_ref().unwrap().node_count(), 12);
        assert_eq!(keypad.graph.as_ref().unwrap().edge_count(), 15);
    }

    #[test]
    fn test_direction_keypad_create_graph() {
        let mut keypad = directional_keypad::DirectionalKeypad::new();
        keypad.create_graph();

        for edge in keypad.graph.as_ref().unwrap().edge_indices() {
            if let Some((source, target)) = keypad.graph.as_ref().unwrap().edge_endpoints(edge) {
                println!(
                    "{} <-> {}",
                    keypad.graph.as_ref().unwrap()[source].value,
                    keypad.graph.as_ref().unwrap()[target].value
                );
            }
        }

        assert_eq!(keypad.graph.is_some(), true);
        assert_eq!(keypad.graph.as_ref().unwrap().node_count(), 6);
        assert_eq!(keypad.graph.as_ref().unwrap().edge_count(), 5);
    }

    #[test]
    fn test_numeric_encoding() -> miette::Result<()> {
        let mut keypad = numeric_keypad::NumericKeypad::new();
        keypad.create_graph();

        let input_sequence = "029A".chars().collect::<Vec<char>>();
        let mut encoded_path = String::new();
        let mut start = 'A';

        for c in input_sequence {
            let shortest_path = keypad.shortest_path(start, c)?;
            encoded_path += &keypad.encode_path_direction(shortest_path)?;
            encoded_path.push('A');
            start = c;
        }

        assert!(
            encoded_path == "<A^A>^^AvvvA"
                || encoded_path == "<A^A^>^AvvvA"
                || encoded_path == "<A^A^^>AvvvA"
        );

        Ok(())
    }

    #[test]
    fn test_directional_encoding() -> miette::Result<()> {
        let mut keypad = directional_keypad::DirectionalKeypad::new();
        keypad.create_graph();

        let input_sequence = "<A^A>^^AvvvA".chars().collect::<Vec<char>>();

        let possible_output = "v<<A>>^A<A>AvA<^AA>A<vAAA>^A";
        let output_len = possible_output.len();
        let output_a_positions: Vec<_> = possible_output
            .chars()
            .enumerate()
            .filter(|(_, c)| *c == 'A')
            .map(|(i, _)| i)
            .collect();
        let output_a_count = output_a_positions.len();

        let mut encoded_path = String::new();
        let mut start = 'A';

        for c in input_sequence.clone() {
            let shortest_path = keypad.shortest_path(start, c)?;
            encoded_path += &keypad.encode_path_direction(shortest_path)?;
            encoded_path.push('A');
            start = c;
        }

        assert_eq!(encoded_path.len(), output_len);

        let path_a_positions: Vec<_> = encoded_path
            .chars()
            .enumerate()
            .filter(|(_, c)| *c == 'A')
            .map(|(i, _)| i)
            .collect();

        assert_eq!(path_a_positions.len(), output_a_count);
        assert_eq!(path_a_positions, output_a_positions);

        Ok(())
    }

    #[test]
    fn test_directional_encoding_long() -> miette::Result<()> {
        let mut keypad = directional_keypad::DirectionalKeypad::new();
        keypad.create_graph();

        let input_sequence = "v<<A>>^A<A>AvA<^AA>A<vAAA>^A"
            .chars()
            .collect::<Vec<char>>();

        let possible_output =
            "<vA<AA>>^AvAA<^A>A<v<A>>^AvA^A<vA>^A<v<A>^A>AAvA^A<v<A>A>^AAAvA<^A>A";
        let output_len = possible_output.len();
        let output_a_positions: Vec<_> = possible_output
            .chars()
            .enumerate()
            .filter(|(_, c)| *c == 'A')
            .map(|(i, _)| i)
            .collect();
        let output_a_count = output_a_positions.len();

        let mut encoded_path = String::new();
        let mut start = 'A';

        for c in input_sequence.clone() {
            let shortest_path = keypad.shortest_path(start, c)?;
            encoded_path += &keypad.encode_path_direction(shortest_path)?;
            encoded_path.push('A');
            start = c;
        }

        assert_eq!(encoded_path.len(), output_len);

        let path_a_positions: Vec<_> = encoded_path
            .chars()
            .enumerate()
            .filter(|(_, c)| *c == 'A')
            .map(|(i, _)| i)
            .collect();

        assert_eq!(path_a_positions.len(), output_a_count);
        assert_eq!(path_a_positions, output_a_positions);

        Ok(())
    }

    #[test]
    fn claude_testing_function() -> miette::Result<()> {
        let input = "029A";
        let expected_output =
            "<vA<AA>>^AvAA<^A>A<v<A>>^AvA^A<vA>^A<v<A>^A>AAvA^A<v<A>A>^AAAvA<^A>A";

        // Known good examples from the problem description
        let possible_level1_sequences = vec!["<A^A>^^AvvvA", "<A^A^>^AvvvA", "<A^A^^>AvvvA"];

        let mut numeric_keypad = numeric_keypad::NumericKeypad::new();
        numeric_keypad.create_graph();

        let mut directional_keypad = directional_keypad::DirectionalKeypad::new();
        directional_keypad.create_graph();

        // Level 1: Convert numeric input to directions
        println!(
            "\n=== Level 1: Converting {} to keypad directions ===",
            input
        );
        let mut encoded_sequence = String::new();
        let mut start = 'A';

        // Track each button press - always add 'A' after movement
        for (i, c) in input.chars().enumerate() {
            let shortest_path = numeric_keypad.shortest_path(start, c)?;
            let path_direction = numeric_keypad.encode_path_direction(shortest_path)?;
            println!("Step {}: Moving from {} to {}", i + 1, start, c);
            println!("  Path: {}", path_direction);
            encoded_sequence += &path_direction;
            encoded_sequence.push('A'); // Always append A for button press
            println!("  Sequence so far: {}", encoded_sequence);
            start = c;
        }
        println!("\nLevel 1 final sequence: {}", encoded_sequence);
        println!("Valid level 1 sequences:");
        for valid in &possible_level1_sequences {
            println!("  {}", valid);
        }

        // Level 2: Convert Level 1 sequence into directional keypad directions
        println!("\n=== Level 2: Converting Level 1 sequence to directional pad sequence ===");
        let mut encoded_sequence_2 = String::new();
        let mut start = 'A';

        // Track each direction press - always add 'A' after movement
        for (i, c) in encoded_sequence.chars().enumerate() {
            let shortest_path = directional_keypad.shortest_path(start, c)?;
            let path_direction = directional_keypad.encode_path_direction(shortest_path)?;
            println!("Step {}: Moving from {} to {}", i + 1, start, c);
            println!("  Path: {}", path_direction);
            encoded_sequence_2 += &path_direction;
            encoded_sequence_2.push('A'); // Always append A for button press
            println!("  Sequence so far: {}", encoded_sequence_2);
            start = c;
        }
        println!("\nLevel 2 final sequence: {}", encoded_sequence_2);

        // Level 3: Convert Level 2 sequence into final sequence
        println!("\n=== Level 3: Converting Level 2 sequence to final sequence ===");
        let mut encoded_sequence_3 = String::new();
        let mut start = 'A';

        // Track each direction press - always add 'A' after movement
        for (i, c) in encoded_sequence_2.chars().enumerate() {
            let shortest_path = directional_keypad.shortest_path(start, c)?;
            let path_direction = directional_keypad.encode_path_direction(shortest_path)?;
            println!("Step {}: Moving from {} to {}", i + 1, start, c);
            println!("  Path: {}", path_direction);
            encoded_sequence_3 += &path_direction;
            encoded_sequence_3.push('A'); // Always append A for button press
            println!("  Sequence so far: {}", encoded_sequence_3);
            start = c;
        }
        println!("\nLevel 3 final sequence: {}", encoded_sequence_3);

        // Compare with expected output
        println!("\n=== Final Analysis ===");
        println!("Length of sequence: {}", encoded_sequence_3.len());
        println!(
            "Number of A's: {}",
            encoded_sequence_3.chars().filter(|&c| c == 'A').count()
        );

        let segments: Vec<_> = encoded_sequence_3.split('A').collect();
        println!("Number of segments: {}", segments.len());
        println!("Segments:");
        for (i, segment) in segments.iter().enumerate() {
            println!("  {}: '{}'", i, segment);
        }

        // Validate Level 1
        let valid_l1_lengths: HashSet<_> = possible_level1_sequences
            .iter()
            .map(|s| s.chars().filter(|&c| c != 'A').count())
            .collect();
        let l1_movement_count = encoded_sequence.chars().filter(|&c| c != 'A').count();
        assert!(
            valid_l1_lengths.contains(&l1_movement_count),
            "Level 1 movement count {} not in valid lengths {:?}",
            l1_movement_count,
            valid_l1_lengths
        );

        // Validate final output
        let expected_length = expected_output.len();
        let actual_length = encoded_sequence_3.len();
        assert_eq!(
            expected_length, actual_length,
            "Final sequence has wrong number of button presses"
        );

        // Validate all segments contain only valid movements
        let valid_chars = ['<', '>', '^', 'v'];
        for segment in encoded_sequence_3.split('A') {
            assert!(
                segment.chars().all(|c| valid_chars.contains(&c)),
                "Invalid movement character in segment: {}",
                segment
            );
        }

        Ok(())
    }

    #[test]
    fn claude_testing_small() -> miette::Result<()> {
        let input = "029A";
        let expected_output =
            "<vA<AA>>^AvAA<^A>A<v<A>>^AvA^A<vA>^A<v<A>^A>AAvA^A<v<A>A>^AAAvA<^A>A";

        // Known good examples from the problem description
        let possible_level1_sequences = vec!["<A^A>^^AvvvA", "<A^A^>^AvvvA", "<A^A^^>AvvvA"];

        let mut numeric_keypad = numeric_keypad::NumericKeypad::new();
        numeric_keypad.create_graph();

        let mut directional_keypad = directional_keypad::DirectionalKeypad::new();
        directional_keypad.create_graph();

        println!("\n=== Converting {} to keypad directions ===", input);

        let mut input_iterator = input.chars();
        let mut level_1_encoded_sequence = String::new();

        let start = 'A';
        let end = input_iterator.next().unwrap(); // First Character "0"

        println!("\n=== Converting Digit {} ===", end);

        // Level 1 - Convert input to keypad directions
        println!("\n=== Level 1 - Numeric Keypad ===");
        let shortest_path = numeric_keypad.shortest_path(start, end)?;
        let path_direction = numeric_keypad.encode_path_direction(shortest_path)?;
        println!("Step 1: Moving from {} to {}", start, end);
        println!("  Path: {}", path_direction);
        level_1_encoded_sequence += &(path_direction + &"A"); // Always append A for button press
        println!("  Running Sequence: {}", level_1_encoded_sequence);

        // Level 2 - Convert input to keypad movement directions
        println!("\n=== Level 2 - D-pad #1 - Directional Commands to Numeric Keypad ===");
        let mut start1 = 'A';
        let mut start2 = 'A';
        let mut level_2_encoded_sequence = String::new();
        let mut level_3_encoded_sequence = String::new();

        for (idx1, c1) in level_1_encoded_sequence.chars().enumerate() {
            let shortest_path = directional_keypad.shortest_path(start1, c1)?;
            let path_direction = directional_keypad.encode_path_direction(shortest_path)?;
            println!("Step 2.{}: Moving from {} to {}", idx1 + 1, start, c1);
            println!("  Path: {}", path_direction);
            level_2_encoded_sequence += &(path_direction.clone() + &"A"); // Always append A for button press
            println!("  Running Sequence: {}", level_2_encoded_sequence);

            println!("\n=== Level 3 - D-pad #2 - Directional Commands to D-Pad #1 ===");
            for (idx2, c2) in path_direction.chars().enumerate() {
                let shortest_path = directional_keypad.shortest_path(start2, c2)?;
                let path_direction = directional_keypad.encode_path_direction(shortest_path)?;
                println!("Step 3.{}: Moving from {} to {}", idx2 + 1, start, c2);
                println!("  Path: {}", path_direction);
                level_3_encoded_sequence += &(path_direction + &"A"); // Always append A for button press
                println!("  Running Sequence: {}", level_3_encoded_sequence);
                start2 = c2;
            }
            start1 = c1;
        }

        Ok(())
    }
}
