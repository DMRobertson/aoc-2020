use std::collections::HashMap;
use std::str::FromStr;

fn main() {
    let numbers: Vec<_> = aoc_2020::problem_lines()
        .map(|s| usize::from_str(&s).unwrap())
        .collect();
    let invalid = find_invalid_number(&numbers, 25).unwrap();
    println!("{}", invalid);
    let subseq = find_subsequence_with_sum(&numbers, invalid).unwrap();
    println!(
        "{}",
        subseq.iter().min().unwrap() + subseq.iter().max().unwrap()
    );
}

fn find_invalid_number(input: &[usize], window_size: usize) -> Option<usize> {
    if input.len() <= window_size {
        return None;
    }
    // We're going to keep track of how many different ways there are
    // in the window before our current location of making the different sums.
    let mut counts: HashMap<usize, usize> = HashMap::new();
    for i in 0..(window_size - 1) {
        for j in (i + 1)..window_size {
            *counts.entry(input[i] + input[j]).or_insert(0) += 1;
        }
    }
    for (index, candidate) in input.iter().enumerate().skip(window_size) {
        if counts.get(candidate).unwrap_or(&0) == &0 {
            return Some(*candidate);
        }
        // Decrement the count for all sums involving the element at the start of the window.
        let to_remove = index - window_size;
        for value in &input[(to_remove + 1)..index] {
            let sum = input[to_remove] + value;
            *counts.get_mut(&sum).unwrap() -= 1;
        }
        // Increment the count for all sums involving the candidate
        for value in &input[(to_remove + 1)..index] {
            let sum = value + candidate;
            *counts.entry(sum).or_insert(0) += 1;
        }
    }
    None
}

fn find_subsequence_with_sum(input: &[usize], target: usize) -> Option<&[usize]> {
    // There are 2 ** n subsets to worry about, but only O(n^2) (nth triangle number)
    // subsequences to worry about. So sod it, let's try all n(n+1)/2 of them.
    for i in 0..(input.len() - 1) {
        // Loop through the subsequences starting at index i
        let mut sum = input[i];
        for j in (i + 1)..(input.len()) {
            sum += input[j];
            if sum == target {
                return Some(&input[i..=j]);
            }
        }
    }
    None
    // *Remark*. We could split `input` by numbers n which are `target` or larger, because
    // any subsequence of length >= 2 containing such a number must sum to more than the target.
    // (Okay, I've made the assumption here that all numbers in the input are positive.)
    // Then we could run the function above on each part and return early if any part finds a match.
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_example_1() {
        let mut numbers: Vec<_> = (1..=25).collect();

        numbers.push(26);
        assert_eq!(find_invalid_number(&numbers, 25), None);
        numbers[25] = 49;
        assert_eq!(find_invalid_number(&numbers, 25), None);
        numbers[25] = 100;
        assert_eq!(find_invalid_number(&numbers, 25), Some(100));
        numbers[25] = 50;
        assert_eq!(find_invalid_number(&numbers, 25), Some(50));
    }

    #[test]
    fn example_2() {
        let mut numbers: Vec<_> = (1..=20).rev().collect();
        numbers.extend_from_slice(&[21, 22, 23, 24, 25, 45]);

        numbers.push(26);
        assert_eq!(find_invalid_number(&numbers, 25), None);
        numbers[26] = 65;
        assert_eq!(find_invalid_number(&numbers, 25), Some(65));
        numbers[26] = 64;
        assert_eq!(find_invalid_number(&numbers, 25), None);
        numbers[26] = 66;
        assert_eq!(find_invalid_number(&numbers, 25), None);
    }

    #[test]
    fn example_window_5() {
        let numbers = [
            35, 20, 15, 25, 47, 40, 62, 55, 65, 95, 102, 117, 150, 182, 127, 219, 299, 277, 309,
            576,
        ];
        assert_eq!(find_invalid_number(&numbers, 5), Some(127));
        assert_eq!(
            find_subsequence_with_sum(&numbers, 127),
            Some(&numbers[2..6])
        );
    }
}
