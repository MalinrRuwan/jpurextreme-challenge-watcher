use std::io::{self, BufRead};

fn decode(s: &str) -> String {
    let mut count_stack: Vec<usize> = Vec::new();
    let mut string_stack: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut num: usize = 0;

    for c in s.chars() {
        if c.is_ascii_digit() {
            num = num * 10 + c.to_digit(10).unwrap() as usize;
        } else if c == '[' {
            count_stack.push(num);
            string_stack.push(current.clone());
            current = String::new();
            num = 0;
        } else if c == ']' {
            let count = count_stack.pop().unwrap();
            let mut prev = string_stack.pop().unwrap();
            for _ in 0..count {
                prev.push_str(&current);
            }
            current = prev;
        } else {
            current.push(c);
        }
    }

    current
}

fn main() {
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();
    let input = line.trim();
    println!("{}", decode(input));
}
