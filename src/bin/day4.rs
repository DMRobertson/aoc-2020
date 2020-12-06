#[macro_use]
extern crate scan_fmt;
#[macro_use]
extern crate lazy_static;

use regex::Regex;

#[derive(Debug)]
enum Height {
    Centimeters(usize),
    Inches(usize),
}

impl Height {
    fn parse(input: &str) -> Option<Self> {
        if let Ok((val, unit)) = scan_fmt!(input, "{d}{}", usize, String) {
            match unit.as_str() {
                "cm" => Some(Height::Centimeters(val)),
                "in" => Some(Height::Inches(val)),
                _ => None,
            }
        } else {
            None
        }
    }

    fn valid(&self) -> bool {
        match self {
            Height::Centimeters(v) => (150..=193).contains(v),
            Height::Inches(v) => (59..=76).contains(v),
        }
    }
}

#[derive(Debug)]
struct RGBColor {
    r: u8,
    g: u8,
    b: u8,
}

impl RGBColor {
    fn parse(input: &str) -> Option<Self> {
        scan_fmt!(input, "#{2x}{2x}{2x}", [hex u8], [hex u8], [hex u8])
            .ok()
            .map(|(r, g, b)| Self { r, g, b })
    }
}

#[derive(Debug)]
enum EyeColor {
    Amber,
    Blue,
    Brown,
    Grey,
    Green,
    Hazel,
    Other,
}

impl EyeColor {
    fn parse(input: &str) -> Option<Self> {
        match input {
            "amb" => Some(Self::Amber),
            "blu" => Some(Self::Blue),
            "brn" => Some(Self::Brown),
            "gry" => Some(Self::Grey),
            "grn" => Some(Self::Green),
            "hzl" => Some(Self::Hazel),
            "oth" => Some(Self::Other),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Passport {
    birth_year: usize,
    issue_year: usize,
    expiration_year: usize,
    height: Height,
    hair_color: RGBColor,
    eye_color: EyeColor,
    passport_id: usize,
    country_id: Option<String>,
}

#[derive(Debug)]
struct ProtoPassport {
    birth_year: Option<usize>,
    issue_year: Option<usize>,
    expiration_year: Option<usize>,
    height: Option<Height>,
    hair_color: Option<RGBColor>,
    eye_color: Option<EyeColor>,
    passport_id: Option<usize>,
    country_id: Option<String>,
}

lazy_static! {
    static ref PASSPORT_ID_PATTERN: Regex = Regex::new(r"^\d{9}$").unwrap();
}

impl ProtoPassport {
    fn new() -> Self {
        Self {
            birth_year: None,
            issue_year: None,
            expiration_year: None,
            height: None,
            hair_color: None,
            eye_color: None,
            passport_id: None,
            country_id: None,
        }
    }

    fn feed(&mut self, key: &str, value: &str) {
        let parse = str::parse::<usize>;
        match key {
            "byr" => self.birth_year = parse(value).ok().filter(|y| (1920..=2002).contains(y)),
            "iyr" => self.issue_year = parse(value).ok().filter(|y| (2010..=2020).contains(y)),
            "eyr" => self.expiration_year = parse(value).ok().filter(|y| (2020..=2030).contains(y)),
            "hgt" => self.height = Height::parse(value).filter(Height::valid),
            "hcl" => self.hair_color = RGBColor::parse(value),
            "ecl" => self.eye_color = EyeColor::parse(value),
            "pid" => {
                self.passport_id = {
                    if PASSPORT_ID_PATTERN.is_match(value) {
                        Some(parse(value).unwrap())
                    } else {
                        None
                    }
                }
            }
            "cid" => self.country_id = Some(value.to_owned()),
            _ => (),
        }
    }

    fn complete(self) -> Option<Passport> {
        match self {
            ProtoPassport {
                birth_year: Some(birth_year),
                issue_year: Some(issue_year),
                expiration_year: Some(expiration_year),
                height: Some(height),
                hair_color: Some(hair_color),
                eye_color: Some(eye_color),
                passport_id: Some(passport_id),
                country_id,
            } => Some(Passport {
                birth_year,
                issue_year,
                expiration_year,
                height,
                hair_color,
                eye_color,
                passport_id,
                country_id,
            }),
            _ => {
                println!("INVALID: {:?}", self);
                None
            }
        }
    }
}

impl Passport {
    fn parse(input: &str) -> Option<Self> {
        let mut passport = ProtoPassport::new();
        input
            .split_whitespace()
            .map(|s| {
                let mut parts = s.split(":").take(2);
                (parts.next().unwrap(), parts.next().unwrap())
            })
            .for_each(|(key, value)| passport.feed(key, value));

        passport.complete()
    }
}

fn main() {
    let mut lines = aoc_2020::problem_lines();
    let mut passports = lines.next().unwrap();
    for line in lines {
        passports.push_str("\n");
        passports.push_str(&line);
    }
    let parsed = parse_passport_listings(&passports);
    for passport in parsed.iter().flatten() {
        println!("{:?}", passport);
    }
    let valid = count_valid_passports(&parsed);
    println!("{}", valid);
}

fn parse_passport_listings(input: &str) -> Vec<Option<Passport>> {
    input
        .split("\n\n")
        .inspect(|x| println!("{:?}", x))
        .map(Passport::parse)
        .collect()
}

fn count_valid_passports(passports: &[Option<Passport>]) -> usize {
    passports.iter().filter(|x| x.is_some()).count()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parser() {
        let feed = |key, value| {
            let mut x = ProtoPassport::new();
            x.feed(key, value);
            x
        };
        assert_eq!(feed("byr", "1919").birth_year.is_some(), false);
        assert_eq!(feed("byr", "1920").birth_year.is_some(), true);
        assert_eq!(feed("byr", "2002").birth_year.is_some(), true);
        assert_eq!(feed("byr", "2003").birth_year.is_some(), false);

        assert_eq!(feed("iyr", "2009").issue_year.is_some(), false);
        assert_eq!(feed("iyr", "2010").issue_year.is_some(), true);
        assert_eq!(feed("iyr", "2020").issue_year.is_some(), true);
        assert_eq!(feed("iyr", "2021").issue_year.is_some(), false);

        assert_eq!(feed("eyr", "2019").expiration_year.is_some(), false);
        assert_eq!(feed("eyr", "2020").expiration_year.is_some(), true);
        assert_eq!(feed("eyr", "2030").expiration_year.is_some(), true);
        assert_eq!(feed("eyr", "2031").expiration_year.is_some(), false);

        assert_eq!(feed("hgt", "149cm").height.is_some(), false);
        assert_eq!(feed("hgt", "150cm").height.is_some(), true);
        assert_eq!(feed("hgt", "193cm").height.is_some(), true);
        assert_eq!(feed("hgt", "194cm").height.is_some(), false);
        assert_eq!(feed("hgt", "149").height.is_some(), false);
        assert_eq!(feed("hgt", "150").height.is_some(), false);
        assert_eq!(feed("hgt", "193").height.is_some(), false);
        assert_eq!(feed("hgt", "194").height.is_some(), false);
        assert_eq!(feed("hgt", "58in").height.is_some(), false);
        assert_eq!(feed("hgt", "59in").height.is_some(), true);
        assert_eq!(feed("hgt", "76in").height.is_some(), true);
        assert_eq!(feed("hgt", "77in").height.is_some(), false);
        assert_eq!(feed("hgt", "58").height.is_some(), false);
        assert_eq!(feed("hgt", "59").height.is_some(), false);
        assert_eq!(feed("hgt", "76").height.is_some(), false);
        assert_eq!(feed("hgt", "77").height.is_some(), false);

        assert_eq!(feed("hcl", "#123abc").height.is_some(), false);
        assert_eq!(feed("hcl", "#123abz").height.is_some(), false);
        assert_eq!(feed("hcl", "#123abc").height.is_some(), false);
        assert_eq!(feed("hcl", "123abz").height.is_some(), false);

        assert_eq!(feed("ecl", "amb").eye_color.is_some(), true);
        assert_eq!(feed("ecl", "blu").eye_color.is_some(), true);
        assert_eq!(feed("ecl", "brn").eye_color.is_some(), true);
        assert_eq!(feed("ecl", "gry").eye_color.is_some(), true);
        assert_eq!(feed("ecl", "grn").eye_color.is_some(), true);
        assert_eq!(feed("ecl", "hzl").eye_color.is_some(), true);
        assert_eq!(feed("ecl", "oth").eye_color.is_some(), true);
        assert_eq!(feed("ecl", "zzz").eye_color.is_some(), false);

        assert_eq!(feed("pid", "000000001").passport_id.is_some(), true);
        assert_eq!(feed("pid", "0123456789").passport_id.is_some(), false);
    }

    #[test]
    fn examples() {
        const FIRST_EXAMPLES: &'static str = "\
ecl:gry pid:860033327 eyr:2020 hcl:#fffffd
byr:1937 iyr:2017 cid:147 hgt:183cm

iyr:2013 ecl:amb cid:350 eyr:2023 pid:028048884
hcl:#cfa07d byr:1929

hcl:#ae17e1 iyr:2013
eyr:2024
ecl:brn pid:760753108 byr:1931
hgt:179cm

hcl:#cfa07d eyr:2025 pid:166559648
iyr:2011 ecl:brn hgt:59in";

        let valid: Vec<_> = parse_passport_listings(FIRST_EXAMPLES)
            .iter()
            .map(|x| x.is_some())
            .collect();
        assert_eq!(valid, vec![true, false, true, false]);
    }

    #[test]
    fn invalid() {
        const INVALID_PASSPORTS: &'static str = "\
eyr:1972 cid:100
hcl:#18171d ecl:amb hgt:170 pid:186cm iyr:2018 byr:1926

iyr:2019
hcl:#602927 eyr:1967 hgt:170cm
ecl:grn pid:012533040 byr:1946

hcl:dab227 iyr:2012
ecl:brn hgt:182cm pid:021572410 eyr:2020 byr:1992 cid:277

hgt:59cm ecl:zzz
eyr:2038 hcl:74454a iyr:2023
pid:3556412378 byr:2007";
        let invalid: Vec<_> = parse_passport_listings(INVALID_PASSPORTS)
            .iter()
            .map(|x| x.is_some())
            .collect();
        assert_eq!(invalid, vec![false; 4]);
    }

    #[test]
    fn valid() {
        const VALID_PASSPORTS: &'static str = "\
pid:087499704 hgt:74in ecl:grn iyr:2012 eyr:2030 byr:1980
hcl:#623a2f

eyr:2029 ecl:blu cid:129 byr:1989
iyr:2014 pid:896056539 hcl:#a97842 hgt:165cm

hcl:#888785
hgt:164cm byr:2001 iyr:2015 cid:88
pid:545766238 ecl:hzl
eyr:2022

iyr:2010 hgt:158cm hcl:#b6652a ecl:blu byr:1944 eyr:2021 pid:093154719";
        let valid: Vec<_> = parse_passport_listings(VALID_PASSPORTS)
            .iter()
            .map(|x| x.is_some())
            .collect();
        assert_eq!(valid, vec![true; 4]);
    }
}
