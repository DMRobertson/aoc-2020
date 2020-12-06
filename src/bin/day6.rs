use std::collections::HashSet;

fn parse_answers(
    input: &str,
    parse_group_answers: fn(&str) -> HashSet<char>,
) -> Vec<HashSet<char>> {
    input.split("\n\n").map(parse_group_answers).collect()
}

fn parse_group_answers_union(input: &str) -> HashSet<char> {
    let mut answers = HashSet::new();
    for line in input.split("\n") {
        for char in line.chars() {
            if char != ' ' {
                answers.insert(char);
            }
        }
    }
    answers
}

fn parse_group_answers_intersection(input: &str) -> HashSet<char> {
    let get_answers = |line: &str| line.chars().filter(|&c| c != ' ').collect::<HashSet<_>>();
    let mut lines = input.split("\n");
    let mut intersection = get_answers(lines.next().unwrap());
    for line in lines {
        intersection = intersection
            .intersection(&get_answers(line))
            .map(|&c| c)
            .collect();
    }
    intersection
}

fn main() {
    let mut input = aoc_2020::problem_content();
    // HACK: last entry ends with \n, not \n\n which means we treat the empty string as a submission
    input.push('\n');
    let sum: usize = parse_answers(&input, parse_group_answers_union)
        .iter()
        .map(HashSet::len)
        .sum();
    println!("{}", sum);

    let sum: usize = parse_answers(&input, parse_group_answers_intersection)
        .iter()
        .map(HashSet::len)
        .sum();
    println!("{}", sum);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example() {
        const ANSWERS: &'static str = "\
abc

a
b
c

ab
ac

a
a
a
a

b";
        let answers = parse_answers(ANSWERS, parse_group_answers_union);
        assert_eq!(
            answers.iter().map(HashSet::len).collect::<Vec<usize>>(),
            vec![3, 3, 3, 1, 1]
        );

        let answers = parse_answers(ANSWERS, parse_group_answers_intersection);
        assert_eq!(
            answers.iter().map(HashSet::len).collect::<Vec<usize>>(),
            vec![3, 0, 1, 1, 1]
        );
        println!("{:?}", answers);
    }
}
