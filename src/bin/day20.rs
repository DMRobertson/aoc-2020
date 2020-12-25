use aoc_2020::lib20::*;
use std::fmt;

fn main() {}

struct Possibilities<'a> {
    x: usize,
    y: usize,
    options: Vec<ArrangedTile<'a>>,
}

impl fmt::Display for Possibilities<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.options.iter().last() {
            None => write!(f, "At ({},{}) out of alternatives", self.x, self.y),
            Some(last) => write!(
                f,
                "At ({},{}) consider {} (1 of {} alternatives)",
                self.x,
                self.y,
                last,
                self.options.len()
            ),
        }
    }
}

fn search_for_composition<'a>(
    tiles: &'a [Tile],
    edge_lookup: &'a EdgeLookup,
) -> Option<Composition<'a>> {
    println!("search for composition");
    // We have N = n^2 tiles and wish to see if they can be arranged into a nxn square.
    // The number of possibilities is vast: N! ways to organise the tiles into a square,
    // and then 8 roto reflections for each tile, for a total of N! 8^N possibilities.
    // This is going to explode (the example data has N = 144) and so a brute force
    // search is out of the question. (The example data has N = 144.)

    // So we need an efficient way to prune this search space.
    // We use a depth-first search to try and find a valid member of the state space.
    // The state of our search is tracked in a "composition" c.
    let size = sqrt(tiles.len()).unwrap();
    let mut c = Composition::new(size, size);

    // (Note: every solution should appear 8 times accounting for rotations and reflections,
    // because of the symmetry of the problem.)

    // We start by guessing which tile and which orientation could be in the top-left corner.
    // No cleverness here---try them all (8N possibilities).
    let mut dfs_stack = Vec::new();
    dfs_stack.push(Possibilities {
        x: 0,
        y: 0,
        options: tiles.iter().flat_map(|t| t.arrangements()).collect(),
    });

    while !dfs_stack.is_empty() {
        // At first I just looked at `head` in place, but the borrow checker wasn't happy.
        // So move its ownership to this function while we investigate it.
        println!("Stack:");
        dfs_stack.iter().for_each(|x| println!("    {}", x));

        let mut head = dfs_stack.pop().unwrap();
        // Pick a tile arrangement out of our options for this square and see if it fits.
        let t = head.options.pop();
        if t.is_none() {
            println!("This didn't work. Pop!");
            // If we're out of options, then this tile isn't right.
            // Remove it from the grid, then we'll head back up the dfs tree in the next iteration.
            c.clear(head.x, head.y);
            continue;
        }
        let t = t.unwrap();
        // Try inserting t here. Does it cause any problems?
        if c.try_insert(t, head.x, head.y) {
            println!("I **CAN** place {} here", t);
            // If not, what square should we consider next?
            let cell = next_cell(head.x, head.y, size);
            // Maybe we've considered all square and completed a composition of tiles.
            if cell.is_none() {
                println!("We're done!");
                return Some(c);
            }
            // But, more likely, we have to consider extra cells and descend down the dfs tree.
            let (next_x, next_y, next_glue_edge) = cell.unwrap();
            let (src_x, src_y) = step(next_glue_edge.e, next_x, next_y);
            let edge_sum = c
                .get_edge_sum(src_x, src_y, next_glue_edge.opposite())
                .unwrap();
            println!(
                "Need to glue ({},{}) edge {} to something with value {}",
                src_x,
                src_y,
                next_glue_edge.opposite(),
                edge_sum
            );

            const EMPTY: [(OrientedEdge, &Tile); 0] = [];
            let options = match edge_lookup.get(&edge_sum) {
                Some(v) => v.as_slice(),
                None => &EMPTY,
            };
            let options = options.iter().filter(|(_, t)| !c.contains(t.id));
            println!("Our options are:");
            for (e, t) in options.clone() {
                println!("    #{} {}", t.id, e);
            }

            dfs_stack.push(head);

            let options: Vec<_> = options
                .map(|(e, t)| ArrangedTile::such_that(*t, *e, next_glue_edge))
                .collect();
            if !options.is_empty() {
                dfs_stack.push(Possibilities {
                    x: next_x,
                    y: next_y,
                    options,
                });
            }
        } else {
            println!("I can't place {} here", t);
            dfs_stack.push(head);
        }
    }
    None
}

fn next_cell(x: usize, y: usize, size: usize) -> Option<(usize, usize, OrientedEdge)> {
    // The next cell to consider is at (x, y).
    // When choosing a tile to place here, start by looking for a match along the given edge.
    if x == size - 1 {
        if y == size - 1 {
            None
        } else {
            Some((
                0,
                y + 1,
                OrientedEdge {
                    e: Edge::Top,
                    o: Orientation::ACW,
                },
            ))
        }
    } else {
        Some((
            x + 1,
            y,
            OrientedEdge {
                e: Edge::Left,
                o: Orientation::ACW,
            },
        ))
    }
}

fn step(e: Edge, x: usize, y: usize) -> (usize, usize) {
    match e {
        Edge::Top => (x, y - 1),
        Edge::Right => (x + 1, y),
        Edge::Bottom => (x, y - 1),
        Edge::Left => (x - 1, y),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_1() {
        use Edge::*;
        use Orientation::*;
        let mut input = EXAMPLE_ONE.split("\n");
        let mut tiles = Vec::new();
        while let Some(t) = Tile::read(&mut input) {
            tiles.push(t);
        }
        let t = tiles.first().unwrap();
        // TODO move this checking into lib2
        let check = |e, o, v| {
            assert_eq!(t.edges.get(&OrientedEdge { e, o }).unwrap(), &v);
        };
        check(Top, CW, 0b0011010010);
        check(Right, CW, 0b0001011001);
        check(Bottom, CW, 0b1110011100);
        check(Left, CW, 0b0100111110);

        check(Top, ACW, 0b0100101100);
        check(Left, ACW, 0b0111110010);
        check(Bottom, ACW, 0b0011100111);
        check(Right, ACW, 0b1001101000);

        let tiles_by_edges = build_edge_lookup(&tiles);
        let c = search_for_composition(&tiles, &tiles_by_edges).unwrap();
        assert_eq!(c.corners(), 20899048083289);
    }

    const EXAMPLE_ONE: &'static str = "\
Tile 2311:
..##.#..#.
##..#.....
#...##..#.
####.#...#
##.##.###.
##...#.###
.#.#.#..##
..#....#..
###...#.#.
..###..###

Tile 1951:
#.##...##.
#.####...#
.....#..##
#...######
.##.#....#
.###.#####
###.##.##.
.###....#.
..#.#..#.#
#...##.#..

Tile 1171:
####...##.
#..##.#..#
##.#..#.#.
.###.####.
..###.####
.##....##.
.#...####.
#.##.####.
####..#...
.....##...

Tile 1427:
###.##.#..
.#..#.##..
.#.##.#..#
#.#.#.##.#
....#...##
...##..##.
...#.#####
.#.####.#.
..#..###.#
..##.#..#.

Tile 1489:
##.#.#....
..##...#..
.##..##...
..#...#...
#####...#.
#..#.#.#.#
...#.#.#..
##.#...##.
..##.##.##
###.##.#..

Tile 2473:
#....####.
#..#.##...
#.##..#...
######.#.#
.#...#.#.#
.#########
.###.#..#.
########.#
##...##.#.
..###.#.#.

Tile 2971:
..#.#....#
#...###...
#.#.###...
##.##..#..
.#####..##
.#..####.#
#..#.#..#.
..####.###
..#.#.###.
...#.#.#.#

Tile 2729:
...#.#.#.#
####.#....
..#.#.....
....#..#.#
.##..##.#.
.#.####...
####.#.#..
##.####...
##..#.##..
#.##...##.

Tile 3079:
#.#.#####.
.#..######
..#.......
######....
####.#..#.
.#...#.##.
#.#####.##
..#.###...
..#.......
..#.###...";
}
