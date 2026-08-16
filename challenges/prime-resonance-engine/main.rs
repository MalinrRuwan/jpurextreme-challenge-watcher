use std::io::{self, BufRead};

fn is_prime(x: u32) -> bool {
    if x < 2 {
        return false;
    }
    if x % 2 == 0 {
        return x == 2;
    }
    let mut d: u32 = 3;
    while d * d <= x {
        if x % d == 0 {
            return false;
        }
        d += 2;
    }
    true
}

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let n: usize = lines
        .next()
        .unwrap()
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let a: Vec<u32> = lines
        .next()
        .unwrap()
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse().unwrap())
        .collect();

    let full: usize = (1usize << n) - 1;
    let mut dp = vec![0u64; 1usize << n];
    dp[0] = 1;

    for mask in 1..(1usize << n) {
        if mask.count_ones() % 2 != 0 {
            continue;
        }
        let i = mask.trailing_zeros() as usize;
        let mut m = mask;
        while m != 0 {
            let j = m.trailing_zeros() as usize;
            m &= m - 1;
            if j == i {
                continue;
            }
            if is_prime(a[i] + a[j]) {
                let sub = mask & !(1usize << i) & !(1usize << j);
                dp[mask] += dp[sub];
            }
        }
    }

    println!("{}", dp[full]);
}
