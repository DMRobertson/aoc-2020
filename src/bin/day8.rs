use std::collections::HashSet;
use std::str::FromStr;

#[derive(Clone)]
enum Instruction {
    Accumulate(isize),
    Jump(isize),
    NoOp(isize),
}

impl Instruction {
    fn parse(input: &str) -> Option<Self> {
        let value = isize::from_str(&input[4..]).ok()?;
        match &input[..3] {
            "acc" => Some(Self::Accumulate(value)),
            "jmp" => Some(Self::Jump(value)),
            "nop" => Some(Self::NoOp(value)),
            _ => None,
        }
    }
}

struct Execution {
    ip: usize,
    acc: isize,
    program: Vec<Instruction>,
    ip_history: HashSet<usize>,
}

impl<'a> Execution {
    fn from_str(input: impl Iterator<Item = &'a str>) -> Option<Self> {
        let program: Option<Vec<_>> = input.map(Instruction::parse).collect();
        Some(Self::new(program?))
    }

    fn new(program: Vec<Instruction>) -> Self {
        Self {
            ip: 0,
            acc: 0,
            program,
            ip_history: HashSet::new(),
        }
    }
}

impl Execution {
    fn step_once(&mut self) {
        self.ip_history.insert(self.ip);
        match self.program[self.ip] {
            Instruction::Accumulate(delta) => {
                self.acc += delta;
                self.ip += 1;
            }
            Instruction::Jump(delta) => {
                let next = (self.ip as isize) + delta;
                self.ip = next as usize;
            }
            Instruction::NoOp(_) => self.ip += 1,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum ProgramOutcome {
    InfiniteLoop,
    Terminated,
}

fn run(state: &mut Execution) -> ProgramOutcome {
    loop {
        // Blurb says "immediately after", but let's err on caution and choose >=
        if state.ip >= state.program.len() {
            return ProgramOutcome::Terminated;
        } else if state.ip_history.contains(&state.ip) {
            return ProgramOutcome::InfiniteLoop;
        } else {
            state.step_once();
        }
    }
}

fn find_fixed(program: &Vec<Instruction>) -> Option<Execution> {
    for (index, instruction) in program.iter().enumerate() {
        let mut altered = (*program).clone();
        match *instruction {
            Instruction::Accumulate(_) => continue,
            Instruction::Jump(value) => altered[index] = Instruction::NoOp(value),
            Instruction::NoOp(value) => altered[index] = Instruction::Jump(value),
        }
        let mut execution = Execution::new(altered);
        match run(&mut execution) {
            ProgramOutcome::InfiniteLoop => continue,
            ProgramOutcome::Terminated => return Some(execution),
        }
    }
    None
}

fn main() {
    let lines: Vec<_> = aoc_2020::problem_lines().collect();
    let mut execution = Execution::from_str(lines.iter().map(String::as_str)).unwrap();
    run(&mut execution);
    println!("{}", execution.acc);

    let fixed = find_fixed(&execution.program).unwrap();
    println!("{}", fixed.acc);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example() {
        const PROGRAM: &'static str = "\
nop +0
acc +1
jmp +4
acc +3
jmp -3
acc -99
acc +1
jmp -4
acc +6";
        let mut execution = Execution::from_str(PROGRAM.split("\n")).unwrap();
        let outcome = run(&mut execution);
        assert_eq!(outcome, ProgramOutcome::InfiniteLoop);
        assert_eq!(execution.acc, 5);

        let fixed = find_fixed(&execution.program).unwrap();
        assert_eq!(fixed.acc, 8);
    }
}
