use std::error;
use std::fs;

pub(crate) fn day04(path: &str) -> Result<(), Box<dyn error::Error>> {
    let input = fs::read_to_string(path)?;

    let range = input
        .split('-')
        .map(|s| s.trim().parse::<u32>())
        .collect::<Result<Vec<_>, _>>()?;
    assert!(range.len() == 2);

    let mut count = 0usize;
    for i in range[0]..=range[1] {
        if is_password_good(i) {
            count += 1;
        }
    }

    println!("answer 1: {}", count);
    Ok(())
}

fn is_password_good(pass: u32) -> bool {
    let digits = to_digits(pass);

    let (_, increases, has_double) = digits.iter().fold(
        (None, true, false),
        |(prev, mut increases, mut double), digit| {
            if prev.is_some() {
                increases = increases && *digit >= prev.unwrap();
                double = double || *digit == prev.unwrap();
            }

            (Some(*digit), increases, double)
        },
    );

    increases && has_double
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
            (111111u32, true),
            (111123, true),
            (135679, false),
            (223450, false),
            (123789, false),
            (676399, false),
        ];

        for (pass, res) in cases {
            assert_eq!(is_password_good(pass), res, "pass {}", pass);
        }
    }
}
