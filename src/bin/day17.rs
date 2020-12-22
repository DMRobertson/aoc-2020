#[macro_use]
extern crate itertools;

use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write};
use std::ops::Range;

fn main() {
    let lines: Vec<_> = aoc_2020::problem_lines().collect();
    let lines = lines.iter().map(|s| s.as_str());
    let orig_grid = new_grid(lines);
    let mut grid = orig_grid.clone();

    for _ in 0..6 {
        grid = iterate(&grid);
    }
    println!(
        "{}",
        grid.spaces
            .values()
            .filter(|&&x| x == Space::Active)
            .count()
    );

    let mut grid = Grid4::embed(&orig_grid);
    for _ in 0..6 {
        grid = iterate4(&grid);
    }
    println!(
        "{}",
        grid.spaces
            .values()
            .filter(|&&x| x == Space::Active)
            .count()
    );
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Space {
    Inactive,
    Active,
}

impl Display for Space {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Space::Inactive => '.',
            Space::Active => '#',
        })
    }
}

#[derive(Clone)]
struct Grid {
    // Use of a hashmap is icky, but it means I can ignore the problems with negative indices
    // without having to waste time transforming coordinages
    width: Range<isize>,
    height: Range<isize>,
    depth: Range<isize>,
    spaces: HashMap<(isize, isize, isize), Space>,
}

struct Grid4 {
    width: Range<isize>,
    height: Range<isize>,
    depth: Range<isize>,
    fourth_dimension: Range<isize>,
    spaces: HashMap<(isize, isize, isize, isize), Space>,
}

impl Grid4 {
    fn embed(src: &Grid) -> Self {
        let mut spaces = HashMap::new();
        for (&(x, y, z), &space) in src.spaces.iter() {
            spaces.insert((x, y, z, 0), space);
        }
        Self {
            width: src.width.clone(),
            height: src.height.clone(),
            depth: src.depth.clone(),
            fourth_dimension: 0..1,
            spaces,
        }
    }
}

impl Grid {
    fn dump(&self) {
        for z in self.depth.clone() {
            println!("z={}", z);
            for y in self.height.clone() {
                for x in self.width.clone() {
                    print!("{}", self.spaces[&(x, y, z)]);
                }
                println!();
            }
            println!();
        }
    }
}

fn expand(range: &Range<isize>) -> Range<isize> {
    (range.start - 1)..(range.end + 1)
}

fn iterate(orig: &Grid) -> Grid {
    let mut next = Grid {
        width: expand(&orig.width),
        height: expand(&orig.height),
        depth: expand(&orig.depth),
        spaces: HashMap::new(),
    };

    for index in iproduct!(next.width.clone(), next.height.clone(), next.depth.clone()) {
        let orig_space = orig.spaces.get(&index).cloned().unwrap_or(Space::Inactive);
        let active_neighbours = neighbours(index)
            .map(|v| orig.spaces.get(&v).cloned().unwrap_or(Space::Inactive))
            .filter(|&x| x == Space::Active)
            .count();
        let next_value = match (orig_space, active_neighbours) {
            (Space::Active, 2) | (Space::Active, 3) => Space::Active,
            (Space::Active, _) => Space::Inactive,
            (Space::Inactive, 3) => Space::Active,
            (Space::Inactive, _) => Space::Inactive,
        };
        next.spaces.insert(index, next_value);
    }
    next
}

fn neighbours(index: (isize, isize, isize)) -> impl Iterator<Item = (isize, isize, isize)> {
    let (x, y, z) = index;
    let xs = (x - 1)..=(x + 1);
    let ys = (y - 1)..=(y + 1);
    let zs = (z - 1)..=(z + 1);
    iproduct!(xs, ys, zs).filter(move |&(x1, y1, z1)| { (x1, y1, z1) } != (x, y, z))
}

fn iterate4(orig: &Grid4) -> Grid4 {
    let mut next = Grid4 {
        width: expand(&orig.width),
        height: expand(&orig.height),
        depth: expand(&orig.depth),
        fourth_dimension: expand(&orig.fourth_dimension),
        spaces: HashMap::new(),
    };

    for index in iproduct!(
        next.width.clone(),
        next.height.clone(),
        next.depth.clone(),
        next.fourth_dimension.clone()
    ) {
        let orig_space = orig.spaces.get(&index).cloned().unwrap_or(Space::Inactive);
        let active_neighbours = neighbours4(index)
            .map(|v| orig.spaces.get(&v).cloned().unwrap_or(Space::Inactive))
            .filter(|&x| x == Space::Active)
            .count();
        let next_value = match (orig_space, active_neighbours) {
            (Space::Active, 2) | (Space::Active, 3) => Space::Active,
            (Space::Active, _) => Space::Inactive,
            (Space::Inactive, 3) => Space::Active,
            (Space::Inactive, _) => Space::Inactive,
        };
        next.spaces.insert(index, next_value);
    }
    next
}

fn neighbours4(
    index: (isize, isize, isize, isize),
) -> impl Iterator<Item = (isize, isize, isize, isize)> {
    let (x, y, z, w) = index;
    let xs = (x - 1)..=(x + 1);
    let ys = (y - 1)..=(y + 1);
    let zs = (z - 1)..=(z + 1);
    let ws = (w - 1)..=(w + 1);
    iproduct!(xs, ys, zs, ws).filter(move |&v| v != index)
}

fn new_grid<'a>(lines: impl Iterator<Item = &'a str> + Clone) -> Grid {
    let width = lines.clone().next().unwrap().chars().count() as isize;
    let height = lines.clone().count() as isize;
    let mut grid = Grid {
        width: 0..width,
        height: 0..height,
        depth: 0..1,
        spaces: HashMap::new(),
    };
    for (y, line) in lines.enumerate() {
        for (x, c) in line.chars().enumerate() {
            let space = match c {
                '#' => Space::Active,
                '.' => Space::Inactive,
                _ => unimplemented!(),
            };
            grid.spaces.insert((x as isize, y as isize, 0), space);
        }
    }
    grid
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_1() {
        let input = "\
.#.
..#
###";
        let lines = input.split("\n");
        let original_grid = new_grid(lines);
        let mut grid = original_grid.clone();
        // println!("Before any cycles");
        // grid.dump();
        for _i in 0..6 {
            grid = iterate(&grid);
            // println!("After {} cycles:", _i + 1);
            // grid.dump();
        }
        assert_eq!(
            grid.spaces
                .values()
                .filter(|&&x| x == Space::Active)
                .count(),
            112
        );

        let mut grid = Grid4::embed(&original_grid);
        for _i in 0..6 {
            grid = iterate4(&grid);
        }
        assert_eq!(
            grid.spaces
                .values()
                .filter(|&&x| x == Space::Active)
                .count(),
            848
        );
    }
}
