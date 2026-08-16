use std::io::{self, BufRead};

struct Fenwick {
    tree: Vec<i64>,
}

impl Fenwick {
    fn new(size: usize) -> Self {
        Fenwick {
            tree: vec![0i64; size + 1],
        }
    }

    fn add(&mut self, idx: usize, delta: i64) {
        let n = self.tree.len();
        let mut i = idx + 1;
        while i < n {
            self.tree[i] += delta;
            i += i & i.wrapping_neg();
        }
    }

    fn sum(&self, idx: usize) -> i64 {
        let mut i = idx + 1;
        let mut s = 0i64;
        while i > 0 {
            s += self.tree[i];
            i -= i & i.wrapping_neg();
        }
        s
    }
}

fn main() {
    let stdin = io::stdin();
    let mut tokens: Vec<i64> = Vec::new();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        for tok in line.split_whitespace() {
            if let Ok(v) = tok.parse::<i64>() {
                tokens.push(v);
            }
        }
    }

    let n = tokens[0] as usize;
    let k = tokens[n + 1];

    let size = 2 * n + 1;
    let mut bit_even = Fenwick::new(size);
    let mut bit_odd = Fenwick::new(size);

    let offset = n as i64;
    bit_even.add((0 + offset) as usize, 1);

    let mut prefix = 0i64;
    let mut ans: i64 = 0;

    for i in 1..=n {
        prefix += if tokens[i] > k { 1 } else { -1 };
        let idx = (prefix + offset) as usize;
        let lower = prefix - 1 + offset;

        if i & 1 == 0 {
            ans += bit_even.sum(idx);
            if lower >= 0 {
                ans += bit_odd.sum(lower as usize);
            }
            bit_even.add(idx, 1);
        } else {
            ans += bit_odd.sum(idx);
            if lower >= 0 {
                ans += bit_even.sum(lower as usize);
            }
            bit_odd.add(idx, 1);
        }
    }

    println!("{}", ans);
}
