// input format: [opcode, operand, opcode, operand, ...]
//
// INSTRUCTION SET
// 0: 'adv' division: divide <a> by 2^<combo operand> and write the result to <a>
// 1: 'bxl' bitwise XOR: bitwise XOR <b> and <literal operand> and write the result to <b>
// 2: 'bst' modulo 8: <combo operand> modulo 8 and write the result to <b>
// 3: 'jnz' jump not zero: if <a> is not zero, jump to <literal operand> (no PC increment)
// 4: 'bxc' bitwise XOR: bitwise XOR <b> and <c> and write the result to <b>
// 5: 'out' output: output <combo operand> modulo 8 (csv appended to output)
// 6: 'bdv' division: divide <a> by 2^<combo operand> and write the result to <b>
// 7: 'cdv' division: divide <a> by 2^<combo operand> and write the result to <c>

pub type Registers = (usize, usize, usize);
pub type Instructions = Vec<usize>;

pub mod processor {
    use std::fmt;

    #[derive(Debug, Clone, Copy)]
    pub struct Register(usize);

    impl Register {
        fn new(val: usize) -> Self {
            Self(val)
        }

        pub fn get(&self) -> usize {
            self.0
        }

        fn div(&mut self, operand: usize) {
            self.0 /= 2usize.pow(operand as u32);
        }

        fn xor(&mut self, operand: usize) {
            self.0 ^= operand;
        }

        fn mod8(&mut self, operand: usize) {
            self.0 = operand % 8;
        }
    }

    #[derive(Debug)]
    pub struct Processor {
        pub reg_a: Register,
        pub reg_b: Register,
        pub reg_c: Register,
        pub instructions: Vec<usize>,
        pub pc: usize,
        pub output: Vec<usize>,
    }

    impl Processor {
        pub const MAX_STEPS: usize = 1000;
        pub const MAX_OUTPUT: usize = 100;

        pub fn new(reg_a: usize, reg_b: usize, reg_c: usize, instructions: Vec<usize>) -> Self {
            Self {
                reg_a: Register::new(reg_a),
                reg_b: Register::new(reg_b),
                reg_c: Register::new(reg_c),
                instructions,
                pc: 0,
                output: Vec::new(),
            }
        }

        fn get_combo(&self, value: usize) -> usize {
            match value {
                0..=3 => value,
                4 => self.reg_a.get(),
                5 => self.reg_b.get(),
                6 => self.reg_c.get(),
                _ => panic!("Invalid combo value: {}", value),
            }
        }

        pub fn run(&mut self) -> &Vec<usize> {
            let mut steps = 0;

            while self.pc < self.instructions.len() - 1 {
                let opcode = self.instructions[self.pc];
                let operand = self.instructions[self.pc + 1];
                println!("{}", self); 

                self.pc += 2;
                
                
                
                steps += 1;
                if steps > Processor::MAX_STEPS {
                    break;
                }

                match opcode {
                    0 => self.reg_a.div(self.get_combo(operand)),
                    1 => self.reg_b.xor(operand),
                    2 => self.reg_b.mod8(self.get_combo(operand)),
                    3 => {
                        if self.reg_a.get() != 0 {
                            println!("  Jump triggered! Jumping to {}", operand);
                            self.pc = operand;
                        }
                    }
                    4 => self.reg_b.xor(self.reg_c.get()),
                    5 => {
                        if self.output.len() < Processor::MAX_OUTPUT {
                            self.output.push(self.get_combo(operand) % 8);
                        } else {
                            break;
                        }
                    }
                    6 => self.reg_b.div(self.get_combo(operand)),
                    7 => self.reg_c.div(self.get_combo(operand)),
                    _ => (),
                }
            }

            &self.output
        }
    }

    impl fmt::Display for Processor {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "PC: {:2} | Instruction: [{},{}] | A: {:10} | B: {:10} | C: {:10} | Out: {:?}",
                self.pc,
                self.instructions.get(self.pc).unwrap_or(&0),
                self.instructions.get(self.pc + 1).unwrap_or(&0),
                self.reg_a.get(),
                self.reg_b.get(),
                self.reg_c.get(),
                self.output
            )
        }
    }
}
pub fn process(input: &str) -> miette::Result<String> {
    let (_, ((a, b, c), instructions)) =
        parser::parse_input(input).map_err(|e| miette::miette!("Failed to parse input: {}", e))?;

    let mut processor = processor::Processor::new(a, b, c, instructions);
    let output = processor.run();

    Ok(output
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(","))
}

mod parser {
    use nom::{
        bytes::complete::tag,
        character::complete::{char, digit1, line_ending},
        combinator::map_res,
        multi::separated_list1,
        sequence::{preceded, separated_pair},
        IResult,
    };

    use super::{Instructions, Registers};

    pub fn parse_input(input: &str) -> IResult<&str, (Registers, Instructions)> {
        separated_pair(
            parse_registers,
            line_ending,
            preceded(line_ending, parse_program),
        )(input)
    }

    fn parse_registers(input: &str) -> IResult<&str, Registers> {
        let (input, a) = preceded(tag("Register A: "), map_res(digit1, str::parse))(input)?;
        let (input, _) = line_ending(input)?;
        let (input, b) = preceded(tag("Register B: "), map_res(digit1, str::parse))(input)?;
        let (input, _) = line_ending(input)?;
        let (input, c) = preceded(tag("Register C: "), map_res(digit1, str::parse))(input)?;

        Ok((input, (a, b, c)))
    }

    fn parse_program(input: &str) -> IResult<&str, Instructions> {
        preceded(
            tag("Program: "),
            separated_list1(char(','), map_res(digit1, str::parse)),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "\
Register A: 729
Register B: 0
Register C: 0

Program: 0,1,5,4,3,0";
        assert_eq!("4,6,3,5,6,3,5,2,1,0", process(input)?);
        Ok(())
    }

    #[test]
    fn test_parser() -> miette::Result<()> {
        let input = "\
Register A: 729
Register B: 0
Register C: 0

Program: 0,1,5,4,3,0";
        let (_, ((a, b, c), program)) = parser::parse_input(input).unwrap();
        assert_eq!((729, 0, 0), (a, b, c));
        assert_eq!(vec![0, 1, 5, 4, 3, 0], program);
        Ok(())
    }

    use rstest::rstest;

    struct TestCase {
        reg_a: usize,
        reg_b: usize,
        reg_c: usize,
        program: Vec<usize>,
        expected_output: Vec<usize>,
        expected_reg_a: Option<usize>,
        expected_reg_b: Option<usize>,
    }

    #[rstest]
    #[test_log::test]
    #[case(TestCase {
        reg_a: 0,
        reg_b: 0,
        reg_c: 9,
        program: vec![2, 6],
        expected_output: vec![],
        expected_reg_a: None,
        expected_reg_b: Some(1),
    })]
    #[case(TestCase {
        reg_a: 10,
        reg_b: 0,
        reg_c: 0,
        program: vec![5, 0, 5, 1, 5, 4],
        expected_output: vec![0, 1, 2],
        expected_reg_a: None,
        expected_reg_b: None,
    })]
    #[case(TestCase {
        reg_a: 2024,
        reg_b: 0,
        reg_c: 0,
        program: vec![0, 1, 5, 4, 3, 0],
        expected_output: vec![4, 2, 5, 6, 7, 7, 7, 7, 3, 1, 0],
        expected_reg_a: Some(0),
        expected_reg_b: None,
    })]
    #[case(TestCase {
        reg_a: 0,
        reg_b: 29,
        reg_c: 0,
        program: vec![1, 7],
        expected_output: vec![],
        expected_reg_a: None,
        expected_reg_b: Some(26),
    })]
    #[case(TestCase {
        reg_a: 0,
        reg_b: 2024,
        reg_c: 43690,
        program: vec![4, 0],
        expected_output: vec![],
        expected_reg_a: None,
        expected_reg_b: Some(44354),
    })]
    #[case(TestCase {
        reg_a: 4,
        reg_b: 0,
        reg_c: 0,
        program: vec![5, 0, 3, 0, 5, 1],
        expected_output: vec![0; processor::Processor::MAX_OUTPUT],
        expected_reg_a: Some(4),
        expected_reg_b: None,
    })]
    fn test_instructions(#[case] test_case: TestCase) -> miette::Result<()> {
        let mut processor = processor::Processor::new(
            test_case.reg_a,
            test_case.reg_b,
            test_case.reg_c,
            test_case.program,
        );

        let output = processor.run();

        if !test_case.expected_output.is_empty() {
            assert_eq!(&test_case.expected_output, output);
        }

        if let Some(expected_a) = test_case.expected_reg_a {
            assert_eq!(expected_a, processor.reg_a.get());
        }

        if let Some(expected_b) = test_case.expected_reg_b {
            assert_eq!(expected_b, processor.reg_b.get());
        }

        Ok(())
    }

    #[test]
    fn test_processor_display() {
        let processor = processor::Processor::new(123, 456, 789, vec![0, 1, 2, 3]);
        let display = format!("{}", processor);
        assert!(display.contains("PC:  0"));
        assert!(display.contains("A:        123"));
        assert!(display.contains("B:        456"));
        assert!(display.contains("C:        789"));
        assert!(display.contains("Instruction: [0,1]"));
    }
}
