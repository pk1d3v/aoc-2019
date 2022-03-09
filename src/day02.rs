use std::error::Error;
use std::fs;

pub(crate) fn day02(path: &str) -> Result<(), Box<dyn Error>> {
    let program = fs::read_to_string(path)?;
    let mut ram = load_program(&program)?;

    // Additional preparations
    ram[1] = 12;
    ram[2] = 2;

    execute_program(&mut ram)?;

    println!("answer 1: {}", ram[0]);
    Ok(())
}

// Loads program from string
fn load_program(program: &str) -> Result<Vec<u32>, Box<dyn Error>> {
    let data = program
        .split(',')
        .map(|s| Ok(s.trim().parse::<u32>()?))
        .collect();
    data
}

fn execute_program(ram: &mut Vec<u32>) -> Result<(), Box<dyn Error>> {
    // Instruction pointer
    let mut ip = 0usize;

    loop {
        let halt = execute_opcode(ram, &mut ip)?;
        if halt {
            break;
        }
    }

    Ok(())
}

fn execute_opcode(ram: &mut Vec<u32>, ip: &mut usize) -> Result<bool, Box<dyn Error>> {
    let opcode = ram[*ip];
    *ip += 1;
    match opcode {
        // Opcode 1 adds together numbers read from two positions and stores the result in a third position.
        // Opcode 2 works exactly like opcode 1, except it multiplies the two inputs instead of adding them.
        op @ (1 | 2) => {
            let operand1 = *ram.get(*ip).ok_or("Operand1 is out of bounds")?;
            let operand2 = *ram.get(*ip + 1).ok_or("Operand2 is out of bounds")?;
            let dest = *ram.get(*ip + 2).ok_or("Operand3 is out of bounds")?;

            // Move instruction pointer
            *ip += 3;

            // Read actual values for operation
            let operand1 = *ram
                .get(operand1 as usize)
                .ok_or(format!("Address {} is out of bounds", operand1))?;
            let operand2 = *ram
                .get(operand2 as usize)
                .ok_or(format!("Address {} is out of bounds", operand2))?;
            let dest = ram
                .get_mut(dest as usize)
                .ok_or(format!("Address {} is out of bounds", dest))?;

            if op == 1 {
                *dest = operand1 + operand2;
            } else {
                *dest = operand1 * operand2;
            }
        }
        // 99 means that the program is finished and should immediately halt.
        99 => {
            return Ok(true);
        }
        _ => {
            return Err(format!(
                "Invalid opcode encountered: {} at {}",
                opcode,
                *ip - 1
            ))?
        }
    };
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_program() {
        assert_eq!(
            load_program("1,0,0,0,99").unwrap(),
            [1u32, 0u32, 0u32, 0u32, 99u32]
        );
        assert_eq!(
            load_program("1, 1, 1, 4, 99 ,5 ,6 ,0 , 99 ").unwrap(),
            [1u32, 1u32, 1u32, 4u32, 99u32, 5u32, 6u32, 0u32, 99u32]
        );
    }

    #[test]
    fn test_execute_program() {
        let mut prog: Vec<u32> = vec![1, 0, 0, 0, 99];

        assert_eq!(execute_program(&mut prog).is_ok(), true);
        assert_eq!(prog, [2, 0, 0, 0, 99]);

        let mut prog: Vec<u32> = vec![2, 4, 4, 5, 99, 0];
        assert_eq!(execute_program(&mut prog).is_ok(), true);
        assert_eq!(prog, [2, 4, 4, 5, 99, 9801]);

        let mut prog: Vec<u32> = vec![1, 1, 1, 4, 99, 5, 6, 0, 99];
        assert_eq!(execute_program(&mut prog).is_ok(), true);
        assert_eq!(prog, [30, 1, 1, 4, 2, 5, 6, 0, 99]);
    }

    #[test]
    fn test_opcode_halt() {
        let mut ip: usize = 0;
        let result = execute_opcode(&mut vec![99u32], &mut ip);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), true);
        assert_eq!(ip, 1);
    }

    #[test]
    fn test_opcode_invalid() {
        let mut ip: usize = 0;
        let result = execute_opcode(&mut vec![0u32], &mut ip);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_opcode_with_out_of_bounds_access() {
        let mut prog: Vec<u32> = vec![1, 100];
        let mut ip: usize = 0;
        let result = execute_opcode(&mut prog, &mut ip);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_opcode_add() {
        let mut prog: Vec<u32> = vec![1, 5, 6, 4, 0, 1, 98];
        let mut ip: usize = 0;
        let result = execute_opcode(&mut prog, &mut ip);
        assert_eq!(result.is_ok(), true, "Result is: {:?}", result);
        assert_eq!(result.unwrap(), false);
        assert_eq!(ip, 4);
        assert_eq!(prog[4], 99);
    }

    #[test]
    fn test_opcode_multiply() {
        let mut prog: Vec<u32> = vec![1, 4, 5, 0, 2, 2];
        let mut ip: usize = 0;
        let result = execute_opcode(&mut prog, &mut ip);
        assert_eq!(result.is_ok(), true, "Result is: {:?}", result);
        assert_eq!(result.unwrap(), false);
        assert_eq!(ip, 4);
        assert_eq!(prog[0], 4);
    }
}
