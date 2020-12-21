#[macro_use]
extern crate scan_fmt;

use std::collections::HashMap;
use std::convert::TryFrom;

use num_enum::TryFromPrimitive;

#[derive(Clone, Copy, TryFromPrimitive, Debug, PartialEq, Eq)]
#[repr(u8)]
enum MaskBit {
    Zero = b'0',
    One = b'1',
    Unset = b'X',
}

type Mask = [MaskBit; 36];

fn apply_mask(mask: &Mask, value: &u64) -> u64 {
    let mut output = value.clone();
    for i in 0..36 {
        let bit = 1 << (35 - i);
        match mask[i] {
            MaskBit::Zero => output &= !bit,
            MaskBit::One => output |= bit,
            MaskBit::Unset => (),
        }
    }
    output
}

fn get_addrs(mask: &Mask, addr: &u64) -> Vec<u64> {
    // First pass to apply all the 1 bits
    let mut base_addr = *addr;
    for i in 0..36 {
        if mask[i] == MaskBit::One {
            let bit = 1 << (35 - i);
            base_addr |= bit;
        }
    }

    // Second pass to apply all the floating bits
    let mut addrs = vec![base_addr];
    for i in 0..36 {
        let bit = 1 << (35 - i);
        match mask[i] {
            MaskBit::Zero | MaskBit::One => (),
            MaskBit::Unset => {
                for j in 0..addrs.len() {
                    addrs[j] &= !bit;
                    addrs.push(addrs[j] | bit);
                }
            }
        }
    }
    addrs
}

struct Computer {
    mask: Mask,
    memory: HashMap<u64, u64>,
}

impl Computer {
    fn new() -> Self {
        Self {
            mask: [MaskBit::Unset; 36],
            memory: HashMap::new(),
        }
    }

    fn execute_v1(&mut self, ins: &Instruction) {
        match ins {
            Instruction::SetMask(m) => self.mask = m.clone(),
            Instruction::WriteMem(addr, value) => {
                self.memory.insert(*addr, apply_mask(&self.mask, value));
            }
        }
    }

    fn execute_v2(&mut self, ins: &Instruction) {
        match ins {
            Instruction::SetMask(m) => self.mask = m.clone(),
            Instruction::WriteMem(addr, value) => {
                for addr in get_addrs(&self.mask, addr) {
                    self.memory.insert(addr, *value);
                }
            }
        }
    }
}

enum Instruction {
    SetMask(Mask),
    WriteMem(u64, u64),
}

impl Instruction {
    fn parse(input: &str) -> Self {
        if let Ok(mask) = scan_fmt!(input, "mask = {}", String) {
            return Self::SetMask(parse_mask(&mask));
        }

        if let Ok((addr, value)) = scan_fmt!(input, "mem[{d}] = {d}", u64, u64) {
            return Self::WriteMem(addr, value);
        }

        unimplemented!() // should really return Option<Self> and provide None here
    }
}

fn parse_mask(input: &str) -> Mask {
    let mut mask = [MaskBit::Unset; 36];
    for (index, byte) in input.bytes().enumerate() {
        mask[index] = MaskBit::try_from(byte).unwrap();
    }
    mask
}

fn main() {
    let lines: Vec<_> = aoc_2020::problem_lines().collect();

    for exec in &[Computer::execute_v1, Computer::execute_v2] {
        let mut computer = Computer::new();
        for line in lines.iter().map(|s| s.as_str()) {
            exec(&mut computer, &Instruction::parse(&line));
        }
        println!(
            "{}",
            computer.memory.values().map(|&x| x as usize).sum::<usize>()
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_1() {
        let input = "\
mask = XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X
mem[8] = 11
mem[7] = 101
mem[8] = 0";
        let mut computer = Computer::new();
        let instructions: Vec<_> = input.split("\n").map(Instruction::parse).collect();

        computer.execute_v1(&instructions[0]);
        computer.execute_v1(&instructions[1]);
        assert_eq!(computer.memory.get(&8u64), Some(&73u64));
        computer.execute_v1(&instructions[2]);
        assert_eq!(computer.memory.get(&7u64), Some(&101u64));
        computer.execute_v1(&instructions[3]);
        assert_eq!(computer.memory.get(&8u64), Some(&64u64));
    }

    #[test]
    fn example_2() {
        let input = "\
mask = 000000000000000000000000000000X1001X
mem[42] = 100
mask = 00000000000000000000000000000000X0XX
mem[26] = 1";
        let mut computer = Computer::new();
        let instructions: Vec<_> = input.split("\n").map(Instruction::parse).collect();

        computer.execute_v2(&instructions[0]);
        computer.execute_v2(&instructions[1]);
        for k in &[26, 27, 58, 59] {
            assert_eq!(computer.memory.get(k), Some(&100));
        }
    }
}
