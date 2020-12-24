use std::collections::{HashMap, HashSet};
use std::mem::swap;
use std::rc::Rc;

use itertools::{join, Itertools};
use petgraph::prelude::GraphMap;
use petgraph::visit::depth_first_search;
use petgraph::visit::DfsEvent::Finish;
use petgraph::Directed;
use regex::Regex;

fn main() {
    let lines: Vec<_> = aoc_2020::problem_lines().collect();
    let i = lines.iter().position(|l| l == "").unwrap();
    let rules = &lines[..i];
    let examples = &lines[(i + 1)..];

    let mut lines = rules.iter().map(AsRef::as_ref);
    let rules = read_rules(&mut lines);
    let dependencies = dependency_graph(&rules);

    // Whole language has ~2M words, so test for that with a regex
    let re = build_re(0, &rules, &dependencies);
    let re = format!("^{}$", &re[&0]);
    let re = Regex::new(&re).unwrap();
    println!(
        "I count {} kosher examples",
        examples.iter().filter(|s| re.is_match(&s)).count()
    );

    let patterns_42 = build_languages(42, &rules, &dependencies);
    let patterns_31 = build_languages(31, &rules, &dependencies);

    let l42 = patterns_42[&42].as_lang().unwrap();
    let l31 = patterns_31[&31].as_lang().unwrap();
    println!("lang42: {} words {:?}", l42.len(), l42);
    println!("lang31: {} words {:?}", l42.len(), l31);
    println!(
        "So lang0 = 42 42 31 contains {} words",
        l42.len() * l42.len() * l31.len()
    );
    println!("Intersection: {:?} (hooray!)", l42.intersection(&l31));
    println!(
        "New ruleset accepts {} words",
        examples
            .iter()
            .filter(|s| valid_new_ruleset(s, l42, l31))
            .count()
    );
}

fn valid_new_ruleset(s: &str, l42: &HashSet<String>, l31: &HashSet<String>) -> bool {
    let mut count42 = 0;
    let mut count31 = 0;
    let mut i = 0;
    // fortunately, these are ascii strings so can just fiddle with each byte
    while i + 8 <= s.len() {
        if l42.contains(&s[i..(i + 8)]) {
            count42 += 1;
            i += 8;
        } else {
            break;
        }
    }
    while i + 8 <= s.len() {
        if l31.contains(&s[i..(i + 8)]) {
            count31 += 1;
            i += 8;
        } else {
            break;
        }
    }
    i == s.len() && count42 > count31 && count31 >= 1
}

#[derive(Debug)]
enum Rule {
    Literal(char),
    ChoiceOfSequences(Vec<Vec<usize>>),
    Sequence(Vec<usize>),
}

impl Rule {
    fn parse<'a>(parts: impl Iterator<Item = &'a str>) -> Self {
        let mut parts = parts.peekable();
        let first = parts.peek().unwrap();
        if first.starts_with("\"") {
            return Rule::Literal(first.chars().skip(1).next().unwrap());
        }

        let mut choices = Vec::new();
        let mut sequence = Vec::new();
        for part in parts {
            if part == "|" {
                let mut seq2 = Vec::new();
                swap(&mut sequence, &mut seq2);
                choices.push(seq2);
            } else {
                sequence.push(usize::from_str_radix(part, 10).unwrap());
            }
        }
        choices.push(sequence);
        if choices.len() == 1 {
            Rule::Sequence(choices.remove(0))
        } else {
            Rule::ChoiceOfSequences(choices)
        }
    }
}

fn read_rules<'a>(input: &mut impl Iterator<Item = &'a str>) -> HashMap<usize, Rule> {
    let mut rules = HashMap::new();
    for line in input.take_while(|&s| s != "") {
        let mut parts = line.split(": ");
        let id = usize::from_str_radix(parts.next().unwrap(), 10).unwrap();
        let rule = Rule::parse(parts.next().unwrap().split(" "));
        rules.insert(id, rule);
    }
    rules
}

fn dependency_graph(rules: &HashMap<usize, Rule>) -> GraphMap<usize, (), Directed> {
    let mut g = GraphMap::new();
    for (index, rule) in rules {
        match rule {
            Rule::Literal(_) => {
                g.add_node(*index);
            }
            Rule::Sequence(seq) => {
                seq.iter().for_each(|dependant_index| {
                    g.add_edge(*index, *dependant_index, ());
                });
            }
            Rule::ChoiceOfSequences(choices) => {
                choices
                    .iter()
                    .flat_map(|c| c.iter())
                    .for_each(|dependant_index| {
                        g.add_edge(*index, *dependant_index, ());
                    });
            }
        }
    }
    g
}

#[derive(Debug, Clone)]
enum Pattern {
    Concatenation(Vec<Rc<Pattern>>),
    OneOfPattern(Vec<Rc<Pattern>>),
    Language(HashSet<String>),
}

impl Pattern {
    fn as_lang(&self) -> Option<&HashSet<String>> {
        match self {
            Pattern::Language(s) => Some(s),
            _ => None,
        }
    }

    fn get_languages(patterns: &Vec<Rc<Pattern>>) -> Option<Vec<&HashSet<String>>> {
        let languages: Vec<_> = patterns
            .iter()
            .filter_map(|p| {
                if let Pattern::Language(s) = &**p {
                    Some(s)
                } else {
                    None
                }
            })
            .collect();
        if languages.len() == patterns.len() {
            Some(languages)
        } else {
            None
        }
    }
}

fn build_languages(
    start_index: usize,
    rules: &HashMap<usize, Rule>,
    dependencies: &GraphMap<usize, (), Directed>,
) -> HashMap<usize, Rc<Pattern>> {
    let mut patterns = HashMap::<usize, Rc<Pattern>>::new();
    depth_first_search(dependencies, Some(start_index), |event| match event {
        Finish(index, _) => {
            let pattern_for_seq = |seq: &Vec<usize>| {
                let ps: Vec<_> = seq.iter().map(|i| patterns[i].clone()).collect();
                if ps.len() == 1 {
                    try_simplify((*ps[0]).clone())
                } else {
                    try_simplify(Pattern::Concatenation(ps))
                }
            };
            let pattern = match &rules[&index] {
                Rule::Literal(c) => {
                    let mut set = HashSet::new();
                    set.insert(c.to_string());
                    Pattern::Language(set)
                }
                Rule::ChoiceOfSequences(seqs) => try_simplify(Pattern::OneOfPattern(
                    seqs.iter().map(pattern_for_seq).map(Rc::new).collect(),
                )),
                Rule::Sequence(seq) => pattern_for_seq(seq),
            };
            // println!(
            //     "Rule {}: {:?} generates {:?}",
            //     index, &rules[&index], pattern
            // );
            patterns.insert(index, Rc::new(pattern));
        }
        _ => (),
    });
    patterns
}

fn try_simplify(pattern: Pattern) -> Pattern {
    match &pattern {
        Pattern::Concatenation(patterns) => {
            if let Some(languages) = Pattern::get_languages(patterns) {
                let mut language = HashSet::new();
                language.insert(String::new());
                let language = languages.iter().fold(language, |l, r| {
                    l.iter()
                        .cartesian_product(r.iter())
                        .map(|(a, b)| format!("{}{}", a, b))
                        .collect()
                });
                return Pattern::Language(language);
            }
        }
        Pattern::OneOfPattern(patterns) => {
            if let Some(languages) = Pattern::get_languages(patterns) {
                let mut language = HashSet::new();
                languages
                    .iter()
                    .for_each(|l| language.extend(l.iter().cloned()));
                return Pattern::Language(language);
            }
        }
        Pattern::Language(_) => {}
    }
    pattern
}

fn build_re(
    start_index: usize,
    rules: &HashMap<usize, Rule>,
    dependencies: &GraphMap<usize, (), Directed>,
) -> HashMap<usize, String> {
    let mut patterns = HashMap::new();
    depth_first_search(dependencies, Some(start_index), |event| match event {
        Finish(index, _) => {
            let pattern_for_seq = |seq: &Vec<usize>| join(seq.iter().map(|i| &patterns[i]), "");
            let pattern = match &rules[&index] {
                Rule::Literal(c) => c.to_string(),
                Rule::ChoiceOfSequences(choices) => {
                    let choices = choices.iter().map(pattern_for_seq);
                    format!("({})", join(choices, "|"))
                }
                Rule::Sequence(seq) => pattern_for_seq(seq),
            };
            // println!(
            //     "Rule {}: {:?} generates {:?}",
            //     index, &rules[&index], pattern
            // );
            patterns.insert(index, pattern);
        }
        _ => (),
    });
    patterns
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_1() {
        let input = "\
0: 4 1 5
1: 2 3 | 3 2
2: 4 4 | 5 5
3: 4 5 | 5 4
4: \"a\"
5: \"b\"

ababbb
bababa
abbbab
aaabbb
aaaabbb";
        let mut lines = input.split("\n");
        let rules = read_rules(&mut lines);
        let dependencies = dependency_graph(&rules);
        let patterns = build_languages(0, &rules, &dependencies);
        if let Pattern::Language(l) = &*patterns[&0] {
            assert!(l.contains("ababbb"));
            assert!(!l.contains("bababa"));
            assert!(l.contains("abbbab"));
            assert!(!l.contains("aaabbb"));
            assert!(!l.contains("aaaabbb"));
        }
    }
}
