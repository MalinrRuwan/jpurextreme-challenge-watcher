use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

const DEFAULT_CONTEST: &str = "jpuraxtreme-3-0-inter-univeristy-section";
const DEFAULT_INTERVAL_SECS: u64 = 15;
const FETCH_SCRIPT: &str = "cloak/fetch.js";
const CHALLENGES_DIR: &str = "challenges";
const SEEN_FILE: &str = "challenges/.seen.json";
const LAST_FETCH_FILE: &str = "challenges/.last_fetch.json";
const SOLVE_MODEL: &str = "opencode-go/deepseek-v4-flash";
const ENV_FILE: &str = ".hkwatch.env";

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Challenge {
    slug: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    body_html: String,
    #[serde(default)]
    problem_statement: String,
    #[serde(default)]
    input_format: String,
    #[serde(default)]
    output_format: String,
    #[serde(default)]
    constraints: String,
    #[serde(default)]
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct FetchResult {
    contest: String,
    #[serde(default)]
    fetched_at: String,
    challenges: Vec<Challenge>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Seen {
    slugs: Vec<String>,
}

fn repo_root() -> PathBuf {
    let root = std::env::current_dir().expect("cannot read cwd");
    if root.join("watcher").join("Cargo.toml").exists() {
        root
    } else {
        root
    }
}

fn load_env_file() {
    if let Ok(contents) = fs::read_to_string(ENV_FILE) {
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                if std::env::var_os(k).is_none() {
                    std::env::set_var(k, v);
                }
            }
        }
    }
}

fn run_fetch(contest: &str, headless: bool) -> Result<FetchResult, String> {
    let root = repo_root();
    let mut cmd = Command::new("node");
    cmd.arg(root.join(FETCH_SCRIPT));
    cmd.arg(contest);
    if headless {
        cmd.arg("--headless");
    }
    if has_credentials() {
        cmd.arg("--login");
    }
    let out = cmd
        .output()
        .map_err(|e| format!("failed to run fetch.js: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    let text = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(&text).map_err(|e| format!("bad fetch JSON: {e}"))
}

fn load_seen() -> Seen {
    fs::read_to_string(SEEN_FILE)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_seen(seen: &Seen) {
    let _ = fs::create_dir_all(CHALLENGES_DIR);
    let json = serde_json::to_string_pretty(seen).unwrap();
    let _ = fs::write(SEEN_FILE, json);
}

fn load_last_fetch() -> Option<FetchResult> {
    fs::read_to_string(LAST_FETCH_FILE)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

fn mark_solved(slug: &str) {
    let mut seen = load_seen();
    if !seen.slugs.iter().any(|s| s == slug) {
        seen.slugs.push(slug.to_string());
        seen.slugs.sort();
        save_seen(&seen);
    }
}

fn play_ring() {
    let _ = Command::new("afplay")
        .arg("/System/Library/Sounds/Glass.aiff")
        .spawn();
    let _ = std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(900));
        let _ = Command::new("afplay")
            .arg("/System/Library/Sounds/Glass.aiff")
            .spawn();
    });
}

fn handle_poll(result: &FetchResult, ring: bool) -> Vec<Challenge> {
    let _ = fs::create_dir_all(CHALLENGES_DIR);
    let _ = fs::write(LAST_FETCH_FILE, serde_json::to_string_pretty(result).unwrap());

    let seen = load_seen();
    let seen_set: BTreeSet<&str> = seen.slugs.iter().map(|s| s.as_str()).collect();
    let new_challenges: Vec<Challenge> = result
        .challenges
        .iter()
        .filter(|c| !seen_set.contains(c.slug.as_str()))
        .cloned()
        .collect();

    for c in &new_challenges {
        let name = if c.name.is_empty() { &c.slug } else { &c.name };
        println!("NEW: [{}] {}", c.slug, name);
    }
    if new_challenges.is_empty() {
        println!("No new challenges ({} seen).", seen_set.len());
    } else if ring {
        play_ring();
    }
    new_challenges
}

fn check(contest: &str, headless: bool, ring: bool) -> Result<Vec<Challenge>, String> {
    let result = run_fetch(contest, headless)?;
    Ok(handle_poll(&result, ring))
}

fn has_credentials() -> bool {
    !std::env::var("HKWATCH_USERNAME").unwrap_or_default().is_empty()
        && !std::env::var("HKWATCH_PASSWORD").unwrap_or_default().is_empty()
}

fn watch_persistent(
    contest: &str,
    headless: bool,
    interval: u64,
    model: &str,
    no_solve: bool,
    skip_current: bool,
    ring: bool,
) -> Result<(), String> {
    let root = repo_root();
    let mut cmd = Command::new("node");
    cmd.arg(root.join(FETCH_SCRIPT))
        .arg(contest)
        .arg("--watch")
        .arg("--interval")
        .arg(interval.to_string());
    if headless {
        cmd.arg("--headless");
    }
    if has_credentials() {
        cmd.arg("--login");
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::inherit());

    println!("Watching {contest} every {interval}s (browser stays open, Ctrl-C to stop)");
    if no_solve {
        println!("--no-solve: new challenges will be reported and rung, not solved.");
    }

    let mut first = true;
    loop {
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("failed to spawn fetch.js: {e}"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or("no stdout from fetch.js")?;
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            let line = line.map_err(|e| format!("read poll line: {e}"))?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<FetchResult>(line) {
                Ok(result) => {
                    let new_challenges = handle_poll(&result, ring);
                    if first {
                        first = false;
                        if skip_current {
                            for c in &new_challenges {
                                mark_solved(&c.slug);
                            }
                            println!(
                                "--skip-current: marked {} current challenge(s) as seen without solving.",
                                new_challenges.len()
                            );
                            continue;
                        }
                    }
                    if no_solve {
                        continue;
                    }
                    for c in new_challenges {
                        if let Err(e) = solve(&c.slug, Some(&c), model) {
                            eprintln!("solve {} failed: {e}", c.slug);
                        }
                    }
                }
                Err(e) => eprintln!("bad poll line: {e}: {line}"),
            }
        }

        let status = child.wait().map_err(|e| format!("wait: {e}"))?;
        eprintln!("fetch.js exited with {status}, restarting in 5s...");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

fn solve(slug: &str, challenge: Option<&Challenge>, model: &str) -> Result<(), String> {
    let root = repo_root();
    let dir = root.join(CHALLENGES_DIR).join(slug);
    fs::create_dir_all(&dir).map_err(|e| format!("mkdir: {e}"))?;

    let text = match challenge {
        Some(c) => {
            let stmt = if !c.body_html.is_empty() {
                &c.body_html
            } else if !c.problem_statement.is_empty() {
                &c.problem_statement
            } else {
                ""
            };
            format!(
                "Challenge: {}\nURL: {}\n\nPROBLEM STATEMENT:\n{}\n\nINPUT FORMAT:\n{}\n\nOUTPUT FORMAT:\n{}\n\nCONSTRAINTS:\n{}",
                if c.name.is_empty() { slug } else { &c.name },
                c.url,
                stmt,
                c.input_format,
                c.output_format,
                c.constraints
            )
        }
        None => format!("Challenge slug: {slug}"),
    };

    let prompt = format!(
        "Solve the following HackerRank contest challenge in Rust.\n\n{text}\n\n\
Tasks:\n1. Write a complete Rust program to {dir}/main.rs. It must read from standard input and write to standard output. Use `use std::io::{{self, BufRead}};` style, no external crates.\n\
2. Create a test script {dir}/tests.sh that (a) compiles with `rustc -O main.rs -o main`, (b) runs the compiled binary against every Sample Input from the problem statement, and (c) compares with the expected Sample Output. The script must exit 0 only if ALL samples pass.\n\
3. Run the tests and report PASS/FAIL for each sample.\n\
Do not modify anything outside {dir}.",
        dir = dir.display()
    );

    println!("Solving {slug} with {model} ...");

    let out = Command::new("opencode2")
        .arg("run")
        .arg("-m")
        .arg(model)
        .arg("--auto")
        .arg("--format")
        .arg("json")
        .arg("--title")
        .arg(format!("solve {slug}"))
        .arg(&prompt)
        .current_dir(&dir)
        .output()
        .map_err(|e| format!("failed to run opencode2: {e}"))?;

    if !out.status.success() {
        eprintln!("opencode2 stderr:\n{}", String::from_utf8_lossy(&out.stderr));
        return Err(format!("opencode2 exited with {}", out.status));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    println!("{}", truncate(&stdout, 4000));

    let main_rs = dir.join("main.rs");
    if !main_rs.exists() {
        return Err("main.rs was not produced".into());
    }

    let test = Command::new("bash")
        .arg("tests.sh")
        .current_dir(&dir)
        .output()
        .map_err(|e| format!("failed to run tests.sh: {e}"))?;

    println!("tests.sh exit: {}", test.status);
    println!("{}", truncate(&String::from_utf8_lossy(&test.stdout), 2000));
    if !test.stdout.is_empty() {
        eprintln!("{}", truncate(&String::from_utf8_lossy(&test.stderr), 2000));
    }

    mark_solved(slug);
    if test.status.success() {
        println!("OK: {slug} all tests passed.");
    } else {
        println!("FAIL: {slug} tests failed.");
    }
    Ok(())
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let t: String = s.chars().take(n).collect();
        format!("{t}\n...[truncated]")
    }
}

fn usage() {
    println!(
        "hkwatch — HackerRank challenge watcher\n\n\
Usage:\n  hkwatch check [--headless] [--contest <slug>] [--no-ring]\n  hkwatch watch [--headless] [--contest <slug>] [--interval <secs>] [--no-solve] [--skip-current] [--no-ring]\n  hkwatch solve <slug> [--headless] [--contest <slug>]\n  hkwatch status\n\n\
Flags:\n  --no-solve       report new challenges (with ring) but never auto-solve\n  --skip-current   mark challenges already listed on first poll as seen, solve only future new ones\n  --no-ring        disable the ring sound (also HKWATCH_RING=0)\n\n\
Default contest: {DEFAULT_CONTEST}\nDefault interval: {DEFAULT_INTERVAL_SECS}s\nSolver model: {SOLVE_MODEL}\nOverride solver: HKWATCH_MODEL env or --model <provider/model>"
    );
}

fn main() {
    load_env_file();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
        std::process::exit(1);
    }

    let mut contest = DEFAULT_CONTEST.to_string();
    let mut headless = false;
    let mut interval = DEFAULT_INTERVAL_SECS;
    let mut model = std::env::var("HKWATCH_MODEL").unwrap_or_else(|_| SOLVE_MODEL.to_string());
    let mut no_solve = false;
    let mut skip_current = false;
    let mut ring = std::env::var("HKWATCH_RING").map(|v| v != "0").unwrap_or(true);

    let mut iter = args.iter();
    let cmd = iter.next().unwrap();
    let cmd: &str = cmd;
    let mut rest: Vec<String> = Vec::new();
    while let Some(a) = iter.next() {
        match a.as_str() {
            "--headless" => headless = true,
            "--no-solve" => no_solve = true,
            "--skip-current" => skip_current = true,
            "--no-ring" => ring = false,
            "--model" => {
                model = iter
                    .next()
                    .expect("--model requires a value")
                    .clone()
            }
            "--contest" => {
                contest = iter
                    .next()
                    .expect("--contest requires a value")
                    .clone()
            }
            "--interval" => {
                interval = iter
                    .next()
                    .expect("--interval requires a value")
                    .parse()
                    .expect("--interval must be a number")
            }
            other => rest.push(other.to_string()),
        }
    }

    match cmd {
        "check" => {
            let _ = fs::create_dir_all(CHALLENGES_DIR);
            match check(&contest, headless, ring) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("check failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        "watch" => {
            if let Err(e) = watch_persistent(&contest, headless, interval, &model, no_solve, skip_current, ring)
            {
                eprintln!("watch failed: {e}");
                std::process::exit(1);
            }
        }
        "solve" => {
            let slug = rest
                .first()
                .map(|s| s.clone())
                .unwrap_or_else(|| {
                    eprintln!("solve requires a <slug>");
                    std::process::exit(1);
                });
            let ch = load_last_fetch().and_then(|f| {
                f.challenges
                    .into_iter()
                    .find(|c| c.slug == slug)
            });
            match solve(&slug, ch.as_ref(), &model) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("solve failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        "status" => {
            let seen = load_seen();
            println!("Seen challenges ({}):", seen.slugs.len());
            for s in &seen.slugs {
                println!("  {s}");
            }
        }
        _ => {
            eprintln!("unknown command: {cmd}");
            usage();
            std::process::exit(1);
        }
    }
}
