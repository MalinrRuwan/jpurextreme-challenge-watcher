use std::io::{self, BufRead};

struct Fenwick {
    tree: Vec<i64>,
    n: usize,
}

impl Fenwick {
    fn new(n: usize) -> Self {
        Fenwick {
            tree: vec![0i64; n + 1],
            n,
        }
    }

    fn add(&mut self, mut i: usize, delta: i64) {
        i += 1;
        while i <= self.n {
            self.tree[i] += delta;
            i += i & (!i + 1);
        }
    }

    fn prefix(&self, mut i: usize) -> i64 {
        let mut sum = 0i64;
        while i > 0 {
            sum += self.tree[i];
            i -= i & (!i + 1);
        }
        sum
    }

    fn count_le(&self, v: i64, n: i64) -> i64 {
        if v < -n {
            return 0;
        }
        let idx = (v + n) as usize;
        self.prefix(idx + 1)
    }
}

fn main() {
    let stdin = io::stdin();
    let mut data = String::new();
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                data.push_str(&l);
                data.push(' ');
            }
            Err(_) => break,
        }
    }

    let nums: Vec<i64> = data
        .split_whitespace()
        .map(|t| t.parse().unwrap())
        .collect();

    let n = nums[0] as usize;
    let arr = &nums[1..1 + n];
    let k = nums[1 + n];

    let mut s = vec![0i64; n + 1];
    for i in 0..n {
        s[i + 1] = s[i] + if arr[i] > k { 1 } else { -1 };
    }

    let size = 2 * n + 2;
    let nf = n as i64;
    let mut bit0 = Fenwick::new(size);
    let mut bit1 = Fenwick::new(size);

    bit0.add(0, 1);

    let mut ans: i64 = 0;
    for j in 1..=n {
        let sj = s[j];
        if j % 2 == 0 {
            ans += bit0.count_le(sj, nf);
            ans += bit1.count_le(sj - 1, nf);
        } else {
            ans += bit1.count_le(sj, nf);
            ans += bit0.count_le(sj - 1, nf);
        }

        if j % 2 == 0 {
            bit0.add((sj + nf) as usize, 1);
        } else {
            bit1.add((sj + nf) as usize, 1);
        }
    }

    println!("{}", ans);
}
