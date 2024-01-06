use num_enum::TryFromPrimitive;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use array2d::Array2D;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt;
use std::ops::Not;

type Grid = [bool; 100];

#[derive(TryFromPrimitive, Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Edge {
    Top = 0,
    Right = 1,
    Bottom = 2,
    Left = 3,
}

impl Edge {
    pub fn opposite(&self) -> Self {
        use Edge::*;
        match self {
            Top => Bottom,
            Right => Left,
            Bottom => Top,
            Left => Right,
        }
    }

    fn vflip(&self) -> Self {
        use Edge::*;
        match self {
            Top => Bottom,
            Right => Right,
            Bottom => Top,
            Left => Left,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Orientation {
    CW,
    ACW,
}

impl Orientation {
    fn opposite(&self) -> Self {
        match self {
            Orientation::CW => Orientation::ACW,
            Orientation::ACW => Orientation::CW,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct OrientedEdge {
    pub e: Edge,
    pub o: Orientation,
}

impl OrientedEdge {
    pub fn opposite(&self) -> Self {
        Self {
            e: self.e.opposite(),
            o: self.o.opposite(),
        }
    }
}

impl fmt::Display for OrientedEdge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}{:?}", self.e, self.o)
    }
}

const ORIENTED_EDGES: [OrientedEdge; 8] = [
    OrientedEdge {
        e: Edge::Top,
        o: Orientation::CW,
    },
    OrientedEdge {
        e: Edge::Top,
        o: Orientation::ACW,
    },
    OrientedEdge {
        e: Edge::Right,
        o: Orientation::CW,
    },
    OrientedEdge {
        e: Edge::Right,
        o: Orientation::ACW,
    },
    OrientedEdge {
        e: Edge::Bottom,
        o: Orientation::CW,
    },
    OrientedEdge {
        e: Edge::Bottom,
        o: Orientation::ACW,
    },
    OrientedEdge {
        e: Edge::Left,
        o: Orientation::CW,
    },
    OrientedEdge {
        e: Edge::Left,
        o: Orientation::ACW,
    },
];

#[derive(Debug)]
pub struct Tile {
    pub id: usize,
    grid: Grid,
    pub edges: HashMap<OrientedEdge, u16>,
}

lazy_static! {
    static ref FLIP_10: Vec<u16> = {
        let flip = |x: u16| {
            let mut flipped = 0;
            for i in 0..10 {
                if x & (1 << i) != 0 {
                    flipped |= 1 << (9 - i);
                }
            }
            flipped
        };
        (0..(1 << 10)).map(flip).collect()
    };
}

impl Tile {
    fn new(id: usize, grid: Grid) -> Self {
        use Edge::*;
        use Orientation::*;
        let top_cw = Self::edge_sum(grid[0..10].iter());
        let right_cw = Self::edge_sum(grid[9..=99].iter().step_by(10));
        let bottom_cw = Self::edge_sum(grid[90..100].iter().rev());
        let left_cw = Self::edge_sum(grid[0..=90].iter().step_by(10).rev());

        let mut edges = HashMap::new();
        for &(e, cw) in &[
            (Top, top_cw),
            (Right, right_cw),
            (Bottom, bottom_cw),
            (Left, left_cw),
        ] {
            edges.insert(OrientedEdge { e, o: CW }, cw);
            edges.insert(OrientedEdge { e, o: ACW }, FLIP_10[cw as usize]);
        }
        Self { id, grid, edges }
    }

    fn edge_sum<'a>(cells: impl Iterator<Item = &'a bool>) -> u16 {
        let mut sum = 0;
        for &value in cells {
            sum <<= 1;
            sum |= value as u16;
        }
        sum
    }
    pub fn read<'a>(mut input: impl Iterator<Item = &'a str>) -> Option<Self> {
        let id = scan_fmt!(input.next()?, "Tile {d}:", usize).ok()?;
        let mut grid = [false; 100];
        for (row, line) in input.take_while(|line| !line.is_empty()).enumerate() {
            for (col, char) in line.chars().enumerate() {
                grid[10 * row + col] = match char {
                    '.' => false,
                    '#' => true,
                    _ => panic!(),
                }
            }
        }
        Some(Tile::new(id, grid))
    }

    pub fn arrangements<'a>(&'a self) -> impl Iterator<Item = ArrangedTile<'a>> {
        RotoReflection::iter().map(move |r| ArrangedTile {
            tile: &self,
            arrangement: r,
        })
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{}(T{}/{} R{}/{} B{}/{} L{}/{})",
            self.id,
            self.edges[&ORIENTED_EDGES[0]],
            self.edges[&ORIENTED_EDGES[1]],
            self.edges[&ORIENTED_EDGES[2]],
            self.edges[&ORIENTED_EDGES[3]],
            self.edges[&ORIENTED_EDGES[4]],
            self.edges[&ORIENTED_EDGES[5]],
            self.edges[&ORIENTED_EDGES[6]],
            self.edges[&ORIENTED_EDGES[7]],
        )
    }
}

pub fn read_tiles<'a>(mut input: impl Iterator<Item = &'a str>) -> Vec<Tile> {
    let mut tiles = Vec::new();
    while let Some(t) = Tile::read(&mut input) {
        tiles.push(t);
    }
    tiles
}

#[derive(TryFromPrimitive, EnumIter, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RotoReflection {
    None = 0,
    CW90 = 1,
    CW180 = 2,
    CW270 = 3,
    VFlip = 4,
    VFlipCW90 = 5,
    VFlipCW180 = 6,
    VFlipCW270 = 7,
}

impl RotoReflection {
    fn factor(&self) -> (bool, u8) {
        let value = *self as u8;
        (value >= 4, value % 4)
    }

    fn apply(&self, e: &OrientedEdge) -> OrientedEdge {
        let (vflip, cw_quarters) = self.factor();

        let o = match vflip {
            true => e.o.opposite(),
            false => e.o,
        };
        let e = if vflip { e.e.vflip() } else { e.e };
        let index = e as u8;
        let e = Edge::try_from((index + cw_quarters) % 4).unwrap();
        OrientedEdge { e, o }
    }

    fn such_that(src: OrientedEdge, dst: OrientedEdge) -> Self {
        let flip = src.o != dst.o;
        let diff_mod_4 = |x, y| if y >= x { y - x } else { y + 4 - x };

        if !flip {
            let turns = diff_mod_4(src.e as u8, dst.e as u8) % 4;
            Self::try_from(turns).unwrap()
        } else {
            let turns = diff_mod_4(src.e.vflip() as u8, dst.e as u8) % 4;
            Self::try_from(4 + turns).unwrap()
        }
    }

    fn transform(&self, u: usize, v: usize, size: usize) -> (usize, usize) {
        let size = size - 1;
        match self {
            RotoReflection::None => (u, v),
            RotoReflection::CW90 => (size - v, u),
            RotoReflection::CW180 => (size - u, size - v),
            RotoReflection::CW270 => (v, size - u),
            RotoReflection::VFlip => (u, size - v),
            RotoReflection::VFlipCW90 => (v, u),
            RotoReflection::VFlipCW180 => (size - u, v),
            RotoReflection::VFlipCW270 => (size - v, size - u),
        }
    }
}

impl Not for RotoReflection {
    type Output = Self;

    fn not(self) -> Self::Output {
        use RotoReflection::*;
        match self {
            None => None,
            CW90 => CW270,
            CW180 => CW180,
            CW270 => CW90,
            VFlip => VFlip,
            VFlipCW90 => VFlipCW90,
            VFlipCW180 => VFlipCW180,
            VFlipCW270 => VFlipCW270,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ArrangedTile<'a> {
    tile: &'a Tile,
    pub arrangement: RotoReflection,
}

impl<'a> ArrangedTile<'a> {
    fn edge_sum(&self, edge: OrientedEdge) -> u16 {
        // Work out which of the original tile's oriented edges we want.
        // Think of RotoReflection as the group A4, acting on the oriented edges.
        // We want to apply the inverse of self.arrangement to the edge we are interested in to
        // get the edge of the origianl tile.a
        let inverse = !self.arrangement;
        let orig_edge = inverse.apply(&edge);
        self.tile.edges[&orig_edge]
    }

    pub fn such_that(tile: &'a Tile, e: OrientedEdge, desired_e: OrientedEdge) -> Self {
        // Need to find a RotoReflection r such that r.apply(e) == desired_e
        Self {
            tile,
            arrangement: RotoReflection::such_that(e, desired_e),
        }
    }
}

impl fmt::Display for ArrangedTile<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{} ({:?})", self.tile.id, self.arrangement,)
    }
}

impl fmt::Debug for ArrangedTile<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#{}(T{}/{} R{}/{} B{}/{} L{}/{})",
            self.tile.id,
            self.edge_sum(ORIENTED_EDGES[0]),
            self.edge_sum(ORIENTED_EDGES[1]),
            self.edge_sum(ORIENTED_EDGES[2]),
            self.edge_sum(ORIENTED_EDGES[3]),
            self.edge_sum(ORIENTED_EDGES[4]),
            self.edge_sum(ORIENTED_EDGES[5]),
            self.edge_sum(ORIENTED_EDGES[6]),
            self.edge_sum(ORIENTED_EDGES[7]),
        )
    }
}

pub struct Pairs<'a, T> {
    max: usize,
    i: usize,
    j: usize,
    parent: &'a [T],
}

impl<'a, T> Pairs<'a, T> {
    pub fn new(src: &'a [T]) -> Self {
        Self {
            max: src.len(),
            i: 0,
            j: 0,
            parent: src,
        }
    }
}

impl<'a, T> Iterator for Pairs<'a, T> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.j += 1;
        if self.j == self.max {
            self.i += 1;
            self.j = self.i + 1;
        }
        if self.i == self.max - 1 {
            None
        } else {
            Some((&self.parent[self.i], &self.parent[self.j]))
        }
    }
}

pub struct Composition<'a> {
    tiles: Array2D<Option<ArrangedTile<'a>>>,
    pub ids: HashSet<usize>,
}

impl<'a> Composition<'a> {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            tiles: Array2D::filled_with(None, width, height),
            ids: HashSet::new(),
        }
    }

    fn get(&self, x: isize, y: isize) -> Option<ArrangedTile> {
        if 0 <= x
            && x < self.tiles.num_columns() as isize
            && 0 <= y
            && y < self.tiles.num_rows() as isize
        {
            *self.tiles.get(x as usize, y as usize).unwrap()
        } else {
            None
        }
    }

    pub fn get_edge_sum(&self, x: usize, y: usize, e: OrientedEdge) -> Option<u16> {
        let t = self.tiles.get(x, y).unwrap();
        t.map(|t| t.edge_sum(e))
    }

    pub fn clear(&mut self, x: usize, y: usize) {
        println!(
            "Remove {} from ({},{})",
            self.tiles.get(x, y).unwrap().unwrap().tile.id,
            x,
            y
        );
        self.ids
            .remove(&self.tiles.get(x, y).unwrap().unwrap().tile.id);
        self.tiles.set(x, y, None).unwrap();
    }

    fn neighbours(&self, x: usize, y: usize) -> Vec<(Edge, ArrangedTile)> {
        use Edge::*;
        let (x, y) = (x as isize, y as isize);
        let mut neighbours = Vec::new();
        for &(x1, y1, dir) in &[
            (x, y - 1, Top),
            (x + 1, y, Right),
            (x, y + 1, Bottom),
            (x - 1, y, Left),
        ] {
            if let Some(t) = self.get(x1, y1) {
                neighbours.push((dir, t));
            }
        }
        neighbours
    }

    pub fn try_insert(&mut self, t: ArrangedTile<'a>, x: usize, y: usize) -> bool {
        for (dir, t2) in self.neighbours(x, y) {
            let e = OrientedEdge {
                e: dir,
                o: Orientation::CW,
            };
            if t.edge_sum(e) != t2.edge_sum(e.opposite()) {
                return false;
            }
        }
        self.ids.insert(t.tile.id);
        self.tiles.set(x, y, Some(t)).unwrap();
        true
    }

    pub fn contains(&self, id: usize) -> bool {
        // Guess you could just brute force the lookup too
        self.ids.contains(&id)
    }

    pub fn corners(&self) -> usize {
        let x = [0, self.tiles.num_columns() - 1];
        let y = [0, self.tiles.num_rows() - 1];
        x.iter()
            .cartesian_product(y.iter())
            .map(|(x, y)| self.tiles.get(*x, *y).unwrap().unwrap().tile.id)
            .product()
    }

    pub fn assemble(&self) -> Array2D<bool> {
        let width = self.tiles.num_columns() * 8;
        let height = self.tiles.num_rows() * 8;
        let mut image = Array2D::filled_with(false, width, height);
        for (x, y) in (0..self.tiles.num_columns()).cartesian_product(0..self.tiles.num_rows()) {
            let src = &self.tiles.get(x, y).unwrap().unwrap();
            for (u, v) in (0..8).cartesian_product(0..8) {
                let (src_u, src_v) = (!src.arrangement).transform(u, v, 8);
                let i = 10 * src_v + src_u;
                image.set(8 * x + u, 8 * y + v, src.tile.grid[i]).unwrap();
            }
        }
        image
    }
}

pub struct ArrangedSquareBitmap<'a> {
    arrangement: RotoReflection,
    src: &'a Array2D<bool>,
}

impl<'a> ArrangedSquareBitmap<'a> {
    pub fn new(arr: RotoReflection, src: &'a Array2D<bool>) -> Option<Self> {
        if src.num_rows() == src.num_columns() {
            Some(Self {
                arrangement: arr,
                src,
            })
        } else {
            None
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&bool> {
        let (src_x, src_y) = (!self.arrangement).transform(x, y, self.src.num_columns());
        println!("{}, {}", src_x, src_y);
        self.src.get(src_x, src_y)
    }
}

pub fn sqrt(n: usize) -> Option<usize> {
    let fsqrt = (n as f64).sqrt();
    let isqrt = fsqrt as usize;
    if isqrt * isqrt == n {
        Some(isqrt)
    } else {
        None
    }
}

pub type EdgeLookup<'a> = HashMap<u16, Vec<(OrientedEdge, &'a Tile)>>;

pub fn build_edge_lookup(tiles: &[Tile]) -> EdgeLookup {
    let mut map = HashMap::new();
    for t in tiles {
        for (e, value) in t.edges.iter() {
            let v = map.entry(*value).or_insert(Vec::new());
            v.push((*e, t));
        }
    }

    // If there's an edge sum that only appears once, we can't use it to glue tiles together.
    // So throw it away!
    map.retain(|_, v| v.len() >= 2);

    map
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pairs() {
        let v = vec![1, 2, 3, 4];
        let p: Vec<_> = Pairs::new(&v).collect();
        assert_eq!(
            p,
            vec![(&1, &2), (&1, &3), (&1, &4), (&2, &3), (&2, &4), (&3, &4)]
        );
    }

    #[test]
    fn test_edge_sums() {
        let input = "\
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
..###..###";
        let tiles = read_tiles(input.split("\n"));
        let check = |e, o, v| {
            assert_eq!(tiles[0].edges.get(&OrientedEdge { e, o }).unwrap(), &v);
        };
        use Edge::*;
        use Orientation::*;
        check(Top, CW, 0b0011010010);
        check(Right, CW, 0b0001011001);
        check(Bottom, CW, 0b1110011100);
        check(Left, CW, 0b0100111110);

        check(Top, ACW, 0b0100101100);
        check(Left, ACW, 0b0111110010);
        check(Bottom, ACW, 0b0011100111);
        check(Right, ACW, 0b1001101000);
    }

    #[test]
    fn test_get_edge() {
        let mut g = [false; 100];
        g[1] = true;
        g[29] = true;
        g[96] = true;
        g[50] = true;
        let t = Tile::new(0, g);
        assert_eq!(t.to_string(), "#0(T256/2 R128/4 B64/8 L32/16)");

        use RotoReflection::*;

        for &(rr, expectation) in &[
            (None, "#0(T256/2 R128/4 B64/8 L32/16)"),
            (CW90, "#0(T32/16 R256/2 B128/4 L64/8)"),
            (CW180, "#0(T64/8 R32/16 B256/2 L128/4)"),
            (CW270, "#0(T128/4 R64/8 B32/16 L256/2)"),
            (VFlip, "#0(T8/64 R4/128 B2/256 L16/32)"),
            (VFlipCW90, "#0(T16/32 R8/64 B4/128 L2/256)"),
            (VFlipCW180, "#0(T2/256 R16/32 B8/64 L4/128)"),
            (VFlipCW270, "#0(T4/128 R2/256 B16/32 L8/64)"),
        ] {
            let a = ArrangedTile {
                tile: &t,
                arrangement: rr,
            };
            assert_eq!(format!("{:?}", a), expectation);
        }
    }

    #[test]
    fn test_such_that() {
        use Edge::*;
        use Orientation::*;
        use RotoReflection::*;
        let check = |a, b, c, d, v| {
            assert_eq!(
                RotoReflection::such_that(OrientedEdge { e: a, o: b }, OrientedEdge { e: c, o: d }),
                v
            )
        };
        check(Top, CW, Top, CW, None);
        check(Right, CW, Top, CW, CW270);
        check(Top, CW, Right, CW, CW90);
        check(Bottom, ACW, Top, ACW, CW180);
        check(Left, ACW, Bottom, ACW, CW270);
        check(Right, ACW, Left, ACW, CW180);

        check(Top, CW, Bottom, ACW, VFlip);
        check(Top, CW, Top, ACW, VFlipCW180);
        check(Left, CW, Left, ACW, VFlip);
        check(Right, CW, Right, ACW, VFlip);
        check(Right, CW, Left, ACW, VFlipCW180);
        check(Left, CW, Bottom, ACW, VFlipCW270);
        check(Bottom, ACW, Left, CW, VFlipCW270);
    }
}
