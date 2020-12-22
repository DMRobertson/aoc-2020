#[macro_use]
extern crate scan_fmt;

use array2d::Array2D;
use bimap::BiMap;
use itertools::{join, Itertools};
use std::collections::{HashMap, VecDeque};
use std::ops::RangeInclusive;

type Rule = [RangeInclusive<usize>; 2];
type Rules = HashMap<String, Rule>;

#[derive(Debug)]
struct Ticket {
    values: Vec<usize>,
}

#[derive(Debug)]
struct Notes {
    rules: Rules,
    my_ticket: Ticket,
    nearby_tickets: Vec<Ticket>,
}

impl Notes {
    fn parse<'a, T>(input: T) -> Self
    where
        T: IntoIterator<Item = &'a str>,
    {
        let mut lines = input.into_iter();
        let rules = Self::parse_rules(lines.by_ref().take_while(|&s| s != ""));

        lines.find(|&s| s == "your ticket:");
        let my_ticket = Self::parse_ticket(lines.next().unwrap());

        lines.find(|&s| s == "nearby tickets:");
        let nearby_tickets = lines.map(Self::parse_ticket).collect();
        Self {
            rules,
            my_ticket,
            nearby_tickets,
        }
    }

    fn parse_rules<'a, L>(lines: L) -> Rules
    where
        L: Iterator<Item = &'a str>,
    {
        let mut rules = HashMap::new();
        for line in lines {
            if let Ok((name, a, b, c, d)) = scan_fmt!(
                line,
                "{/[^:]+/}: {d}-{d} or {d}-{d}",
                String,
                usize,
                usize,
                usize,
                usize
            ) {
                rules.insert(name, [a..=b, c..=d]);
            }
        }
        rules
    }

    fn parse_ticket(input: &str) -> Ticket {
        let values = input
            .split(",")
            .map(|s| usize::from_str_radix(s, 10).unwrap())
            .collect();
        Ticket { values }
    }
}

fn test_validity(ticket: &Ticket, rules: &Rules) -> Option<usize> {
    ticket
        .values
        .iter()
        .find(|v| {
            !rules
                .values()
                .any(|[range1, range2]| range1.contains(v) || range2.contains(v))
        })
        .cloned()
}

fn dump<'a>(
    possibilities: &Array2D<bool>,
    known: &BiMap<&str, usize>,
    rules: impl Iterator<Item = &'a str>,
) {
    let columns: Vec<usize> = (0..possibilities.num_columns())
        .filter(|i| !known.contains_right(i))
        .collect();
    print!("\n                        ");
    for i in &columns {
        print!(" {:2}", i);
    }
    println!();
    for (name_index, name) in rules.enumerate() {
        if known.contains_left(&name) {
            continue;
        }
        let entries = columns
            .iter()
            .map(|&i| possibilities.get(name_index, i).unwrap())
            .map(|&b| match b {
                true => 't',
                false => ' ',
            });
        println!("{:25} {}", name, join(entries, "  "));
    }
    println!("Known: {:?}\n", known);
}

fn name_index_known_for_index(index: usize, possibilities: &Array2D<bool>) -> Option<usize> {
    let mut matches = possibilities
        .column_iter(index)
        .enumerate()
        .filter(|(_, b)| **b)
        .map(|(i, _)| i);
    match (matches.next(), matches.next()) {
        (s, None) => s,
        (_, Some(_)) => None,
    }
}

fn index_known_for_name_index(name_index: usize, possibilities: &Array2D<bool>) -> Option<usize> {
    let mut matches = possibilities
        .row_iter(name_index)
        .enumerate()
        .filter(|(_, b)| **b)
        .map(|(i, _)| i);
    match (matches.next(), matches.next()) {
        (s, None) => s,
        (_, Some(_)) => None,
    }
}

enum Consider {
    Row,
    Column,
}

#[derive(PartialEq)]
enum Investigate {
    Row,
    Column,
    Both,
}

fn deduce_indices<'a>(rules: &'a Rules, tickets: &[&Ticket]) -> BiMap<&'a str, usize> {
    // Track whether each (name, index) possibility is possible in a matrix.
    // To do so, fix an ordering of the rules.
    let rules = rules.iter().collect_vec();
    let mut possibilities = Array2D::filled_with(true, rules.len(), rules.len());
    let mut known: BiMap<&str, usize> = BiMap::with_capacity(rules.len());

    let mut example_values = tickets.iter().flat_map(|t| t.values.iter().enumerate());
    let mut recently_marked_false = VecDeque::new();

    while known.len() < rules.len() {
        match recently_marked_false.pop_front() {
            Some((name_index, index, Consider::Column)) => {
                if let Some(other_name_index) = name_index_known_for_index(index, &possibilities) {
                    println!(
                        "Looking at columns, now {} must be {}",
                        rules[other_name_index].0, index
                    );
                    dump(
                        &possibilities,
                        &known,
                        rules.iter().map(|(name, _)| name.as_str()),
                    );
                    known.insert(rules[other_name_index].0.as_str(), index);
                    for other_index in (0..possibilities.num_columns()).filter(|&o| o != index) {
                        mark_false(
                            &mut possibilities,
                            &mut recently_marked_false,
                            other_name_index,
                            rules[other_name_index].0,
                            other_index,
                            Investigate::Column,
                        );
                    }
                }
            }
            Some((name_index, index, Consider::Row)) => {
                if let Some(other_index) = index_known_for_name_index(name_index, &possibilities) {
                    println!(
                        "Looking at rows, now {} must be {}",
                        rules[name_index].0, other_index
                    );
                    dump(
                        &possibilities,
                        &known,
                        rules.iter().map(|(name, _)| name.as_str()),
                    );
                    known.insert(rules[name_index].0.as_str(), other_index);
                    for other_name_index in
                        (0..possibilities.num_rows()).filter(|&o| o != name_index)
                    {
                        mark_false(
                            &mut possibilities,
                            &mut recently_marked_false,
                            other_name_index,
                            rules[other_name_index].0,
                            other_index,
                            Investigate::Row,
                        );
                    }
                }
            }
            None => {
                if let Some((index, value)) = example_values.next() {
                    if known.contains_right(value) {
                        continue;
                    }
                    for (name_index, (name, [r1, r2])) in rules.iter().enumerate() {
                        if known.contains_left(&name.as_str()) {
                            continue;
                        }
                        if !possibilities.get(name_index, index).unwrap() {
                            continue;
                        }
                        if r1.contains(value) || r2.contains(value) {
                            continue;
                        }
                        // print!("Since {} isn't in {:?} | {:?}, ", value, r1, r2);
                        mark_false(
                            &mut possibilities,
                            &mut recently_marked_false,
                            name_index,
                            rules[name_index].0,
                            index,
                            Investigate::Both,
                        );
                    }
                } else {
                    break;
                }
            }
        }
    }
    known
}

fn mark_false(
    possibilities: &mut Array2D<bool>,
    recently_marked_false: &mut VecDeque<(usize, usize, Consider)>,
    name_index: usize,
    name: &str,
    index: usize,
    investigate: Investigate,
) {
    if *possibilities.get(name_index, index).unwrap() {
        // println!("{} can't be {}", name, index);
        if investigate == Investigate::Row || investigate == Investigate::Both {
            recently_marked_false.push_back((name_index, index, Consider::Row));
        }
        if investigate == Investigate::Column || investigate == Investigate::Both {
            recently_marked_false.push_back((name_index, index, Consider::Column));
        }
        possibilities.set(name_index, index, false).unwrap();
    }
}

fn filter_completely_invalid<'a>(rules: &Rules, tickets: &'a [Ticket]) -> (Vec<&'a Ticket>, usize) {
    let mut sum = 0;
    let mut remaining = Vec::new();
    for ticket in tickets {
        match test_validity(ticket, rules) {
            None => remaining.push(ticket),
            Some(bad_value) => sum += bad_value,
        }
    }
    (remaining, sum)
}

fn main() {
    let input = aoc_2020::problem_content();
    let notes = Notes::parse(input.split("\n"));
    let (remaining, sum_bad_entries) =
        filter_completely_invalid(&notes.rules, &notes.nearby_tickets);
    println!("{}", sum_bad_entries);
    let mapping = deduce_indices(&notes.rules, &remaining);
    let departure_prod = mapping
        .iter()
        .inspect(|(name, &i)| println!("{}, {}, {}", name, i, notes.my_ticket.values[i]))
        .filter(|(name, _)| name.starts_with("departure"))
        .map(|(_, &i)| notes.my_ticket.values[i])
        .product::<usize>();
    println!("{}", departure_prod);

    for (name, [r1, r2]) in &notes.rules {
        let &index = mapping.get_by_left(&name.as_str()).unwrap();
        for ticket in &remaining {
            assert!(r1.contains(&ticket.values[index]) || r2.contains(&ticket.values[index]));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_1() {
        let input = "\
class: 1-3 or 5-7
row: 6-11 or 33-44
seat: 13-40 or 45-50

your ticket:
7,1,14

nearby tickets:
7,3,47
40,4,50
55,2,20
38,6,12";
        let notes = Notes::parse(input.split("\n"));
        let sum_bad_entries = filter_completely_invalid(&notes.rules, &notes.nearby_tickets).1;
        assert_eq!(sum_bad_entries, 71);
    }

    #[test]
    fn example_2() {
        let input = "\
class: 0-1 or 4-19
row: 0-5 or 8-19
seat: 0-13 or 16-19

your ticket:
11,12,13

nearby tickets:
3,9,18
15,1,5
5,14,9";
        let notes = Notes::parse(input.split("\n"));
        let remaining = filter_completely_invalid(&notes.rules, &notes.nearby_tickets).0;
        let mapping = deduce_indices(&notes.rules, &remaining);
        assert_eq!(mapping.get_by_left(&"row"), Some(&0));
        assert_eq!(mapping.get_by_left(&"class"), Some(&1));
        assert_eq!(mapping.get_by_left(&"seat"), Some(&2));
    }
}
