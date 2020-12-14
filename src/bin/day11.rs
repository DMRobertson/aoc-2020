use std::fmt;
use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone, Eq, PartialEq)]
enum Tile {
    Floor,
    EmptySeat,
    OccupiedSeat,
}

impl Tile {
    fn char(&self) -> char {
        match self {
            Tile::Floor => '.',
            Tile::EmptySeat => 'L',
            Tile::OccupiedSeat => '#',
        }
    }
    fn parse(c: char) -> Self {
        match c {
            '.' => Self::Floor,
            'L' => Self::EmptySeat,
            '#' => Self::OccupiedSeat,
            _ => unimplemented!(),
        }
    }
}

const DIRECTION_VECTORS: [(isize, isize); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

#[derive(PartialEq, Eq, Clone)]
struct Layout {
    width: usize,
    height: usize,
    grid: Vec<Tile>,
}

impl Layout {
    fn parse(input: &str) -> Self {
        Self::from_rows(input.split("\n"))
    }

    fn from_rows<'a>(mut input: impl Iterator<Item = &'a str>) -> Self {
        let first = input.next().unwrap();
        let width = first.len();
        let mut grid: Vec<_> = first.chars().map(Tile::parse).collect();
        let mut height = 1;
        for line in input {
            grid.extend(line.chars().map(Tile::parse));
            height += 1;
        }
        Self {
            width,
            height,
            grid,
        }
    }

    fn get(&self, x: isize, y: isize) -> Option<&Tile> {
        if 0 <= x && x < self.width as isize && 0 <= y && y < self.height as isize {
            Some(&self.grid[self.width * (y as usize) + x as usize])
        } else {
            None
        }
    }

    fn iterate(
        &self,
        count_occupation: fn(&Layout, isize, isize) -> u8,
        occupation_leave_threshold: u8,
    ) -> Self {
        let mut next = Layout {
            width: self.width,
            height: self.height,
            grid: self.grid.to_vec(),
        };
        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                let occupied = count_occupation(self, x as isize, y as isize);
                let next_tile = match self.grid[index] {
                    Tile::Floor => Tile::Floor,
                    Tile::EmptySeat => {
                        if occupied == 0 {
                            Tile::OccupiedSeat
                        } else {
                            Tile::EmptySeat
                        }
                    }
                    Tile::OccupiedSeat => {
                        if occupied >= occupation_leave_threshold {
                            Tile::EmptySeat
                        } else {
                            Tile::OccupiedSeat
                        }
                    }
                };
                next.grid[index] = next_tile;
            }
        }
        next
    }

    fn adjacent_occupied_seats(&self, x: isize, y: isize) -> u8 {
        // Not very happy with the casts going on here. Maybe there's a better way.
        let occupied_at = |(dx, dy): &(isize, isize)| -> bool {
            self.get(x + dx, y + dy) == Some(&Tile::OccupiedSeat)
        };
        DIRECTION_VECTORS
            .iter()
            .map(occupied_at)
            .map(|b| b as u8)
            .sum()
    }

    fn visible_occupied_seats(&self, x: isize, y: isize) -> u8 {
        let occupied_visible_in = |(dx, dy): &(isize, isize)| -> bool {
            let (mut x, mut y) = (x, y);
            loop {
                x += dx;
                y += dy;
                match self.get(x, y) {
                    None => return false,
                    Some(&Tile::OccupiedSeat) => return true,
                    Some(&Tile::EmptySeat) => return false,
                    Some(&Tile::Floor) => continue,
                }
            }
        };
        DIRECTION_VECTORS
            .iter()
            .map(occupied_visible_in)
            .map(|b| b as u8)
            .sum()
    }

    fn count(&self, t: Tile) -> usize {
        self.grid.iter().filter(|tile| **tile == t).count()
    }
}

impl Debug for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                output.push(self.grid[y * self.width + x].char());
            }
            if y != self.height - 1 {
                output.push('\n');
            }
        }
        f.write_str(&output)
    }
}

fn find_steady_state(
    layout: &Layout,
    count_occupation: fn(&Layout, isize, isize) -> u8,
    occupation_leave_threshold: u8,
) -> Layout {
    let mut layout = layout.clone();
    loop {
        let next = layout.iterate(count_occupation, occupation_leave_threshold);
        if layout == next {
            return next;
        }
        layout = next;
    }
}

fn main() {
    let input: Vec<_> = aoc_2020::problem_lines().collect();
    let start = Layout::from_rows(input.iter().map(String::as_str));
    let steady = find_steady_state(&start, Layout::adjacent_occupied_seats, 4);
    println!("{}", steady.count(Tile::OccupiedSeat));

    let steady2 = find_steady_state(&start, Layout::visible_occupied_seats, 5);
    println!("{}", steady2.count(Tile::OccupiedSeat));
}

#[cfg(test)]
mod test {
    use super::*;
    const STARTING_LAYOUT: &str = "\
L.LL.LL.LL
LLLLLLL.LL
L.L.L..L..
LLLL.LL.LL
L.LL.LL.LL
L.LLLLL.LL
..L.L.....
LLLLLLLLLL
L.LLLLLL.L
L.LLLLL.LL";

    #[test]
    fn example_1() {
        let start = Layout::parse(STARTING_LAYOUT);
        let steady = find_steady_state(&start, Layout::adjacent_occupied_seats, 4);
        assert_eq!(steady.count(Tile::OccupiedSeat), 37);
    }

    #[test]
    fn example_2() {
        let start = Layout::parse(STARTING_LAYOUT);

        let step1 = Layout::parse(
            "\
#.##.##.##
#######.##
#.#.#..#..
####.##.##
#.##.##.##
#.#####.##
..#.#.....
##########
#.######.#
#.#####.##",
        );
        assert_eq!(step1, start.iterate(Layout::visible_occupied_seats, 5));
        let step2 = Layout::parse(
            "\
#.LL.LL.L#
#LLLLLL.LL
L.L.L..L..
LLLL.LL.LL
L.LL.LL.LL
L.LLLLL.LL
..L.L.....
LLLLLLLLL#
#.LLLLLL.L
#.LLLLL.L#",
        );
        assert_eq!(step2, step1.iterate(Layout::visible_occupied_seats, 5));
        let step3 = Layout::parse(
            "\
#.L#.##.L#
#L#####.LL
L.#.#..#..
##L#.##.##
#.##.#L.##
#.#####.#L
..#.#.....
LLL####LL#
#.L#####.L
#.L####.L#",
        );
        assert_eq!(step3, step2.iterate(Layout::visible_occupied_seats, 5));

        let steady = find_steady_state(&start, Layout::visible_occupied_seats, 5);
        assert_eq!(steady.count(Tile::OccupiedSeat), 26);
    }
}
