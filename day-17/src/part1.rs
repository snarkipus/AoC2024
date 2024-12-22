pub fn process(input: &str) -> miette::Result<String> {
    let (_, (init_regs, instructions)) =
        parser::parse_input(input).map_err(|e| miette::miette!("Failed to parse input: {}", e))?;

    let mut processor = processor::Processor::new(init_regs, instructions);
    let output = processor.run()?;

    Ok(output
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(","))
}

pub mod processor {
    use miette::miette;
    use std::fmt;

    use super::parser::RegisterValues;
    pub type Program = Vec<usize>;

    #[derive(Debug, Clone, Copy)]
    pub struct Register(usize);

    impl Register {
        fn new(val: usize) -> Self {
            Self(val)
        }

        pub fn read(&self) -> usize {
            self.0
        }

        pub fn write(&mut self, val: usize) {
            self.0 = val;
        }
    }

    #[derive(Debug)]
    pub struct Processor {
        pub register_a: Register,
        pub register_b: Register,
        pub register_c: Register,
        pub program: Program,
        pub pc: usize,
        pub output: Vec<usize>,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Instruction(OpCode, Operand);

    #[derive(Debug, Clone, Copy)]
    pub struct OpCode(pub usize);

    #[derive(Debug, Clone, Copy)]
    pub struct Operand(pub usize);

    impl Processor {
        pub const MAX_STEPS: usize = 1000;
        pub const MAX_OUTPUT: usize = 100;

        // INIT
        pub fn new(init: RegisterValues, program: Program) -> Self {
            Self {
                register_a: Register::new(init[0]),
                register_b: Register::new(init[1]),
                register_c: Register::new(init[2]),
                program,
                pc: 0,
                output: Vec::new(),
            }
        }

        // FETCH
        fn fetch(&self) -> miette::Result<Instruction> {
            let slice = self
                .program
                .get(self.pc..self.pc + 2)
                .ok_or(miette!("Failed to fetch instruction"))?;
            Ok(Instruction(OpCode(slice[0]), Operand(slice[1])))
        }

        // DECODE & EXECUTE
        fn decode_execute(&mut self, instruction: Instruction) -> miette::Result<()> {
            match instruction {
                // 'adv' division: divide <a> by 2^<combo operand> and write the result to <a>
                Instruction(OpCode(0), Operand(operand)) => {
                    let num = self.register_a.read();
                    let operand = self.get_combo(operand);
                    let divisor = 2usize.pow(operand as u32);
                    self.register_a.write(num / divisor);
                    self.pc += 2;
                    Ok(())
                }
                // 'bxl' bitwise XOR: bitwise XOR <b> and <literal operand> and write the result to <b>
                Instruction(OpCode(1), Operand(operand)) => {
                    let val = self.register_b.read();
                    let result = val ^ operand;
                    self.register_b.write(result);
                    self.pc += 2;
                    Ok(())
                }
                // 'bst' modulo 8: <combo operand> modulo 8 and write the result to <b>
                Instruction(OpCode(2), Operand(operand)) => {
                    let val = self.get_combo(operand);
                    let result = val % 8;
                    self.register_b.write(result);
                    self.pc += 2;
                    Ok(())
                }
                // 'jnz' jump not zero: if <a> is not zero, jump to <literal operand> (no PC increment)
                Instruction(OpCode(3), Operand(operand)) => {
                    if self.register_a.read() != 0 {
                        self.pc = operand;
                    }
                    Ok(())
                }
                // 'bxc' bitwise XOR: bitwise XOR <b> and <c> and write the result to <b>
                Instruction(OpCode(4), Operand(_operand)) => {
                    let val_1 = self.register_b.read();
                    let val_2 = self.register_c.read();
                    let result = val_1 ^ val_2;
                    self.register_b.write(result);
                    self.pc += 2;
                    Ok(())
                }
                // 'out' output: output <combo operand> modulo 8 (csv appended to output)
                Instruction(OpCode(5), Operand(operand)) => {
                    let val = self.get_combo(operand);
                    let result = val % 8;
                    self.output.push(result);
                    self.pc += 2;
                    Ok(())
                }
                // 'bdv' division: divide <a> by 2^<combo operand> and write the result to <b>
                Instruction(OpCode(6), Operand(operand)) => {
                    let num = self.register_a.read();
                    let operand = self.get_combo(operand);
                    let divisor = 2usize.pow(operand as u32);
                    self.register_b.write(num / divisor);
                    self.pc += 2;
                    Ok(())
                }
                // 'cdv' division: divide <a> by 2^<combo operand> and write the result to <c>
                Instruction(OpCode(7), Operand(operand)) => {
                    let num = self.register_a.read();
                    let operand = self.get_combo(operand);
                    let divisor = 2usize.pow(operand as u32);
                    self.register_c.write(num / divisor);
                    self.pc += 2;
                    Ok(())
                }
                _ => panic!("Invalid instruction: {:?}", instruction),
            }
        }

        fn get_combo(&self, value: usize) -> usize {
            match value {
                0..=3 => value,
                4 => self.register_a.read(),
                5 => self.register_b.read(),
                6 => self.register_c.read(),
                _ => panic!("Invalid combo value: {}", value),
            }
        }

        pub fn run(&mut self) -> miette::Result<&Vec<usize>> {
            let mut steps = 0;

            while self.pc < self.program.len() - 1 {
                let instruction = self.fetch()?;
                self.decode_execute(instruction)?;

                if steps > Processor::MAX_STEPS {
                    break;
                }

                steps += 1;
            }

            Ok(&self.output)
        }
    }

    impl fmt::Display for Processor {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "PC: {:2} | Instruction: [{},{}] | A: {:10} | B: {:10} | C: {:10} | Out: {:?}",
                self.pc,
                self.program.get(self.pc).unwrap_or(&0),
                self.program.get(self.pc + 1).unwrap_or(&0),
                self.register_a.read(),
                self.register_b.read(),
                self.register_c.read(),
                self.output
            )
        }
    }
}

mod parser {
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{char, digit1, line_ending, newline},
        combinator::map_res,
        multi::separated_list1,
        sequence::{preceded, separated_pair},
        IResult,
    };

    use crate::part1::processor::Program;

    pub type RegisterValues = Vec<usize>;

    pub fn parse_input(input: &str) -> IResult<&str, (RegisterValues, Program)> {
        separated_pair(
            parse_registers,
            line_ending,
            preceded(line_ending, parse_program),
        )(input)
    }

    fn parse_registers(input: &str) -> IResult<&str, RegisterValues> {
        separated_list1(
            newline,
            preceded(
                alt((
                    tag("Register A: "),
                    tag("Register B: "),
                    tag("Register C: "),
                )),
                map_res(digit1, str::parse::<usize>),
            ),
        )(input)
    }

    fn parse_program(input: &str) -> IResult<&str, Program> {
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
        let (_, (regs, program)) = parser::parse_input(input).unwrap();
        assert_eq!((729, 0, 0), (regs[0], regs[1], regs[2]));
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
    fn test_instructions(#[case] test_case: TestCase) -> miette::Result<()> {
        let mut processor = processor::Processor::new(
            vec![test_case.reg_a, test_case.reg_b, test_case.reg_c],
            test_case.program,
        );

        let output = processor.run()?;

        if !test_case.expected_output.is_empty() {
            assert_eq!(&test_case.expected_output, output);
        }

        if let Some(expected_a) = test_case.expected_reg_a {
            assert_eq!(expected_a, processor.register_a.read());
        }

        if let Some(expected_b) = test_case.expected_reg_b {
            assert_eq!(expected_b, processor.register_b.read());
        }

        Ok(())
    }

    #[test]
    fn test_processor_display() {
        let processor = processor::Processor::new(vec![123, 456, 789], vec![0, 1, 2, 3]);
        let display = format!("{}", processor);
        assert!(display.contains("PC:  0"));
        assert!(display.contains("A:        123"));
        assert!(display.contains("B:        456"));
        assert!(display.contains("C:        789"));
        assert!(display.contains("Instruction: [0,1]"));
    }
}
