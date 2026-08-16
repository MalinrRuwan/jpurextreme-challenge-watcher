use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    let first_line = lines.next().unwrap().unwrap();
    let mut it = first_line.split_whitespace();
    let n: usize = it.next().unwrap().parse().unwrap();
    let k: usize = it.next().unwrap().parse().unwrap();
    let s: i64 = it.next().unwrap().parse().unwrap();

    let mut a: Vec<i64> = Vec::with_capacity(n);
    for line in lines {
        for tok in line.unwrap().split_whitespace() {
            a.push(tok.parse().unwrap());
            if a.len() == n {
                break;
            }
        }
        if a.len() == n {
            break;
        }
    }

    let mut left: usize = 0;
    let mut sum: i64 = 0;
    for right in 0..n {
        sum += a[right];
        while sum > s && left <= right {
            sum -= a[left];
            left += 1;
        }
        if sum == s && left <= right && right - left + 1 <= k {
            println!("{} {}", left, right);
            return;
        }
    }
    println!("-1");
}
