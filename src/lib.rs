use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::{BufRead, Read};

// Lots of unwraps here---naughty helpers!

fn problem_name() -> String {
    env::current_exe()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}

fn problem_input() -> BufReader<File> {
    let mut path = env::current_dir().unwrap();
    path.push("input");
    path.push(format!("{}.txt", problem_name()));
    BufReader::new(File::open(path).unwrap())
}

pub fn problem_lines() -> impl Iterator<Item = String> {
    problem_input().lines().map(|r| r.unwrap())
}

pub fn problem_content() -> String {
    let mut s = String::new();
    problem_input().read_to_string(&mut s).unwrap();
    s
}
