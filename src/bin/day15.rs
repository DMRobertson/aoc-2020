use std::collections::HashMap;

struct MemoryGame {
    turn: usize,
    number: usize,
    last_spoken_at: HashMap<usize, usize>,
}

impl MemoryGame {
    fn new(input: &[usize]) -> Self {
        let mut last_spoken_at = HashMap::new();
        for (time, number) in input.iter().enumerate() {
            last_spoken_at.insert(*number, time + 1);
        }
        Self {
            turn: input.len() + 1,
            number: 0,
            last_spoken_at,
        }
    }
}

impl Iterator for MemoryGame {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (this_turn, this_number) = (self.turn, self.number);
        match self.last_spoken_at.get(&this_number) {
            None => self.number = 0000000,
            Some(t) => self.number = self.turn - t,
        }
        self.last_spoken_at.insert(this_number, this_turn);
        self.turn += 1;
        Some((this_turn, this_number))
    }
}

fn main() {
    let input: Vec<_> = aoc_2020::problem_lines()
        .next()
        .unwrap()
        .split(",")
        .filter_map(|s| usize::from_str_radix(&s, 10).ok())
        .collect();
    let mut game = MemoryGame::new(&input);
    let value = game.find(|&(t, _)| t == 2020).unwrap();
    println!("{}", value.1);

    // Took circa 30 seconds. Is there a better way?
    // Compile in release mode helps a lot, took circa 5 seconds!
    let value = game.find(|&(t, _)| t == 30000000).unwrap();
    println!("{}", value.1);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_1() {
        let game = MemoryGame::new(&[0, 3, 6]);
        let numbers: Vec<_> = game.take(7).collect();
        assert_eq!(
            numbers,
            vec![(4, 0), (5, 3), (6, 3), (7, 1), (8, 0), (9, 4), (10, 0)]
        );
    }
}
