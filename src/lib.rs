use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

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

pub fn problem_input() -> impl Iterator<Item=String> {
    let mut path = env::current_dir().unwrap();
    path.push("input");
    path.push(format!("{}.txt", problem_name()));
    println!("{:?}", path);
    BufReader::new(File::open(path).unwrap()).lines()
        .map(|r| r.unwrap())
}
