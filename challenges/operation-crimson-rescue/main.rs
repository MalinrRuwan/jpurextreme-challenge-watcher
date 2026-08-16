use std::collections::VecDeque;
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let tokens: Vec<String> = stdin
        .lock()
        .lines()
        .filter_map(|line| line.ok())
        .flat_map(|line| {
            line.split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .collect();

    let n: usize = tokens[0].parse().unwrap();
    let m: usize = tokens[1].parse().unwrap();

    let mut grid = vec![vec!['0'; m]; n];
    let mut start = (0usize, 0usize);
    let mut target = (0usize, 0usize);

    let chars: Vec<char> = tokens[2..].iter().flat_map(|t| t.chars()).collect();
    for i in 0..n {
        for j in 0..m {
            let ch = chars[i * m + j];
            grid[i][j] = ch;
            if ch == 'R' {
                start = (i, j);
            } else if ch == 'B' {
                target = (i, j);
            }
        }
    }

    let mut dist = vec![vec![-1i32; m]; n];
    let mut queue = VecDeque::new();
    dist[start.0][start.1] = 0;
    queue.push_back(start);

    let dirs: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    while let Some((r, c)) = queue.pop_front() {
        if (r, c) == target {
            break;
        }
        for (dr, dc) in dirs.iter() {
            let nr = r as i32 + dr;
            let nc = c as i32 + dc;
            if nr < 0 || nr >= n as i32 || nc < 0 || nc >= m as i32 {
                continue;
            }
            let (nr, nc) = (nr as usize, nc as usize);
            if grid[nr][nc] == '1' {
                continue;
            }
            if dist[nr][nc] == -1 {
                dist[nr][nc] = dist[r][c] + 1;
                queue.push_back((nr, nc));
            }
        }
    }

    if dist[target.0][target.1] == -1 {
        println!("TRAPPED");
    } else {
        println!("{}", dist[target.0][target.1]);
    }
}
