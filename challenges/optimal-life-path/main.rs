use std::io::{self, Read};

struct Solver {
    m: usize,
    adj: Vec<Vec<u32>>,
    parent: Vec<usize>,
    heavy: Vec<usize>,
    value: Vec<i32>,
}

impl Solver {
    fn dfs(&self, u: usize) -> Vec<i32> {
        let m = self.m;
        let vu = self.value[u];
        let h = self.heavy[u];
        if h == usize::MAX {
            return vec![vu; m + 1];
        }
        let mut a = self.dfs(h);
        for s in (1..=m).rev() {
            let cand = a[s - 1] - vu;
            if cand > a[s] {
                a[s] = cand;
            }
        }
        for &v in &self.adj[u] {
            let v = v as usize;
            if v == self.parent[u] || v == h {
                continue;
            }
            let b = self.dfs(v);
            for s in 0..=m {
                let mut cand = b[s];
                if s >= 1 {
                    let c2 = b[s - 1] - vu;
                    if c2 > cand {
                        cand = c2;
                    }
                }
                if cand > a[s] {
                    a[s] = cand;
                }
            }
        }
        for x in a.iter_mut() {
            *x += vu;
        }
        a
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut it = input.split_whitespace();

    let n: usize = it.next().unwrap().parse().unwrap();
    let m: usize = it.next().unwrap().parse().unwrap();

    let mut adj = vec![Vec::new(); n];
    for _ in 0..n - 1 {
        let a: usize = it.next().unwrap().parse().unwrap();
        let b: usize = it.next().unwrap().parse().unwrap();
        adj[a].push(b as u32);
        adj[b].push(a as u32);
    }

    let mut value = Vec::with_capacity(n);
    for _ in 0..n {
        value.push(it.next().unwrap().parse::<i32>().unwrap());
    }

    let mut parent = vec![n; n];
    let mut order = Vec::with_capacity(n);
    let mut stack = vec![0usize];
    while let Some(u) = stack.pop() {
        order.push(u);
        for &v in &adj[u] {
            let v = v as usize;
            if v != parent[u] {
                parent[v] = u;
                stack.push(v);
            }
        }
    }

    let mut size = vec![1usize; n];
    for &u in order.iter().rev() {
        for &v in &adj[u] {
            let v = v as usize;
            if v != parent[u] {
                size[u] += size[v];
            }
        }
    }

    let mut heavy = vec![usize::MAX; n];
    for &u in order.iter() {
        let mut best = usize::MAX;
        let mut best_size = 0usize;
        for &v in &adj[u] {
            let v = v as usize;
            if v != parent[u] && size[v] > best_size {
                best_size = size[v];
                best = v;
            }
        }
        heavy[u] = best;
    }

    let solver = Solver {
        m,
        adj,
        parent,
        heavy,
        value,
    };

    let handle = std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || {
            let dp = solver.dfs(0);
            dp[solver.m]
        })
        .unwrap();

    let ans = handle.join().unwrap();
    println!("{}", ans);
}
