use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn process(input: &str) -> miette::Result<String> {
    let (_, (_, instructions)) =
        parser::parse_input(input).map_err(|e| miette::miette!("Failed to parse input: {}", e))?;

    // Configure thread pool to match CPU
    ThreadPoolBuilder::new()
        .num_threads(16) // Match 5800X3D's thread count
        .build_global()
        .unwrap();

    // Optimize chunk size for the 96MB L3 cache
    const CHUNK_SIZE: usize = 50_000; // Should fit nicely in L3 cache
                                      // const MAX_RANGE: usize = 500_000_000;
    const MAX_RANGE: usize = 281_474_977_000_000;
    const REPORT_INTERVAL: usize = 1_000_000; // Report progress every million numbers

    let found = Arc::new(AtomicBool::new(false));

    // for range_start in (281_474_976_000_000..MAX_RANGE).step_by(REPORT_INTERVAL) {
    for range_start in (0..MAX_RANGE).step_by(REPORT_INTERVAL) {
        let range_end = (range_start + REPORT_INTERVAL).min(MAX_RANGE);
        println!("Searching range {} to {}", range_start, range_end);

        // Pre-generate our chunks for this range
        let chunks: Vec<_> = (range_start..range_end)
            .step_by(CHUNK_SIZE)
            // Create aligned ranges for better cache utilization
            .map(|start| {
                let end = (start + CHUNK_SIZE - 1).min(range_end);
                start..=end
            })
            .collect();

        if let Some(solution) = chunks.into_par_iter().find_map_first(|range| {
            range.into_par_iter().find_first(|&init| {
                if found.load(Ordering::Relaxed) {
                    return false;
                }

                let mut processor =
                    processor::Processor::new(vec![init, 0, 0], instructions.clone());
                match processor.run() {
                    Ok(output) => {
                        let matches = output == &instructions;
                        if matches {
                            found.store(true, Ordering::Relaxed);
                            println!("Found potential solution: {}", init);
                        }
                        matches
                    }
                    Err(_) => false,
                }
            })
        }) {
            let mut processor =
                processor::Processor::new(vec![solution, 0, 0], instructions.clone());
            let output = processor.run()?.clone();

            println!("Confirmed solution at reg_a_init = {}", solution);

            return Ok(output
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(","));
        }
    }

    Err(miette::miette!("No solution found within the search range"))
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
                    // Check for overflow before performing 2^operand
                    if operand >= u32::BITS as usize {
                        return Err(miette!(
                            "Power overflow: 2^{} exceeds maximum value",
                            operand
                        ));
                    }
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
                    } else {
                        self.pc += 2;
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
                    // Check for overflow before performing 2^operand
                    if operand >= u32::BITS as usize {
                        return Err(miette!(
                            "Power overflow: 2^{} exceeds maximum value",
                            operand
                        ));
                    }
                    self.register_b.write(num / divisor);
                    self.pc += 2;
                    Ok(())
                }
                // 'cdv' division: divide <a> by 2^<combo operand> and write the result to <c>
                Instruction(OpCode(7), Operand(operand)) => {
                    let num = self.register_a.read();
                    let operand = self.get_combo(operand);
                    let divisor = 2usize.pow(operand as u32);
                    // Check for overflow before performing 2^operand
                    if operand >= u32::BITS as usize {
                        return Err(miette!(
                            "Power overflow: 2^{} exceeds maximum value",
                            operand
                        ));
                    }
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
            let max_output: usize = self.program.len();

            let mut steps = 0;

            while self.pc < self.program.len() - 1 {
                let instruction = self.fetch()?;

                // if self.register_a.read() == 117440 {
                //     println!("{}", &self);
                // }

                self.decode_execute(instruction)?;

                if steps > Processor::MAX_STEPS {
                    break;
                }

                if self.output.len() >= max_output {
                    break;
                }

                if self.output != &self.program[0..self.output.len()] {
                    break;
                }

                if self.register_a.read() == 0 {
                    break;
                }

                // if self.register_b.read() != 0 || self.register_c.read() != 0 {
                //     break;
                // }

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

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "\
Register A: 2024
Register B: 0
Register C: 0

Program: 0,3,5,4,3,0";
        assert_eq!("0,3,5,4,3,0", process(input)?);
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
