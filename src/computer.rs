use anyhow::{anyhow, bail, Context, Result};

#[derive(Debug, Default)]
pub struct IntcodeComputer {
    program: Vec<i32>,
    ram: Ram,
    // Instruction pointer
    ip: usize,
    halted: bool,
    // I/O
    input: Vec<i32>,
    output: Vec<i32>,
}

impl IntcodeComputer {
    pub fn new(program: &str) -> Result<IntcodeComputer> {
        let program = program
            .trim()
            .split(',')
            .map(|s| s.trim().parse::<i32>())
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to read intcode program from file")?;

        Ok(IntcodeComputer {
            program: program.clone(),
            ram: Ram(program),
            ip: 0,
            halted: false,
            input: Vec::new(),
            output: Vec::new(),
        })
    }

    pub fn ram(&self) -> &Ram {
        &self.ram
    }

    pub fn reset(&mut self) {
        self.ram = Ram(self.program.clone());
        self.ip = 0;
        self.halted = false;
        self.input.clear();
        self.output.clear();
    }

    // Starts program execution in computer
    pub fn run_nv(&mut self, noun: u32, verb: u32) -> Result<()> {
        // Additional input
        self.ram.write(1, noun as i32)?;
        self.ram.write(2, verb as i32)?;

        self.execute()?;
        Ok(())
    }

    pub fn run(&mut self, input: Option<Vec<i32>>) -> Result<()> {
        if let Some(input) = input {
            // Reverse the input so it can be easily popped
            self.input = input;
        }

        self.execute()?;
        Ok(())
    }

    // Internal instruction execution loop
    fn execute(&mut self) -> Result<()> {
        while !self.halted {
            self.process_instruction()?;
        }
        Ok(())
    }

    fn process_instruction(&mut self) -> Result<()> {
        let instruction = Instruction::decode(&self.ram, self.ip)?;
        match instruction {
            Instruction::Add(a, b, dst) => self.ram.write(dst, a + b)?,
            Instruction::Multiply(a, b, dst) => self.ram.write(dst, a * b)?,
            Instruction::Halt => self.halted = true,
            Instruction::Input(dst) => self
                .ram
                .write(dst, self.input.pop().ok_or(anyhow!("Input is exhausted"))?)?,
            Instruction::Output(src) => self.output.push(*self.ram.read(src)?),
        };
        self.ip += instruction.size();
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Ram(Vec<i32>);

impl Ram {
    pub fn read(&self, address: usize) -> Result<i32> {
        self.0
            .get(address)
            .ok_or(anyhow!(
                "Read RAM failure: out of bounds access, address {}",
                address
            ))
            .copied()
    }

    fn write(&mut self, address: usize, value: i32) -> Result<()> {
        let v = self.0.get_mut(address).ok_or(anyhow!(
            "Write RAM failure: out of bounds access, address {}",
            address
        ))?;
        *v = value;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ParameterMode {
    Address,
    Value,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Opcode {
    op: u8,
    modes: [ParameterMode; 3],
}

impl TryFrom<i32> for Opcode {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let mut params_part = value / 100;
        let mut modes = [ParameterMode::Address; 3];
        for m in modes.iter_mut() {
            *m = match params_part % 10 {
                0 => ParameterMode::Address,
                1 => ParameterMode::Value,
                _ => bail!(
                    "Invalid parameter mode {} in opcode {}",
                    params_part % 10,
                    value
                ),
            };
            params_part /= 10;
        }
        if params_part > 0 {
            bail!("Invalid opcode {}: too many parameter modes", value);
        }

        let op = (value % 100) as u8;
        Ok(Opcode { op, modes })
    }
}

enum Instruction {
    Add(i32, i32, usize),      // operand 1, operand 2, destination
    Multiply(i32, i32, usize), // operand 1, operand 2, destination
    Input(usize),              // destination
    Output(usize),             // source
    Halt,
}

impl Instruction {
    fn decode(mem: &Ram, address: usize) -> Result<Instruction> {
        let opcode: Opcode = mem.read(address).try_into()?;
        match opcode.op {
            // Opcode 1 adds together numbers read from two positions and stores the result in a third position.
            // Opcode 2 works exactly like opcode 1, except it multiplies the two inputs instead of adding them.
            1 | 2 => {
                let operand1 = mem.read(address + 1)?;
                let operand2 = mem.read(address + 2)?;
                let dest = mem.read(address + 3)?;

                // Read actual values for operation
                let &operand1 = mem.read(operand1 as usize)?;
                let &operand2 = mem.read(operand2 as usize)?;

                if opcode.op == 1 {
                    Ok(Self::Add(operand1, operand2, dest as usize))
                } else {
                    Ok(Self::Multiply(operand1, operand2, dest as usize))
                }
            }
            // Opcode 3 takes a single integer as input and saves it to the position given by its only parameter.
            3 => Ok(Self::Input(*mem.read(address + 1)? as usize)),
            // Opcode 4 outputs the value of its only parameter.
            4 => Ok(Self::Output(*mem.read(address + 1)? as usize)),
            // 99 means that the program is finished and should immediately halt.
            99 => Ok(Self::Halt),
            _ => bail!("Invalid opcode encountered: {} at {}", opcode, address),
        }
    }

    // Returns size of instruction
    fn size(&self) -> usize {
        match *self {
            Self::Add(_, _, _) | Self::Multiply(_, _, _) => 4,
            Self::Input(_) | Self::Output(_) => 2,
            Self::Halt => 0, // Don't move further when halt is reached
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cases = [
            ("1,0,0,0,99", vec![1, 0, 0, 0, 99]),
            (
                "1, 1, 1, 4, 99 ,5 ,6 , 0 , 99, -1 ",
                vec![1, 1, 1, 4, 99, 5, 6, 0, 99, -1],
            ),
        ];

        for (input, expected) in cases {
            let c = IntcodeComputer::new(&input);
            assert!(c.is_ok());
            assert_eq!(c.unwrap().ram.0, expected);
        }
    }

    #[test]
    fn test_reset() {
        let expected = [2, 0, 0, 0, 99];
        let c = IntcodeComputer::new("1, 0, 0, 0, 99");
        assert!(c.is_ok());
        let mut c = c.unwrap();
        for _ in 0..2 {
            assert!(c.execute().is_ok());
            assert!(c.halted);
            assert_eq!(c.ram.0, expected);
            c.reset();
            assert!(!c.halted);
            assert_eq!(c.ip, 0);
            assert_eq!(c.ram.0, c.program);
            assert_ne!(c.ram.0, expected);
        }
    }

    #[test]
    fn test_sample_programs() {
        let cases = [
            ("1, 0, 0, 0, 99", vec![2, 0, 0, 0, 99]),
            ("2, 4, 4, 5, 99, 0", vec![2, 4, 4, 5, 99, 9801]),
            (
                "1, 1, 1, 4, 99, 5, 6, 0, 99",
                vec![30, 1, 1, 4, 2, 5, 6, 0, 99],
            ),
        ];

        for (input, expected) in cases {
            let c = IntcodeComputer::new(input);
            assert!(c.is_ok());
            let mut c = c.unwrap();
            assert!(c.execute().is_ok());
            assert!(c.halted);
            assert_eq!(c.ram.0, expected);
        }
    }

    #[test]
    fn test_io() {
        let c = IntcodeComputer::new("3,0,4,0,99");
        assert!(c.is_ok());
        let mut c = c.unwrap();
        assert!(c.run(Some(vec![123])).is_ok());
        assert!(c.halted);
        assert_eq!(c.output.len(), 1);
        assert_eq!(*c.output.first().unwrap(), 123);
    }

    #[test]
    fn test_opcode_halt() {
        let mut c = IntcodeComputer::new("99").unwrap();
        assert!(c.execute().is_ok());
        assert!(c.halted);
        assert_eq!(c.ip, 0);
    }

    #[test]
    fn test_opcode_invalid() {
        let mut c = IntcodeComputer {
            ram: Ram(vec![0]),
            ..Default::default()
        };
        assert!(c.execute().is_err());
    }

    #[test]
    fn test_opcode_add() {
        let mut c = IntcodeComputer {
            ram: Ram(vec![1, 5, 6, 4, 0, 1, 98]),
            ..Default::default()
        };
        assert!(c.process_instruction().is_ok());
        assert_eq!(c.ip, 4);
        assert_eq!(c.ram.0[4], 99);
    }

    #[test]
    fn test_opcode_multiply() {
        let mut c = IntcodeComputer {
            ram: Ram(vec![2, 4, 5, 0, 10, 11]),
            ..Default::default()
        };
        assert!(c.process_instruction().is_ok());
        assert_eq!(c.ip, 4);
        assert_eq!(c.ram.0[0], 10 * 11);
    }

    #[test]
    fn test_opcode_invalid_access() {
        let mut c = IntcodeComputer {
            ram: Ram(vec![1, 100]),
            ..Default::default()
        };
        assert!(c.process_instruction().is_err());
    }

    #[test]
    fn test_opcode_parse() {
        let cases = [
            (
                1002i32,
                Opcode {
                    op: 2,
                    modes: [
                        ParameterMode::Address,
                        ParameterMode::Value,
                        ParameterMode::Address,
                    ],
                },
            ),
            (
                11101i32,
                Opcode {
                    op: 1,
                    modes: [ParameterMode::Value; 3],
                },
            ),
            (
                99i32,
                Opcode {
                    op: 99,
                    modes: [ParameterMode::Address; 3],
                },
            ),
        ];

        for (val, expected) in cases {
            let opcode: Result<Opcode> = val.try_into();
            assert!(opcode.is_ok());
            assert_eq!(opcode.unwrap(), expected, "value: {}", val);
        }
    }
}
