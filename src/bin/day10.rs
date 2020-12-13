use std::collections::HashMap;
use std::str::FromStr;

use petgraph::graphmap::GraphMap;
use petgraph::Directed;
use petgraph::Direction::Incoming;

type Graph = GraphMap<usize, (), Directed>;

fn main() {
    let mut numbers: Vec<_> = aoc_2020::problem_lines()
        .map(|s| usize::from_str(&s).unwrap())
        .collect();
    numbers.push(0);
    numbers.sort();
    numbers.push(numbers.last().unwrap() + 3);

    let differences = joltage_difference_distribution(&numbers);
    println!("{}", differences[&1] * differences[&3]);

    let g = joltage_graph(&numbers);
    println!("{}", count_joltage_chains(&numbers, &g));
}

fn joltage_difference_distribution(joltages_sorted: &[usize]) -> HashMap<usize, usize> {
    let mut differences = HashMap::new();
    for window in joltages_sorted.windows(2) {
        match window {
            [a, b] => *differences.entry(b - a).or_insert(0) += 1,
            _ => unreachable!(),
        }
    }
    differences
}

fn joltage_graph(joltages_sorted: &[usize]) -> Graph {
    let mut g = Graph::new();
    for &input_joltage in joltages_sorted {
        for jump in &[1, 2, 3] {
            // could do better here---only need to look at index+1, index+2 and index+3
            let target = input_joltage + jump;
            if joltages_sorted.contains(&target) {
                g.add_edge(input_joltage, target, ());
            }
        }
    }
    g
}

fn count_joltage_chains(joltages_sorted: &[usize], g: &Graph) -> usize {
    let target = joltages_sorted.last().unwrap();

    // We want to count the number of paths from 0 to `target` in this DAG.
    // To each node N we will associate the number of paths from 0 to N.
    // We can compute this as:
    //    paths from 0 to N = sum(paths from 0 to P) where P is a direct parent of N
    // and this all works because it's a DAG.

    let mut paths_from_root = HashMap::new();
    // One path of length zero from the root to itself!
    paths_from_root.insert(&0, 1);
    for joltage in joltages_sorted.iter().skip(1) {
        let paths = g
            .neighbors_directed(*joltage, Incoming)
            .map(|src| paths_from_root[&src])
            .sum();
        paths_from_root.insert(joltage, paths);
    }
    paths_from_root[target]
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example() {
        let mut joltages = [16, 10, 15, 5, 1, 11, 7, 19, 6, 12, 4].to_vec();
        joltages.push(0);
        joltages.sort();
        joltages.push(22);
        let differences = joltage_difference_distribution(&joltages);
        assert_eq!(differences.get(&1), Some(&7));
        assert_eq!(differences.get(&3), Some(&5));

        let g = joltage_graph(&joltages);
        assert_eq!(count_joltage_chains(&joltages, &g), 8);
    }
}
