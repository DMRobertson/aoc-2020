use std::ops::RangeInclusive;

#[macro_use]
extern crate scan_fmt;

trait PasswordPolicy: Sized {
    fn parse(desc: &str) -> Option<(Self, String)>;
    fn permits(&self, password: &str) -> bool;
}

struct OldPasswordPolicy {
    required: char,
    occurrences: RangeInclusive<usize>,
}

impl PasswordPolicy for OldPasswordPolicy {
    fn parse(desc: &str) -> Option<(Self, String)> {
        scan_fmt!(desc, "{d}-{d} {}: {}", usize, usize, char, String)
            .ok()
            .map(|(min, max, required, password)| {
                (
                    Self {
                        required,
                        occurrences: RangeInclusive::new(min, max),
                    },
                    password,
                )
            })
    }

    fn permits(&self, password: &str) -> bool {
        let occurrences = password.chars().filter(|&c| c == self.required).count();
        self.occurrences.contains(&occurrences)
    }
}

struct NewPasswordPolicy {
    required: char,
    positions: [usize; 2],
}

impl PasswordPolicy for NewPasswordPolicy {
    fn parse(desc: &str) -> Option<(Self, String)> {
        scan_fmt!(desc, "{d}-{d} {}: {}", usize, usize, char, String)
            .ok()
            .map(|(first, second, required, password)| {
                (
                    Self {
                        required,
                        positions: [first, second],
                    },
                    password,
                )
            })
    }

    fn permits(&self, password: &str) -> bool {
        let count: usize = password
            .chars()
            .enumerate()
            .filter(|&(index, c)| self.positions.contains(&(index + 1)) && c == self.required)
            .count();
        count == 1
        // should also check the passowrd is sufficiently long but screw it
    }
}

fn valid_password<Policy>(line: &str) -> Option<bool>
where
    Policy: PasswordPolicy,
{
    Policy::parse(line).map(|(policy, password)| policy.permits(&password))
}

fn main() {
    let passwords: Vec<_> = aoc_2020::problem_lines().collect();
    println!(
        "{} valid passwords",
        count_valid_passwords::<OldPasswordPolicy>(&passwords)
    );
    println!(
        "{} valid passwords",
        count_valid_passwords::<NewPasswordPolicy>(&passwords)
    );
}

fn count_valid_passwords<Policy>(lines: &Vec<String>) -> usize
where
    Policy: PasswordPolicy,
{
    lines
        .iter()
        .map(|s| valid_password::<Policy>(s))
        .filter(|x| x.unwrap())
        .count()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn old_policy() {
        assert_eq!(
            valid_password::<OldPasswordPolicy>("1-3 a: abcde"),
            Some(true)
        );
        assert_eq!(
            valid_password::<OldPasswordPolicy>("1-3 b: cdefg"),
            Some(false)
        );
        assert_eq!(
            valid_password::<OldPasswordPolicy>("2-9 c: ccccccccc"),
            Some(true)
        );
    }

    #[test]
    fn new_policy() {
        assert_eq!(
            valid_password::<NewPasswordPolicy>("1-3 a: abcde"),
            Some(true)
        );
        assert_eq!(
            valid_password::<NewPasswordPolicy>("1-3 b: cdefg"),
            Some(false)
        );
        assert_eq!(
            valid_password::<NewPasswordPolicy>("2-9 c: ccccccccc"),
            Some(false)
        );
    }
}
