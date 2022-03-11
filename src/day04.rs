use std::error;
use std::fs;

pub fn solve(path: &str) -> Result<(), Box<dyn error::Error>> {
    let input = fs::read_to_string(path)?;

    let range = input
        .split('-')
        .map(|s| s.trim().parse::<u32>())
        .collect::<Result<Vec<_>, _>>()?;
    assert!(range.len() == 2);

    let mut answer1 = 0usize;
    let mut answer2 = 0usize;
    for i in range[0]..=range[1] {
        let (part1, part2) = is_password_good(i);
        // Just for fun. Should be done with if part1/part2.
        answer1 += part1 as usize;
        answer2 += part2 as usize;
    }

    println!("answer 1: {}", answer1);
    println!("answer 2: {}", answer2);
    Ok(())
}

fn is_password_good(pass: u32) -> (bool, bool) {
    let digits = to_digits(pass);

    let mut increases = true;
    let mut has_double = false;
    let mut has_adjacent = false;
    let (last_group_size, _) = digits.iter().fold((1, None), |(mut group, prev), digit| {
        if prev.is_some() {
            increases = increases && *digit >= prev.unwrap();
            if *digit == prev.unwrap() {
                has_adjacent = true;
                group += 1;
            } else {
                has_double = has_double || group == 2;
                group = 1;
            }
        }

        (group, Some(*digit))
    });

    // Finalize double-digit requirement with last group size
    has_double = has_double || last_group_size == 2;

    (increases && has_adjacent, increases && has_double)
}

fn to_digits(mut num: u32) -> Vec<u8> {
    let mut digits = Vec::new();

    while num > 0 {
        digits.push((num % 10) as u8);
        num = num / 10;
    }

    digits.reverse();
    digits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_digits() {
        assert_eq!(to_digits(223450u32), [2, 2, 3, 4, 5, 0]);
    }

    #[test]
    fn test() {
        let cases = [
            (111111u32, (true, false)),
            (123444, (true, false)),
            (111123, (true, false)),
            (135679, (false, false)),
            (223450, (false, false)),
            (123789, (false, false)),
            (676399, (false, false)),
        ];

        for (pass, res) in cases {
            assert_eq!(is_password_good(pass), res, "pass {}", pass);
        }
    }
}
