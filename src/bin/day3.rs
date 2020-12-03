#[derive(Copy, Clone, PartialEq, Debug)]
enum Tile { Open, Tree }

impl Tile {
    fn parse(c: char) -> Option<Self> {
        match c {
            '.' => Some(Tile::Open),
            '#' => Some(Tile::Tree),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Map {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
}

impl Map {
    fn parse<'a>(mut lines: impl Iterator<Item=&'a str>) -> Self {
        let mut tiles: Vec<Tile> = lines.next().unwrap().chars()
            .map(Tile::parse)
            .map(Option::unwrap)
            .collect();
        let width = tiles.len();
        let mut height = 1;
        for line in lines {
            line.chars()
                .map(Tile::parse)
                .map(Option::unwrap)
                .for_each(|t| tiles.push(t));
            height += 1;
        }
        Self { width, height, tiles }
    }

    fn at(&self, location: [usize; 2]) -> Option<Tile> {
        if location[1] < self.height {
            let index = (location[0] % self.width) + (self.width * location[1]);
            Some(self.tiles[index])
        } else {
            None
        }
    }
}

struct TobogganRide<'a> {
    location: [usize; 2],
    direction: [usize; 2],
    map: &'a Map,
}

impl Iterator for TobogganRide<'_> {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.map.at(self.location)
            .map(|c| {
                self.location[0] += self.direction[0];
                self.location[1] += self.direction[1];
                c
            })
    }
}

const DIRECTIONS: [[usize; 2]; 5] = [
    [1, 1],
    [3, 1],
    [5, 1],
    [7, 1],
    [1, 2],
];

fn main() {
    let lines: Vec<_> = aoc_2020::problem_input().collect();
    let map = Map::parse(lines.iter().map(|s| s.as_str()));
    let counts: Vec<_> = DIRECTIONS.iter()
        .map(|dir| trees_hit(&map, *dir))
        .collect();
    println!("{:?}", counts);
    println!("{}", counts.iter().product::<usize>());
}

fn trees_hit(map: &Map, direction: [usize; 2]) -> usize {
    let path = TobogganRide { location: [0, 0], direction, map: &map };
    path
        .filter(|&t| t == Tile::Tree)
        .count()
}

#[cfg(test)]
mod test {
    use super::*;

    const LINES: &'static str = "\
..##.......
#...#...#..
.#....#..#.
..#.#...#.#
.#...##..#.
..#.##.....
.#.#.#....#
.#........#
#.##...#...
#...##....#
.#..#...#.#";

    #[test]
    fn example_toboggan_ride()
    {
        let map = Map::parse(lines.split('\n'));
        assert_eq!(trees_hit(&map, [3, 1]), 7);
    }

    #[test]
    fn lots_of_toboggan_rides() {
        let map = Map::parse(lines.split('\n'));
        let trees_hit = DIRECTIONS.iter().map(|dir| trees_hit(&map, direction)).collect();
        assert_eq!(trees_hit, vec![2, 7, 3, 4, 2]);
        assert_eq!(trees_hit.iter().product::<usize>(), 336);
    }
}