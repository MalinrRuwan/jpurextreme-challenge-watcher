use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut tokens: Vec<String> = Vec::new();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        for t in line.split_whitespace() {
            tokens.push(t.to_string());
        }
    }
    let mut it = tokens.iter();

    let n: usize = it.next().unwrap().parse().unwrap();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for _ in 0..n.saturating_sub(1) {
        let u: usize = it.next().unwrap().parse().unwrap();
        let v: usize = it.next().unwrap().parse().unwrap();
        adj[u].push(v);
        adj[v].push(u);
    }

    let mut delivery = vec![false; n];
    for i in 0..n {
        let x: usize = it.next().unwrap().parse().unwrap();
        delivery[i] = x == 1;
    }

    let mut parent = vec![usize::MAX; n];
    let mut depth = vec![0usize; n];
    let mut order = Vec::with_capacity(n);
    let mut stack = vec![0usize];
    parent[0] = 0;
    while let Some(u) = stack.pop() {
        order.push(u);
        for &v in &adj[u] {
            if v == parent[u] {
                continue;
            }
            parent[v] = u;
            depth[v] = depth[u] + 1;
            stack.push(v);
        }
    }

    let mut has_delivery = vec![false; n];
    let mut edge_count: usize = 0;
    let mut max_depth: usize = 0;
    for &u in order.iter().rev() {
        if delivery[u] {
            has_delivery[u] = true;
            if depth[u] > max_depth {
                max_depth = depth[u];
            }
        }
        if u != 0 && has_delivery[u] {
            edge_count += 1;
            has_delivery[parent[u]] = true;
        }
    }

    let answer = if edge_count == 0 { 0 } else { 2 * edge_count - max_depth };
    println!("{}", answer);
}
