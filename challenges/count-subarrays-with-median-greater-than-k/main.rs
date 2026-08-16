use std::io::{self, BufRead};

struct Fenwick {
    tree: Vec<i64>,
}

impl Fenwick {
    fn new(size: usize) -> Self {
        Fenwick {
            tree: vec![0; size + 1],
        }
    }

    fn add(&mut self, mut idx: usize, val: i64) {
        while idx < self.tree.len() {
            self.tree[idx] += val;
            idx += idx & idx.wrapping_neg();
        }
    }

    fn sum(&self, mut idx: usize) -> i64 {
        let mut res = 0i64;
        while idx > 0 {
            res += self.tree[idx];
            idx -= idx & idx.wrapping_neg();
        }
        res
    }
}

fn main() {
    let stdin = io::stdin();
    let mut all: Vec<i64> = Vec::new();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        for tok in line.split_whitespace() {
            all.push(tok.parse().unwrap());
        }
    }

    let n = all[0] as usize;
    let k = all[1 + n];

    let mut prefix = vec![0i64; n + 1];
    for i in 0..n {
        let d = if all[1 + i] > k { 1i64 } else { -1i64 };
        prefix[i + 1] = prefix[i] + d;
    }

    let offset = n as i64 + 1;
    let size = 2 * n + 3;
    let mut bit_even = Fenwick::new(size);
    let mut bit_odd = Fenwick::new(size);

    bit_even.add((0 + offset) as usize, 1);

    let mut ans: i64 = 0;
    for r in 1..=n {
        let pr = prefix[r];
        let idx_pr = (pr + offset) as usize;
        let idx_pr_m1 = (pr - 1 + offset) as usize;
        if r % 2 == 0 {
            ans += bit_even.sum(idx_pr) + bit_odd.sum(idx_pr_m1);
            bit_even.add(idx_pr, 1);
        } else {
            ans += bit_odd.sum(idx_pr) + bit_even.sum(idx_pr_m1);
            bit_odd.add(idx_pr, 1);
        }
    }

    println!("{}", ans);
}
