use aoc_2020::lib20::*;
use petgraph::prelude::NodeIndex;

fn main() {}

struct Possibilities<'a> {
    x: usize,
    y: usize,
    options: Vec<ArrangedTile<'a>>,
}

fn search_for_composition<'a>(
    tiles: &'a [Tile],
    g: &Graph,
    node_indices: &[NodeIndex<u32>],
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

        let mut head = dfs_stack.pop().unwrap();
        // Pick a tile arrangement out of our options for this square and see if it fits.
        let t = head.options.pop();
        if t.is_none() {
            // If we're out of options, then this tile isn't right.
            // Remove it, then proceed back up the dfs tree.
            c.clear(head.x, head.y);
            dfs_stack.pop();
            continue;
        }
        let t = t.unwrap();
        // Does inserting t here cause any problems?
        if c.try_insert(t, head.x, head.y) {
            // If not and the grid is complete, we're done!
            if head.x == size && head.y == size {
                return Some(c);
            }
            // If we were successful but have more squares to handle,
            // descend down the dfs tree.
            let next_possibility = Possibilities {
                x: (head.x + 1) % size,
                y: head.y + (head.x == size - 1) as usize,
                // TODO: query the graph for tile arrangements which we could put here
                options: Vec::new(),
            };
            dfs_stack.push(head);
            dfs_stack.push(next_possibility);
        } else {
            dfs_stack.push(head);
        }
    }
    None
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

        let (g, node_indices) = build_graph(&tiles);
        search_for_composition(&tiles, &g, &node_indices);
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
