use itertools::join;
use petgraph::graphmap::GraphMap;
use petgraph::visit::depth_first_search;
use petgraph::visit::DfsEvent::Finish;
use petgraph::Directed;
use regex::Regex;
use std::collections::HashMap;
use std::mem::swap;

fn main() {
    let lines: Vec<_> = aoc_2020::problem_lines().collect();
    let mut lines = lines.iter().map(AsRef::as_ref);
    let mut rules = read_rules(&mut lines);
    let dependencies = dependency_graph(&rules);

    let original_patterns = build_patterns(0, &rules, &dependencies);
    let original_matches: Vec<_> = {
        let pattern = format!("^{}$", &original_patterns[&0]);
        let re = Regex::new(&pattern).unwrap();
        lines.clone().filter(|s| re.is_match(s)).collect()
    };
    println!("{}", original_matches.len());
    println!("{:?}", original_patterns);

    rules.insert(8, Rule::OneOrMore(42));
    rules.insert(11, Rule::Balanced(42, 31));
    let new_patterns = build_patterns(0, &rules, &dependencies);
    let candidate_larger_matches: Vec<_> = {
        let re = Regex::new(&format!("^{}$", &new_patterns[&0])).unwrap();
        println!("{}", re.as_str());
        lines
            .filter_map(|s| {
                re.captures(s)
                    .map(|c| c.name(&"balanced").unwrap().as_str())
            })
            .collect()
    };

    let mut pattern = String::new();
    for i in 1..=100 {
        // candidate_larger_matches looks for stuff in a language of the form A*B*:
        // but we want the sublanguage containing the same number of As as Bs, i.e. the union
        // over natural numbers i of A^iB^i.
        // Try some small values of i:
        pattern = format!(
            "^{}{}{}$",
            original_patterns[&42], pattern, original_patterns[&31]
        );
        let re = Regex::new(&pattern).unwrap();
        println!(
            "Match {} times on both sides: {}",
            i,
            candidate_larger_matches
                .iter()
                .filter(|s| re.is_match(s))
                .count()
        );
    }
}

#[derive(Debug)]
enum Rule {
    Literal(char),
    ChoiceOfSequences(Vec<Vec<usize>>),
    Sequence(Vec<usize>),
    OneOrMore(usize),
    Balanced(usize, usize),
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
            Rule::OneOrMore(other_index) => {
                g.add_edge(*index, *other_index, ());
            }
            Rule::Balanced(a, b) => {
                g.add_edge(*index, *a, ());
                g.add_edge(*index, *b, ());
            }
        }
    }
    g
}

fn build_patterns(
    start_index: usize,
    rules: &HashMap<usize, Rule>,
    dependencies: &GraphMap<usize, (), Directed>,
) -> HashMap<usize, String> {
    let mut patterns = HashMap::<usize, String>::new();
    depth_first_search(dependencies, Some(start_index), |event| match event {
        Finish(index, _) => {
            let pattern = match &rules[&index] {
                Rule::Literal(c) => String::from(*c),
                Rule::Sequence(seq) => join(seq.iter().map(|i| &patterns[i]), ""),
                Rule::ChoiceOfSequences(choices) => {
                    let fmt_seq = |seq: &Vec<usize>| join(seq.iter().map(|i| &patterns[i]), "");
                    format!("({})", join(choices.iter().map(fmt_seq), "|"))
                }
                Rule::OneOrMore(other) => {
                    format!("({})+", patterns.get(other).unwrap())
                }
                Rule::Balanced(a, b) => {
                    format!(
                        "(?P<balanced>({})+({})+)",
                        patterns.get(a).unwrap(),
                        patterns.get(b).unwrap()
                    )
                }
            };
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
        let patterns = build_patterns(0, &rules, &dependencies);
        println!("{:?}", patterns);
        let pattern = format!("^{}$", patterns[&0]);
        println!("{}", pattern);
        let re = Regex::new(&pattern).unwrap();
        assert!(re.is_match("ababbb"));
        assert!(!re.is_match("bababa"));
        assert!(re.is_match("abbbab"));
        assert!(!re.is_match("aaabbb"));
        assert!(!re.is_match("aaaabbb"));
    }

    #[test]
    fn example_two() {
        let input = "\
42: 9 14 | 10 1
9: 14 27 | 1 26
10: 23 14 | 28 1
1: \"a\"
11: 42 31
5: 1 14 | 15 1
19: 14 1 | 14 14
12: 24 14 | 19 1
16: 15 1 | 14 14
31: 14 17 | 1 13
6: 14 14 | 1 14
2: 1 24 | 14 4
0: 8 11
13: 14 3 | 1 12
15: 1 | 14
17: 14 2 | 1 7
23: 25 1 | 22 14
28: 16 1
4: 1 1
20: 14 14 | 1 15
3: 5 14 | 16 1
27: 1 6 | 14 18
14: \"b\"
21: 14 1 | 1 14
25: 1 1 | 1 14
22: 14 14
8: 42
26: 14 22 | 1 20
18: 15 15
7: 14 5 | 1 21
24: 14 1

abbbbbabbbaaaababbaabbbbabababbbabbbbbbabaaaa
bbabbbbaabaabba
babbbbaabbbbbabbbbbbaabaaabaaa
aaabbbbbbaaaabaababaabababbabaaabbababababaaa
bbbbbbbaaaabbbbaaabbabaaa
bbbababbbbaaaaaaaabbababaaababaabab
ababaaaaaabaaab
ababaaaaabbbaba
baabbaaaabbaaaababbaababb
abbbbabbbbaaaababbbbbbaaaababb
aaaaabbaabaaaaababaa
aaaabbaaaabbaaa
aaaabbaabbaaaaaaabbbabbbaaabbaabaaa
babaaabbbaaabaababbaabababaaab
aabbbbbaabbbaaaaaabbbbbababaaaaabbaaabba";

        let mut lines = input.split("\n");
        let mut rules = read_rules(&mut lines);
        let dependencies = dependency_graph(&rules);
        let patterns = build_patterns(0, &rules, &dependencies);
        let pattern = format!("^{}$", &patterns[&0]);
        let re = Regex::new(&pattern).unwrap();

        assert_eq!(lines.clone().filter(|s| re.is_match(s)).count(), 3);

        println!("11: 42 31 | 42 11 31");
        println!("42: {}", patterns[&42]);
        println!("31: {}", patterns[&31]);
        rules.insert(8, Rule::OneOrMore(42));
        rules.insert(11, Rule::Balanced(42, 31));
        let patterns = build_patterns(0, &rules, &dependencies);
        let re = Regex::new(&patterns[&0]).unwrap();
        let expected_matches: Vec<_> = "\
bbabbbbaabaabba
babbbbaabbbbbabbbbbbaabaaabaaa
aaabbbbbbaaaabaababaabababbabaaabbababababaaa
bbbbbbbaaaabbbbaaabbabaaa
bbbababbbbaaaaaaaabbababaaababaabab
ababaaaaaabaaab
ababaaaaabbbaba
baabbaaaabbaaaababbaababb
abbbbabbbbaaaababbbbbbaaaababb
aaaaabbaabaaaaababaa
aaaabbaabbaaaaaaabbbabbbaaabbaabaaa
aabbbbbaabbbaaaaaabbbbbababaaaaabbaaabba"
            .lines()
            .collect();
        let larger_matches: Vec<_> = lines.filter(|s| re.is_match(s)).collect();
        for e in expected_matches {
            assert!(larger_matches.contains(&e));
        }
    }
}
