use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let line = lines.next().unwrap().unwrap();
    let n: u64 = line.trim().parse().unwrap();

    let target: u64 = 2 * n + 3;

    let mut k: u32 = 0;
    let mut pow: u64 = 1;
    while pow < target {
        pow *= 3;
        k += 1;
    }

    println!("{}", k);
}
