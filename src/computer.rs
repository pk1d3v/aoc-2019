use anyhow::{anyhow, bail, Context, Result};

#[derive(Debug, Default)]
pub struct IntcodeComputer {
    program: Vec<i32>,
    ram: Ram,
    // Instruction pointer
    ip: usize,
    halted: bool,
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
        })
    }

    pub fn ram(&self) -> &Ram {
        &self.ram
    }

    pub fn reset(&mut self) {
        self.ram = Ram(self.program.clone());
        self.ip = 0;
        self.halted = false;
    }

    // Starts program execution in computer
    pub fn run(&mut self, noun: u32, verb: u32) -> Result<()> {
        // Additional input
        self.ram.write(1, noun as i32)?;
        self.ram.write(2, verb as i32)?;

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
        };
        self.ip += instruction.size();
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Ram(Vec<i32>);

impl Ram {
    pub fn read(&self, address: usize) -> Result<&i32> {
        self.0.get(address).ok_or(anyhow!(
            "Read RAM failure: out of bounds access, address {}",
            address
        ))
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

enum Instruction {
    Add(i32, i32, usize),      // operand 1, operand 2, destination
    Multiply(i32, i32, usize), // operand 1, operand 2, destination
    Halt,
}

impl Instruction {
    fn decode(mem: &Ram, address: usize) -> Result<Instruction> {
        let &opcode = mem.read(address)?;
        match opcode {
            // Opcode 1 adds together numbers read from two positions and stores the result in a third position.
            // Opcode 2 works exactly like opcode 1, except it multiplies the two inputs instead of adding them.
            1 | 2 => {
                let &operand1 = mem.read(address + 1)?;
                let &operand2 = mem.read(address + 2)?;
                let &dest = mem.read(address + 3)?;

                // Read actual values for operation
                let &operand1 = mem.read(operand1 as usize)?;
                let &operand2 = mem.read(operand2 as usize)?;

                if opcode == 1 {
                    Ok(Self::Add(operand1, operand2, dest as usize))
                } else {
                    Ok(Self::Multiply(operand1, operand2, dest as usize))
                }
            }
            // 99 means that the program is finished and should immediately halt.
            99 => Ok(Self::Halt),
            _ => bail!("Invalid opcode encountered: {} at {}", opcode, address),
        }
    }

    // Returns size of instruction
    fn size(&self) -> usize {
        match *self {
            Self::Add(_, _, _) | Self::Multiply(_, _, _) => 4,
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
}
