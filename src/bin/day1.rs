use std::str::FromStr;
use std::collections::HashSet;

fn two_entries_summing_to(target: usize, entries: &HashSet<usize>)
                      -> Option<[usize; 2]> {
    for first in entries.iter().filter(|&&e| e < target) {
        let second = target - first;
        if entries.contains(&second) {
            return Some([*first, second])
        }
    }
    None
}

fn three_entries_summing_to(target: usize, entries: &HashSet<usize>)
                          -> Option<[usize; 3]> {
    for first in entries.iter().filter(|&&e| e < target) {
        let subtarget = target - first;
        if let Some([b,c]) = two_entries_summing_to(subtarget, entries) {
            return Some([*first, b, c])
        }
    }
    None
}


fn main() {
    let expenses = aoc_2020::problem_input()
        .map(|s| usize::from_str(&s).unwrap());
    let expenses: HashSet<usize> = expenses.collect();
    
    if let Some([a, b]) = two_entries_summing_to(2020, &expenses) {
        println!("[{},{}] -> {}", a, b, a*b);
    }

    if let Some([a, b, c]) = three_entries_summing_to(2020, &expenses) {
        println!("[{},{},{}] -> {}", a, b, c, a*b*c);
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use super::*;

    #[test]
    fn example_expense_entries(){
        let expenses: [usize; 6] = [1721,979,366,299,675,1456];
        let expenses: HashSet<usize> = expenses.iter().cloned().collect();

        let mut answer1 = two_entries_summing_to(2020, &expenses).unwrap();
        answer1.sort();
        assert_eq!(answer1, [299,1721]);

        let mut answer2 = three_entries_summing_to(2020, &expenses).unwrap();
        answer2.sort();
        assert_eq!(answer2, [366,675, 979]);

    }
}