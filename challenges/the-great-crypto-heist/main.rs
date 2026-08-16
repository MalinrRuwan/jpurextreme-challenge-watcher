use std::collections::HashMap;
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    let _n: usize = lines
        .next()
        .unwrap()
        .unwrap()
        .trim()
        .parse()
        .unwrap();

    let arr: Vec<i64> = lines
        .next()
        .unwrap()
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse::<i64>().unwrap())
        .collect();

    let mut counts: HashMap<i64, i64> = HashMap::new();
    for &v in &arr {
        *counts.entry(v).or_insert(0) += 1;
    }

    let mut keys: Vec<i64> = counts.keys().cloned().collect();
    keys.sort();

    let mut prev2: i64 = 0;
    let mut prev1: i64 = 0;
    let mut prev_key: i64 = i64::MIN;

    for &k in &keys {
        let gain = k * counts[&k];
        let cur = if prev_key == k - 1 {
            prev1.max(prev2 + gain)
        } else {
            prev1 + gain
        };
        prev2 = prev1;
        prev1 = cur;
        prev_key = k;
    }

    println!("{}", prev1);
}
