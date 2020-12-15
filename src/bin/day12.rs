use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;

struct Action {
    kind: ActionKind,
    value: usize,
}

enum ActionKind {
    North,
    East,
    South,
    West,
    RotateLeft,
    RotateRight,
    Forward,
}

impl From<Direction> for ActionKind {
    fn from(d: Direction) -> Self {
        match d {
            Direction::North => Self::North,
            Direction::East => Self::East,
            Direction::South => Self::South,
            Direction::West => Self::West,
        }
    }
}

impl Action {
    fn parse(input: &str) -> Self {
        let (first, rest) = input.split_at(1);
        use ActionKind::*;
        let kind = match first {
            "F" => Forward,
            "L" => RotateLeft,
            "R" => RotateRight,
            "N" => North,
            "E" => East,
            "S" => South,
            "W" => West,
            _ => unimplemented!(),
        };
        let value = usize::from_str_radix(rest, 10).unwrap();
        Self { kind, value }
    }
}

#[derive(IntoPrimitive, TryFromPrimitive, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i8)]
enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

fn rotate_left(d: Direction, angle: usize) -> Direction {
    let mut value: i8 = d.into();
    value -= (angle / 90) as i8;
    Direction::try_from(value.rem_euclid(4)).unwrap()
}

fn rotate_right(d: Direction, angle: usize) -> Direction {
    let mut value: i8 = d.into();
    value += (angle / 90) as i8;
    Direction::try_from(value.rem_euclid(4)).unwrap()
}

struct Ship {
    dir: Direction,
    position: (isize, isize),
}

impl Ship {
    fn new() -> Self {
        Self {
            dir: Direction::East,
            position: (0, 0),
        }
    }

    fn act(&mut self, action: &Action) {
        use ActionKind::*;
        match action.kind {
            North => self.position.1 += action.value as isize,
            South => self.position.1 -= action.value as isize,
            East => self.position.0 += action.value as isize,
            West => self.position.0 -= action.value as isize,
            RotateLeft => self.dir = rotate_left(self.dir, action.value),
            RotateRight => self.dir = rotate_right(self.dir, action.value),
            Forward => self.act(&Action {
                kind: self.dir.into(),
                value: action.value,
            }),
        }
    }
}

struct ShipWithWaypoint {
    position: (isize, isize),
    waypoint: (isize, isize),
}

impl ShipWithWaypoint {
    fn new() -> Self {
        Self {
            position: (0, 0),
            waypoint: (10, 1),
        }
    }

    fn act(&mut self, action: &Action) {
        use ActionKind::*;
        match action.kind {
            North => self.waypoint.1 += action.value as isize,
            South => self.waypoint.1 -= action.value as isize,
            East => self.waypoint.0 += action.value as isize,
            West => self.waypoint.0 -= action.value as isize,
            RotateLeft => self.rotate_waypoint_anticlockwise(action.value),
            RotateRight => self.rotate_waypoint_anticlockwise(360 - action.value),
            Forward => {
                self.position.0 += self.waypoint.0 * action.value as isize;
                self.position.1 += self.waypoint.1 * action.value as isize;
            }
        }
    }

    fn rotate_waypoint_anticlockwise(&mut self, angle: usize) {
        let steps = (angle / 90).rem_euclid(4);
        match steps {
            0 => (),
            1 => {
                let temp = -self.waypoint.1;
                self.waypoint.1 = self.waypoint.0;
                self.waypoint.0 = temp;
            }
            2 => {
                self.waypoint.0 *= -1;
                self.waypoint.1 *= -1
            }
            3 => {
                let temp = self.waypoint.1;
                self.waypoint.1 = -self.waypoint.0;
                self.waypoint.0 = temp;
            }
            _ => unreachable!(),
        }
    }
}

fn main() {
    let mut ship1 = Ship::new();
    let mut ship2 = ShipWithWaypoint::new();

    for line in aoc_2020::problem_lines() {
        let a = Action::parse(&line);
        ship1.act(&a);
        ship2.act(&a);
    }
    println!("{}", manhattan_distance((0, 0), ship1.position));
    println!("{}", manhattan_distance((0, 0), ship2.position));
}

fn manhattan_distance(src: (isize, isize), dest: (isize, isize)) -> usize {
    let d = (dest.0 - src.0).abs() + (dest.1 - src.1).abs();
    d as usize
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_1() {
        let mut ship = Ship::new();
        ship.act(&Action::parse("F10"));
        assert_eq!(ship.position, (10, 0));
        ship.act(&Action::parse("N3"));
        assert_eq!(ship.position, (10, 3));
        ship.act(&Action::parse("F7"));
        assert_eq!(ship.position, (17, 3));
        ship.act(&Action::parse("R90"));
        assert_eq!(ship.position, (17, 3));
        assert_eq!(ship.dir, Direction::South);
        ship.act(&Action::parse("F11"));
        assert_eq!(ship.position, (17, -8));
        assert_eq!(manhattan_distance((0, 0), ship.position), 25);
    }

    #[test]
    fn example_2() {
        let mut ship = ShipWithWaypoint::new();
        ship.act(&Action::parse("F10"));
        assert_eq!(ship.position, (100, 10));
        assert_eq!(ship.waypoint, (10, 1));
        ship.act(&Action::parse("N3"));
        assert_eq!(ship.position, (100, 10));
        assert_eq!(ship.waypoint, (10, 4));
        ship.act(&Action::parse("F7"));
        assert_eq!(ship.position, (170, 38));
        assert_eq!(ship.waypoint, (10, 4));
        ship.act(&Action::parse("R90"));
        assert_eq!(ship.position, (170, 38));
        assert_eq!(ship.waypoint, (4, -10));
        ship.act(&Action::parse("F11"));
        assert_eq!(ship.position, (214, -72));
        assert_eq!(ship.waypoint, (4, -10));
        assert_eq!(manhattan_distance((0, 0), ship.position), 286);
    }
}
