#[macro_use]
extern crate itertools;

use itertools::join;
use ndarray::Array3;
use std::fmt::{Display, Formatter, Write};

fn main() {
    let input = aoc_2020::problem_lines();
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

type Grid = Array3<Space>;
fn iterate(orig: &Grid) -> Grid {
    let orig_shape = orig.shape();
    let next_shape = (orig_shape[0] + 2, orig_shape[1] + 2, orig_shape[2] + 2);

    let mut next = Array3::from_elem(next_shape, Space::Inactive);
    // let embedded = next.slice(embedded_shape);

    for (index, next_value) in next.indexed_iter_mut() {
        // What was this space in the previous iteration?
        let orig_index = previous_index(index);
        let space = orig_index
            .map(|i| orig.get(i).cloned().unwrap_or(Space::Inactive))
            .unwrap_or(Space::Inactive);

        // What were its neighbours at the time?
        let active_neighbours = neighbours(index)
            .map(|v| orig.get(v).cloned().unwrap_or(Space::Inactive))
            .filter(|&x| x == Space::Active)
            .count();
        *next_value = match (space, active_neighbours) {
            (Space::Active, 2) | (Space::Active, 3) => Space::Active,
            (Space::Active, _) => Space::Inactive,
            (Space::Inactive, 3) => Space::Active,
            (Space::Inactive, _) => Space::Inactive,
        };
    }
    next
}

fn previous_index(index: (usize, usize, usize)) -> Option<(usize, usize, usize)> {
    let (x, y, z) = index;
    let x1 = x.checked_sub(1)?;
    let y1 = y.checked_sub(1)?;
    let z1 = z.checked_sub(1)?;
    Some((x1, y1, z1))
}

fn neighbours(index: (usize, usize, usize)) -> impl Iterator<Item = (usize, usize, usize)> {
    let (x, y, z) = (index.0 as isize, index.1 as isize, index.2 as isize);
    let xs = (x - 1)..=(x + 1);
    let ys = (y - 1)..=(y + 1);
    let zs = (z - 1)..=(z + 1);
    iproduct!(xs, ys, zs)
        .filter(move |&(x1, y1, z1)| { (x1, y1, z1) } != (x, y, z) && x1 >= 0 && y1 >= 0 && z1 >= 0)
        .map(|(x1, y1, z1)| (x1 as usize, y1 as usize, z1 as usize))
}

#[cfg(test)]
mod test {
    use ndarray::Axis;

    use super::*;

    #[test]
    fn example_1() {
        let input = "\
.#.
..#
###";
        let width = input.split("\n").next().unwrap().chars().count();
        let height = input.split("\n").count();
        let elements: Vec<_> = input
            .split("\n")
            .flat_map(|line| line.chars())
            .map(|c| match c {
                '.' => Space::Inactive,
                '#' => Space::Active,
                _ => unimplemented!(),
            })
            .collect();
        let mut spaces = Array3::from_shape_vec((width, height, 1), elements).unwrap();

        for i in 0..6 {
            println!("After {} cycles", i);
            for z in 0isize..(2 * i + 1) {
                println!("z={}", z - i);
                let xy_plane = spaces.index_axis(Axis(2), z as usize);
                for y in 0..spaces.shape()[1] {
                    // Not sure if the row-col major is right here
                    let row = xy_plane.index_axis(Axis(1), y);
                    println!("{}", join(row.iter(), ""));
                }
                println!();
            }
            println!();
            spaces = iterate(&spaces);
        }
        let count = spaces.iter().filter(|&&x| x == Space::Active).count();
        assert_eq!(count, 112);
    }
}
