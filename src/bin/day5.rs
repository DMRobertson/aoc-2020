use std::ops::RangeInclusive;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Seat {
    row: u8,
    column: u8,
}

impl Seat {
    fn id(&self) -> u16 {
        self.row as u16 * 8 + self.column as u16
    }

    fn new(desc: &str) -> Self {
        let mut rows: RangeInclusive<u8> = 0..=128;
        let mut half_width = 64;
        for char in desc[0..7].chars() {
            match char {
                'F' => rows = *rows.start()..=(*rows.start() + half_width),
                'B' => rows = (*rows.end() - half_width)..=*rows.end(),
                _ => unreachable!(),
            }
            half_width /= 2;
        }

        let mut cols: RangeInclusive<u8> = 0..=8;
        let mut half_height = 4;
        for char in desc[7..10].chars() {
            match char {
                'L' => cols = *cols.start()..=(*cols.start() + half_height),
                'R' => cols = (*cols.end() - half_height)..=*cols.end(),
                _ => unreachable!(),
            }
            half_height /= 2;
        }
        Self {
            row: *rows.start(),
            column: *cols.start(),
        }
    }
}

fn main() {
    let lines = aoc_2020::problem_lines();
    let mut ids: Vec<_> = lines.map(|x| Seat::new(&x).id()).collect();

    println!("{}", ids.iter().max().unwrap());

    ids.sort();
    for i in 1..(ids.len() - 1) {
        if ids[i] + 2 == ids[i + 1] {
            println!("{}", ids[i] + 1);
            break;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn example(desc: &str, row: u8, column: u8, id: u16) {
        let seat = Seat::new(desc);
        assert_eq!(seat.row, row);
        assert_eq!(seat.column, column);
        assert_eq!(seat.id(), id);
    }

    #[test]
    fn test() {
        example("FBFBBFFRLR", 44, 5, 357);
        example("BFFFBBFRRR", 70, 7, 567);
        example("FFFBBBFRRR", 14, 7, 119);
        example("BBFFBBFRLL", 102, 4, 820);
    }
}
