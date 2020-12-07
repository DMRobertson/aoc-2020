#[macro_use]
extern crate lazy_static;
extern crate regex;
use petgraph::visit::DfsEvent::TreeEdge;
use petgraph::visit::{depth_first_search, DfsPostOrder, Walker};

use petgraph::graphmap::{GraphMap, NodeTrait};
use petgraph::Directed;
use std::collections::HashMap;
use std::str::FromStr;

type Graph<'a> = GraphMap<&'a str, u32, Directed>;

lazy_static! {
    static ref CONTAINED_BAGS: regex::Regex =
        regex::Regex::new(r"(\d+) ([^,]+) bags?[,.]").unwrap();
}

fn get_color(input: &str) -> &str {
    let index = input.find(" bags contain ").unwrap();
    &input[..index]
}

fn parse_contents(src: &str) -> impl Iterator<Item = (u32, &str)> {
    CONTAINED_BAGS.captures_iter(src).map(|cap| {
        let count = cap.get(1).unwrap().as_str();
        let count = u32::from_str(count).unwrap();
        let color = cap.get(2).unwrap().as_str();
        (count, color)
    })
}

fn parse_rules<'a>(input: impl Iterator<Item = &'a str>) -> Graph<'a> {
    let mut g = GraphMap::new();
    for line in input {
        let name = get_color(&line);
        // Ensure we add the node, even if it has no contents.
        g.add_node(name);
        for (count, color) in parse_contents(&line) {
            g.add_edge(name, color, count);
        }
    }
    g
}

// On futher searching I found there's an adapter
// petgraph::algo::Reversed
// which might do what this does. Oh well, I've written it now.
fn reversed<N, E>(src: &GraphMap<N, E, Directed>) -> GraphMap<N, E, Directed>
where
    N: NodeTrait,
    E: Copy,
{
    let mut g = GraphMap::new();
    for (start, end, &weight) in src.all_edges() {
        g.add_edge(end, start, weight);
    }
    g
}

fn shiny_gold_containers<'a>(contained_in: &'a Graph) -> Vec<&'a str> {
    let mut containers = Vec::new();
    depth_first_search(contained_in, Some("shiny gold"), |event| {
        if let TreeEdge(_, b) = event {
            containers.push(b);
        }
    });
    containers
}

fn shiny_gold_contents<'a>(contains: &'a Graph) -> u32 {
    let mut bags_inside = HashMap::new();
    let dfs_postorder = DfsPostOrder::new(contains, "shiny gold");
    for node in dfs_postorder.iter(contains) {
        let count = contains
            .edges(node)
            .map(|(_, dest, weight)| weight * (1 + bags_inside.get(dest).unwrap()))
            .sum();
        bags_inside.insert(node, count);
    }
    bags_inside["shiny gold"]
}

fn main() {
    let lines: Vec<_> = aoc_2020::problem_lines().collect();
    let lines = lines.iter().map(|s| s.as_str());
    let containers = parse_rules(lines);
    let contained_in = reversed(&containers);
    println!("{}", shiny_gold_containers(&contained_in).iter().count());

    println!("{}", shiny_gold_contents(&containers));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example() {
        const RULES: &'static str = "\
light red bags contain 1 bright white bag, 2 muted yellow bags.
dark orange bags contain 3 bright white bags, 4 muted yellow bags.
bright white bags contain 1 shiny gold bag.
muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.
shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.
dark olive bags contain 3 faded blue bags, 4 dotted black bags.
vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.
faded blue bags contain no other bags.
dotted black bags contain no other bags.";
        let lines = RULES.split("\n");
        let containers = parse_rules(lines);
        let contained_in = reversed(&containers);
        let mut gold_containers = shiny_gold_containers(&contained_in);
        gold_containers.sort();
        assert_eq!(
            gold_containers,
            vec!["bright white", "dark orange", "light red", "muted yellow"]
        );

        let gold_contents = shiny_gold_contents(&containers);
        assert_eq!(gold_contents, 32);
    }
}
