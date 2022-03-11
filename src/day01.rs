use anyhow::Result;
use std::fs;

pub fn solve(path: &String) -> Result<()> {
    let input_str = fs::read_to_string(path)?;

    let modules: Vec<u32> = input_str
        .lines()
        .map(|l| l.trim().parse().unwrap())
        .collect();

    let ans1: u32 = modules.iter().fold(0u32, |sum, val| sum + calc_fuel(*val));

    let ans2: u32 = modules
        .iter()
        .fold(0u32, |sum, val| sum + calc_fuel_total(*val));

    println!("answer 1: {}", ans1);
    println!("answer 2: {}", ans2);
    Ok(())
}

fn calc_fuel(mass: u32) -> u32 {
    return (mass / 3).saturating_sub(2);
}

fn calc_fuel_total(module_mass: u32) -> u32 {
    let mut ans = 0;
    let mut mass = module_mass;
    while mass > 0 {
        mass = calc_fuel(mass);
        ans += mass;
    }

    return ans + mass;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_fuel() {
        assert_eq!(calc_fuel(12), 2);
        assert_eq!(calc_fuel(14), 2);
        assert_eq!(calc_fuel(1969), 654);
        assert_eq!(calc_fuel(100756), 33583);
    }

    #[test]
    fn test_calc_total() {
        assert_eq!(calc_fuel_total(14), 2);
        assert_eq!(calc_fuel_total(1969), 966);
        assert_eq!(calc_fuel_total(100756), 50346);
    }
}
