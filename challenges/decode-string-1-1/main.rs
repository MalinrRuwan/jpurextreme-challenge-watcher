use std::io::{self, BufRead};

fn decode(s: &str) -> String {
    let mut counts: Vec<usize> = Vec::new();
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut num: usize = 0;
    let mut has_num = false;

    for c in s.chars() {
        if c.is_ascii_digit() {
            num = num * 10 + (c as u8 - b'0') as usize;
            has_num = true;
        } else if c == '[' {
            counts.push(num);
            parts.push(std::mem::take(&mut current));
            num = 0;
            has_num = false;
        } else if c == ']' {
            let count = counts.pop().expect("valid input");
            let mut base = parts.pop().expect("valid input");
            for _ in 0..count {
                base.push_str(&current);
            }
            current = base;
        } else {
            if has_num {
                current.push_str(&num.to_string());
                num = 0;
                has_num = false;
            }
            current.push(c);
        }
    }
    if has_num {
        current.push_str(&num.to_string());
    }
    current
}

fn main() {
    let stdin = io::stdin();
    let input = stdin
        .lock()
        .lines()
        .filter_map(|line| line.ok())
        .collect::<Vec<_>>()
        .join("\n");
    let s = input.trim();
    println!("{}", decode(s));
}
