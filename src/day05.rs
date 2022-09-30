use crate::computer::IntcodeComputer;
use anyhow::Result;
use std::fs;

pub fn solve(path: &str) -> Result<()> {
    let input = fs::read_to_string(path)?;

    let computer = IntcodeComputer::new(&input)?;

    unimplemented!()
}
