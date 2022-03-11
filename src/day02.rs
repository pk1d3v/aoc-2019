use crate::computer::IntcodeComputer;
use std::error::Error;
use std::fs;

pub fn solve(path: &str) -> Result<(), Box<dyn Error>> {
    let program = fs::read_to_string(path)?;
    let mut computer = IntcodeComputer::new(&program)?;

    computer.run(12, 2)?;
    let &ans = computer.ram().read(0)?;
    println!("answer 1: {}", ans);

    let target = 19690720u32;

    for noun in 1..100 {
        for verb in 1..100 {
            computer.reset();
            computer.run(noun, verb)?;
            let &val = computer.ram().read(0)?;
            if val as u32 == target {
                println!("answer 2: {}", 100 * noun + verb);
                return Ok(());
            }
        }
    }

    Err("Answer not found!")?
}
