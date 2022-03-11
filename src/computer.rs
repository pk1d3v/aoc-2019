use std::error;

#[derive(Debug, Default)]
pub struct IntcodeComputer {
    program: Vec<i32>,
    ram: Ram,
    // Instruction pointer
    ip: usize,
    halted: bool,
}

impl IntcodeComputer {
    pub fn new(program: &str) -> Result<IntcodeComputer, Box<dyn error::Error>> {
        let program = program
            .trim()
            .split(',')
            .map(|s| s.trim().parse::<i32>())
            .collect::<Result<Vec<_>, _>>()?;

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
    pub fn run(&mut self, noun: u32, verb: u32) -> Result<(), Box<dyn error::Error>> {
        // Additional input
        self.ram.write(1, noun as i32)?;
        self.ram.write(2, verb as i32)?;

        self.execute()?;
        Ok(())
    }

    // Internal instruction execution loop
    fn execute(&mut self) -> Result<(), Box<dyn error::Error>> {
        while !self.halted {
            self.process_instruction()?;
        }
        Ok(())
    }

    fn process_instruction(&mut self) -> Result<(), Box<dyn error::Error>> {
        let &opcode = self.ram.read(self.ip)?;
        self.ip += 1;
        match opcode {
            // Opcode 1 adds together numbers read from two positions and stores the result in a third position.
            // Opcode 2 works exactly like opcode 1, except it multiplies the two inputs instead of adding them.
            op @ (1 | 2) => {
                let &operand1 = self.ram.read(self.ip)?;
                let &operand2 = self.ram.read(self.ip + 1)?;
                let &dest = self.ram.read(self.ip + 2)?;

                // Move instruction pointer
                self.ip += 3;

                // Read actual values for operation
                let &operand1 = self.ram.read(operand1 as usize)?;
                let &operand2 = self.ram.read(operand2 as usize)?;

                if op == 1 {
                    self.ram.write(dest as usize, operand1 + operand2)?;
                } else {
                    self.ram.write(dest as usize, operand1 * operand2)?;
                }
            }
            // 99 means that the program is finished and should immediately halt.
            99 => self.halted = true,
            _ => {
                return Err(format!(
                    "Invalid opcode encountered: {} at {}",
                    opcode,
                    self.ip - 1
                ))?
            }
        };
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Ram(Vec<i32>);

impl Ram {
    pub fn read(&self, address: usize) -> Result<&i32, String> {
        self.0.get(address).ok_or(format!(
            "Read RAM failure: out of bounds access, address {}",
            address
        ))
    }

    fn write(&mut self, address: usize, value: i32) -> Result<(), String> {
        let v = self.0.get_mut(address).ok_or(format!(
            "Write RAM failure: out of bounds access, address {}",
            address
        ))?;
        *v = value;
        Ok(())
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
        assert_eq!(c.ip, 1);
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
