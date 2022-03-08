use std::env;
use std::error::Error;
use std::fs;
use std::process::exit;

fn main() {
    if let Err(err) = day01() {
        println!("{}", err);
        exit(-1);
    }
}

fn day01() -> Result<(), Box<dyn Error>> {
    let input_str = match env::args().nth(1) {
        None => Err("Invalid number of arguments. Expected 2.")?,
        Some(path) => fs::read_to_string(path)?,
    };

    let modules: Vec<u32> = input_str
        .lines()
        .map(|l| l.trim().parse().unwrap())
        .collect();

    let res: u64 = modules
        .iter()
        .fold(0u64, |sum, val| sum + calc_fuel_for_module(*val) as u64);

    println!("answer: {}", res);
    Ok(())
}

fn calc_fuel_for_module(mass: u32) -> u32 {
    return mass / 3 - 2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_fuel() {
        assert_eq!(calc_fuel_for_module(12), 2);
        assert_eq!(calc_fuel_for_module(14), 2);
        assert_eq!(calc_fuel_for_module(1969), 654);
        assert_eq!(calc_fuel_for_module(100756), 33583);
    }
}
