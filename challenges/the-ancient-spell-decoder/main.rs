use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines().map(|l| l.unwrap());

    let k: usize = lines.next().unwrap().trim().parse().unwrap();

    // Trie node: (count of dictionary words passing through, children as (byte, child_index))
    // Root is node 0. Child indices are u32 (max nodes <= sum_of_lengths + 1 <= 10^6 + 1).
    let mut trie: Vec<(u32, Vec<(u8, u32)>)> = vec![(0u32, Vec::new())];

    for _ in 0..k {
        let word = lines.next().unwrap();
        let bytes = word.as_bytes();
        let mut node = 0usize;
        for &b in bytes {
            let idx = b - b'a';
            let mut next = None;
            {
                let children = &trie[node].1;
                for &(c, child) in children.iter() {
                    if c == idx {
                        next = Some(child as usize);
                        break;
                    }
                }
            }
            node = match next {
                Some(n) => n,
                None => {
                    let new_id = trie.len() as u32;
                    trie[node].1.push((idx, new_id));
                    trie.push((0u32, Vec::new()));
                    new_id as usize
                }
            };
            trie[node].0 += 1;
        }
    }

    let scroll = lines.next().unwrap();
    let bytes = scroll.as_bytes();

    let mut out: Vec<String> = Vec::with_capacity(bytes.len());
    let mut node = 0usize;
    let mut broken = false;

    for &b in bytes {
        if broken {
            out.push("0".to_string());
            continue;
        }
        let idx = b - b'a';
        let mut next = None;
        {
            let children = &trie[node].1;
            for &(c, child) in children.iter() {
                if c == idx {
                    next = Some(child as usize);
                    break;
                }
            }
        }
        match next {
            Some(n) => {
                node = n;
                out.push(trie[node].0.to_string());
            }
            None => {
                broken = true;
                out.push("0".to_string());
            }
        }
    }

    println!("{}", out.join(" "));
}
