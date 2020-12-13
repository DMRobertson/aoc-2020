#[derive(Clone, Eq, PartialEq)]
enum Tile {
    Floor,
    EmptySeat,
    OccupiedSeat,
}

impl Tile {
    fn parse(c: char) -> Self {
        match c {
            '.' => Self::Floor,
            'L' => Self::EmptySeat,
            '#' => Self::OccupiedSeat,
            _ => unimplemented!(),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
struct Layout {
    width: usize,
    height: usize,
    grid: Vec<Tile>,
}

impl Layout {
    fn parse<'a>(mut input: impl Iterator<Item = &'a str>) -> Self {
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

    fn iterate(&self) -> Self {
        let mut next = Layout {
            width: self.width,
            height: self.height,
            grid: self.grid.to_vec(),
        };
        for x in 0..self.width {
            for y in 0..self.height {
                let index = y * self.width + x;
                let occupied = self.occupied_adjacent_seats(x as isize, y as isize);
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
                        if occupied >= 4 {
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

    fn occupied_adjacent_seats(&self, x: isize, y: isize) -> u8 {
        // Not very happy with the casts going on here. Maybe there's a better way.
        let occupied_at = |(dx, dy): &(isize, isize)| -> u8 {
            let x = x + dx;
            let y = y + dy;
            let occupied =
                if 0 <= x && x < self.width as isize && 0 <= y && y < self.height as isize {
                    self.grid[self.width * (y as usize) + x as usize] == Tile::OccupiedSeat
                } else {
                    false
                };
            occupied as u8
        };
        let offsets: [(isize, isize); 8] = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];
        offsets.iter().map(occupied_at).sum()
    }

    fn count(&self, t: Tile) -> usize {
        self.grid.iter().filter(|tile| **tile == t).count()
    }
}

fn find_steady_state(layout: &Layout) -> Layout {
    let mut layout = layout.clone();
    loop {
        let next = layout.iterate();
        if layout == next {
            return next;
        }
        layout = next;
    }
}

fn main() {
    let input: Vec<_> = aoc_2020::problem_lines().collect();
    let start = Layout::parse(input.iter().map(String::as_str));
    let steady = find_steady_state(&start);
    println!("{}", steady.count(Tile::OccupiedSeat));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example() {
        let layout = "\
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
        let start = Layout::parse(layout.split("\n"));
        let steady = find_steady_state(&start);
        assert_eq!(steady.count(Tile::OccupiedSeat), 37);
    }
}
