use std::fmt;

use strum::IntoEnumIterator;

use aoc_2020::lib20::*;

fn main() {
    // One day I'll work out how to write "an Iterator that yields &str or &String". Until now,
    // collect and feel dirty.
    let lines: Vec<_> = aoc_2020::problem_lines().collect();
    let tiles = read_tiles(lines.iter().map(AsRef::as_ref));
    let tiles_by_edges = build_edge_lookup(&tiles);
    let c = search_for_composition(&tiles, &tiles_by_edges).unwrap();
    println!("{}", c.corners());
}

fn read_tiles<'a>(mut input: impl Iterator<Item = &'a str>) -> Vec<Tile> {
    let mut tiles = Vec::new();
    while let Some(t) = Tile::read(&mut input) {
        tiles.push(t);
    }
    tiles
}

#[derive(Debug)]
struct Possibilities<'a> {
    x: usize,
    y: usize,
    candidate: ArrangedTile<'a>,
    other_options: Vec<ArrangedTile<'a>>,
}

impl fmt::Display for Possibilities<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "At ({},{}) considering {} with {} other alternatives",
            self.x,
            self.y,
            self.candidate,
            self.other_options.len()
        )
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
    let mut options: Vec<ArrangedTile> = tiles.iter().flat_map(|t| t.arrangements()).collect();
    dfs_stack.push(Possibilities {
        x: 0,
        y: 0,
        // Think we can do better here. If there's a solution where the top left has orientation
        // A, then VFlipCw90 that solution to get another solution with the same tile in the top
        // left but a differnet orientation. So that's a factor of 2 to find somewhere.
        candidate: options.pop().unwrap(),
        other_options: options,
    });

    while !dfs_stack.is_empty() {
        println!("Stack:");
        let start = dfs_stack.len().saturating_sub(4);
        dfs_stack
            .iter()
            .enumerate()
            .skip(start)
            .for_each(|(i, x)| println!("  {:3}] {}", i, x));

        // At first I just looked at `head` in place, but the borrow checker wasn't happy.
        // So move its ownership to this function while we investigate it.
        // Pick a tile arrangement out of our options for this square and see if it fits.
        // Try inserting the tile. Does it cause any problems?
        let outcome = try_insertion(
            dfs_stack.iter_mut().last().unwrap(),
            &mut c,
            size,
            &edge_lookup,
        );
        match outcome {
            InsertionOutcome::SuccessComplete => println!("Yes! We're done!"),
            InsertionOutcome::SuccessDescend(_) => println!("Yes. Descend to another search level"),
            InsertionOutcome::InsertionWouldClash => {
                println!("No: insertion would clash with tiles we've already tried")
            }
            InsertionOutcome::SuccessButNoOptions => {
                println!("No: insertion would work but we couldn't place a next tile.")
            }
        };

        match outcome {
            InsertionOutcome::SuccessButNoOptions | InsertionOutcome::InsertionWouldClash => {
                // println!("stack contains {} things", dfs_stack.len());
                while let Some(mut head) = dfs_stack.pop() {
                    // println!("stack now contains {} things", dfs_stack.len());
                    // println!("head we're considering is {}", head);
                    c.clear(head.x, head.y);
                    if let Some(t) = head.other_options.pop() {
                        head.candidate = t;
                        dfs_stack.push(head);
                        break;
                    }
                }
                // match dfs_stack.last() {
                //     None => println!("stack now empty"),
                //     Some(x) => println!("stack now has {} things, head {}", dfs_stack.len(), x),
                // }
            }
            InsertionOutcome::SuccessDescend(p) => {
                dfs_stack.push(p);
            }
            InsertionOutcome::SuccessComplete => {
                return Some(c);
            }
        }
    }
    None
}

enum InsertionOutcome<'a> {
    SuccessComplete,
    SuccessDescend(Possibilities<'a>),
    InsertionWouldClash,
    SuccessButNoOptions,
}

fn try_insertion<'a, 'b>(
    head: &'b mut Possibilities<'a>,
    c: &'b mut Composition<'a>,
    size: usize,
    edge_lookup: &EdgeLookup<'a>,
) -> InsertionOutcome<'a> {
    println!(
        "Can we place {:?} at ({},{})?",
        head.candidate, head.x, head.y
    );

    if c.try_insert(head.candidate, head.x, head.y) {
        // If not, what square should we consider next?
        let cell = next_cell(head.x, head.y, size);
        // Maybe we've considered all square and completed a composition of tiles.
        if cell.is_none() {
            return InsertionOutcome::SuccessComplete;
        }

        // But, more likely, we have to consider extra cells and descend down the dfs tree.
        let (next_x, next_y, next_glue_edge) = cell.unwrap();
        let (src_x, src_y) = step(next_glue_edge.e, next_x, next_y);
        let edge_sum = c
            .get_edge_sum(src_x, src_y, next_glue_edge.opposite())
            .unwrap();
        println!(
            "Yes. To continue, we need to glue ({},{}) edge {} to something with value {}. \
            Options:",
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
        let mut options: Vec<_> = options
            .iter()
            .filter(|(_, t)| !c.contains(t.id))
            .inspect(|(e, t)| println!("    #{} {}", t.id, e))
            .map(|(e, t)| ArrangedTile::such_that(*t, *e, next_glue_edge))
            .collect();

        if !options.is_empty() {
            let candidate = options.pop().unwrap();
            let p = Possibilities {
                x: next_x,
                y: next_y,
                candidate,
                other_options: options,
            };
            InsertionOutcome::SuccessDescend(p)
        } else {
            InsertionOutcome::SuccessButNoOptions
        }
    } else {
        InsertionOutcome::InsertionWouldClash
    }
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
    use std::collections::HashSet;
    use std::iter::FromIterator;

    use array2d::Array2D;
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_edge_lookup() {
        let input = EXAMPLE_ONE.split("\n");
        let tiles1 = read_tiles(input.clone());
        let tiles_by_edges1 = build_edge_lookup(&tiles1);

        let tiles2 = read_tiles(input);
        let tiles_by_edges2 = build_edge_lookup(&tiles2);

        assert_eq!(tiles_by_edges1.len(), tiles_by_edges2.len());
        assert_eq!(
            HashSet::<u16>::from_iter(tiles_by_edges1.keys().cloned()),
            HashSet::<u16>::from_iter(tiles_by_edges2.keys().cloned())
        );
        for k in tiles_by_edges1.keys() {
            let v1 = tiles_by_edges1.get(k).unwrap();
            let v2 = tiles_by_edges1.get(k).unwrap();
            assert_eq!(v1.len(), v2.len());

            assert_eq!(
                HashSet::<(OrientedEdge, usize)>::from_iter(
                    v1.iter().cloned().map(|(e, t)| (e, t.id))
                ),
                HashSet::<(OrientedEdge, usize)>::from_iter(
                    v2.iter().cloned().map(|(e, t)| (e, t.id))
                )
            );
        }
    }

    #[test]
    fn example_1() {
        let input = EXAMPLE_ONE.split("\n");
        let tiles = read_tiles(input);
        let tiles_by_edges = build_edge_lookup(&tiles);
        let c = search_for_composition(&tiles, &tiles_by_edges).unwrap();
        assert_eq!(c.corners(), 20899048083289);
        let image = c.assemble();
        let expected_image = Array2D::from_iter_row_major(
            EXAMPLE_ONE_ASSEMBLED
                .split("\n")
                .flat_map(|s| s.chars().map(|c| c == '#')),
            24,
            24,
        );

        assert_eq!(image.num_rows(), 24);
        assert_eq!(image.num_columns(), 24);
        assert_eq!(expected_image.num_rows(), 24);
        assert_eq!(expected_image.num_columns(), 24);
        println!("{:?}", expected_image);

        let test = |arrangement: RotoReflection| {
            let view = ArrangedSquareBitmap::new(arrangement, &image).unwrap();
            println!("{:?}", arrangement);
            (0..image.num_columns())
                .cartesian_product(0..image.num_rows())
                .all(|(x, y)| expected_image.get(x, y).unwrap() == view.get(x, y).unwrap())
        };
        assert!(RotoReflection::iter().any(test));
    }

    #[test]
    fn example_1_reversed() {
        let input = EXAMPLE_ONE.split("\n");
        let tiles: Vec<_> = read_tiles(input).into_iter().rev().collect();
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

    const EXAMPLE_ONE_ASSEMBLED: &'static str = "\
.#.#..#.##...#.##..#####
###....#.#....#..#......
##.##.###.#.#..######...
###.#####...#.#####.#..#
##.#....#.##.####...#.##
...########.#....#####.#
....#..#...##..#.#.###..
.####...#..#.....#......
#..#.##..#..###.#.##....
#.####..#.####.#.#.###..
###.#.#...#.######.#..##
#.####....##..########.#
##..##.#...#...#.#.#.#..
...#..#..#.#.##..###.###
.#.#....#.##.#...###.##.
###.#...#..#.##.######..
.#.#.###.##.##.#..#.##..
.####.###.#...###.#..#.#
..#.#..#..#.#.#.####.###
#..####...#.#.#.###.###.
#####..#####...###....##
#.##..#..#...#..####...#
.#.###..##..##..####.##.
...###...##...#...#..###";
}
