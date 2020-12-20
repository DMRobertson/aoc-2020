struct Notes {
    depart_after: u128,
    busses: Vec<u128>,
}

impl Notes {
    fn parse<'a>(mut input: impl Iterator<Item = &'a str>) -> Self {
        let depart_after = u128::from_str_radix(input.next().unwrap(), 10).unwrap();
        let busses = parse_ids(input.next().unwrap())
            .iter()
            .filter_map(|&x| x)
            .collect();
        Self {
            depart_after,
            busses,
        }
    }

    fn earliest_bus(&self) -> (u128, u128) {
        let get_time = |id| -> u128 {
            let div = self.depart_after / id;
            let rem = self.depart_after % id;
            if rem != 0 {
                (div + 1) * id
            } else {
                self.depart_after
            }
        };
        self.busses
            .iter()
            .map(|&id| (id, get_time(id)))
            .min_by_key(|(_, time)| *time)
            .unwrap()
    }
}

fn extended_gcd(a: i128, b: i128) -> (u128, i128, i128) {
    let (d, mut u, mut v) = extended_gcd_positive(a.abs() as u128, b.abs() as u128);
    if a < 0 {
        u *= -1;
    }
    if b < 0 {
        v *= -1;
    }
    (d, u, v)
}

fn extended_gcd_positive(a: u128, b: u128) -> (u128, i128, i128) {
    // From this exposition of the extended gcd algorithm http://anh.cs.luc.edu/331/notes/xgcd.pdf>.
    // Computes d = gcd(a, b) and returns a triple (d, x, y) where d = ax + by.
    let (mut last_x, mut x) = (1i128, 0i128);
    let (mut last_y, mut y) = (0i128, 1i128);

    let (mut last_remainder, mut remainder) = (a, b);

    while remainder > 0 {
        let quotient = last_remainder.div_euclid(remainder) as i128;
        let oldx = x;
        x = last_x - quotient * x;
        last_x = oldx;

        let oldy = y;
        y = last_y - quotient * y;
        last_y = oldy;

        let oldremainder = remainder;
        remainder = last_remainder.rem_euclid(remainder);
        last_remainder = oldremainder;
    }
    (last_remainder, last_x, last_y)
}

#[derive(Debug)]
struct IncreasingLinearSubsequence {
    // Set of integers of the form k*step + offset, for some integer k
    start: u128,
    step: u128,
}

impl IncreasingLinearSubsequence {
    fn new(start: i128, step: i128) -> Self {
        // normalise to a unique representation
        Self {
            start: start.rem_euclid(step) as u128,
            step: step.abs() as u128,
        }
    }

    fn intersect(&self, other: &Self) -> Self {
        // Solve self.step * x + self.offset == other.step * y + other.offset for (x, y)
        // Rearrange to get self.step*x - other.step * y == other.offset - self.offset
        let a = self.step as i128;
        let b = -(other.step as i128);
        let c = (other.start as i128) - (self.start as i128);
        let line = solve_linear_diophantine(a, b, c).unwrap();
        Self {
            start: self.start + self.step * line.x.start,
            step: num::integer::lcm(self.step, other.step),
        }
    }
}

struct BezoutSolution {
    x: IncreasingLinearSubsequence,
    _y: (i128, i128), // TODO: better to have decreasing linear sequence here
}

fn solve_linear_diophantine(a: i128, b: i128, c: i128) -> Option<BezoutSolution> {
    // The equation ax + by = c is a linear diophantine equation (where a, b, c are known
    // integers and x and y are integer variables).
    // This has a solution if and only if c is a multiple of the greatest common divisor of a and b.
    let (d, u, v) = extended_gcd(a, b);
    let d = d as i128;
    // (We divide through both sides by d.)
    // u and v chosen such that au + bv = d
    if c % d != 0 {
        return None;
    }
    // scale up to our exisiting equation
    let scale_factor = c / d;

    println!("{} {} {}", u, v, scale_factor);
    let (x, y) = (u * scale_factor, v * scale_factor);
    // Moreover, if (x, y) is a solution, then the other solutions have the form (x + kv, y −
    // ku), where k is an arbitrary integer, and u and v are the quotients of a and b
    // (respectively) by the greatest common divisor of a and b.
    Some(BezoutSolution {
        x: IncreasingLinearSubsequence::new(x, b / d),
        _y: (y, -a / d),
    })
}

impl IncreasingLinearSubsequence {}

fn parse_ids(input: &str) -> Vec<Option<u128>> {
    input
        .split(",")
        .map(|x| u128::from_str_radix(x, 10).ok())
        .collect()
}

fn string_of_departures(ids: &[Option<u128>]) -> u128 {
    let mut departures = IncreasingLinearSubsequence { start: 0, step: 1 };
    for (index, &entry) in ids.iter().enumerate() {
        match entry {
            None => {}
            Some(id) => {
                departures = departures.intersect(&IncreasingLinearSubsequence::new(
                    -(index as i128),
                    id as i128,
                ));
            }
        }
    }
    departures.start
}

fn main() {
    let input: Vec<_> = aoc_2020::problem_lines().collect();
    let notes = Notes::parse(input.iter().map(|s| s.as_str()));
    let (id, time) = notes.earliest_bus();
    println!("{}", id * (time - notes.depart_after));

    println!("{}", string_of_departures(&parse_ids(&input[1])));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gcd() {
        assert_eq!(extended_gcd(14, 33), (1, -7, 3));
        assert_eq!(extended_gcd(12, 32), (4, 3, -1));
    }

    #[test]
    fn example_1() {
        let input = "939\n7,13,x,x,59,x,31,19";
        let notes = Notes::parse(input.split("\n"));
        assert_eq!(notes.earliest_bus(), (59, 944));
    }

    #[test]
    fn example_2() {
        assert_eq!(
            string_of_departures(&parse_ids("7,13,x,x,59,x,31,19")),
            1068781
        );
        assert_eq!(string_of_departures(&parse_ids("17,x,13,19")), 3417);
        assert_eq!(
            string_of_departures(&parse_ids("1789,37,47,1889")),
            1202161486
        );
    }
}
