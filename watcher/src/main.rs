use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
#[cfg(unix)]
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

static SOLVE_ON: AtomicBool = AtomicBool::new(true);

const DEFAULT_CONTEST: &str = "jpuraxtreme-3-0-inter-univeristy-section";
const DEFAULT_INTERVAL_SECS: u64 = 15;
const FETCH_SCRIPT: &str = "cloak/fetch.js";
const CHALLENGES_DIR: &str = "challenges";
const SEEN_FILE: &str = "challenges/.seen.json";
const LAST_FETCH_FILE: &str = "challenges/.last_fetch.json";
const SOLVE_MODEL: &str = "opencode-go/deepseek-v4-flash";
const VISION_MODEL: &str = "opencode-go/qwen3.7-plus";
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
    #[serde(default)]
    images: Vec<String>,
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
    std::env::current_dir().expect("cannot read cwd")
}

fn fetch_script_path() -> PathBuf {
    if let Ok(p) = std::env::var("HKWATCH_FETCH") {
        return PathBuf::from(p);
    }
    let cwd = repo_root();
    let local = cwd.join(FETCH_SCRIPT);
    if local.exists() {
        return local;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for cand in [
                dir.join(FETCH_SCRIPT),
                dir.join("..").join(FETCH_SCRIPT),
                dir.join("..").join("..").join(FETCH_SCRIPT),
            ] {
                if cand.exists() {
                    return cand;
                }
            }
        }
    }
    local
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
    let mut cmd = Command::new("node");
    cmd.arg(fetch_script_path());
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

#[cfg(unix)]
fn setup_solve_toggle() {
    use signal_hook::consts::signal::SIGUSR1;
    use signal_hook::flag;
    let received = Arc::new(AtomicBool::new(false));
    if let Err(e) = flag::register(SIGUSR1, Arc::clone(&received)) {
        eprintln!("warning: cannot register SIGUSR1 toggle: {e}");
        return;
    }
    std::thread::spawn(move || loop {
        if received.swap(false, Ordering::SeqCst) {
            let prev = SOLVE_ON.fetch_xor(true, Ordering::SeqCst);
            println!("[toggle] solve mode -> {}", if prev { "OFF" } else { "ON" });
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    });
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

fn no_solve_active() -> bool {
    !SOLVE_ON.load(Ordering::SeqCst)
}

fn watch_persistent(
    contest: &str,
    headless: bool,
    interval: u64,
    model: &str,
    vision_model: &str,
    use_vision: bool,
    skip_current: bool,
    ring: bool,
) -> Result<(), String> {
    let mut cmd = Command::new("node");
    cmd.arg(fetch_script_path());
    cmd.arg(contest)
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
    #[cfg(unix)]
    println!(
        "Solve mode: {}. Toggle at runtime WITHOUT stopping: kill -USR1 {} (or SIGUSR1)",
        if SOLVE_ON.load(Ordering::SeqCst) { "ON" } else { "OFF" },
        std::process::id()
    );
    #[cfg(not(unix))]
    println!(
        "Solve mode: {}. Restart the watcher with --no-solve to disable solving.",
        if SOLVE_ON.load(Ordering::SeqCst) { "ON" } else { "OFF" }
    );

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
                    if no_solve_active() {
                        for c in &new_challenges {
                            mark_solved(&c.slug);
                            println!("SKIP: [{}] solve mode OFF", c.slug);
                        }
                        continue;
                    }
                    for c in new_challenges {
                        if let Err(e) = solve(&c.slug, Some(&c), model, vision_model, use_vision) {
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

fn vision_transcribe(image: &PathBuf, vision_model: &str) -> Result<String, String> {
    let prompt = "You are an image transcription assistant for a competitive programming problem. \
Examine the image carefully and transcribe everything that matters for solving the problem. \
If it is a grid or diagram, output every row exactly, cell by cell, using a consistent symbol set and the same number of columns per row. \
If it contains text or data, reproduce it verbatim. Be precise; do not summarize.";
    let out = Command::new("opencode2")
        .arg("run")
        .arg("-m")
        .arg(vision_model)
        .arg("--auto")
        .arg("--format")
        .arg("json")
        .arg("-f")
        .arg(image)
        .arg("--title")
        .arg("vision transcription")
        .arg(prompt)
        .output()
        .map_err(|e| format!("failed to run vision model: {e}"))?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut texts: Vec<String> = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if !line.starts_with('{') {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if v["type"] == "text" {
                if let Some(t) = v["part"]["text"].as_str() {
                    texts.push(t.to_string());
                }
            }
        }
    }
    let joined = texts.join("\n");
    if joined.trim().is_empty() {
        Err("vision model returned no transcription".into())
    } else {
        Ok(joined)
    }
}

fn solve(
    slug: &str,
    challenge: Option<&Challenge>,
    model: &str,
    vision_model: &str,
    use_vision: bool,
) -> Result<(), String> {
    let root = repo_root();
    let dir = root.join(CHALLENGES_DIR).join(slug);
    fs::create_dir_all(&dir).map_err(|e| format!("mkdir: {e}"))?;

    let mut text = match challenge {
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

    if use_vision {
        let images: Vec<String> = challenge
            .map(|c| c.images.clone())
            .unwrap_or_default();
        if !images.is_empty() {
            println!(
                "{} statement image(s) found — transcribing with {vision_model} ...",
                images.len()
            );
            for img in &images {
                let p = root.join(img);
                if !p.exists() {
                    eprintln!("image not found: {}", p.display());
                    continue;
                }
                match vision_transcribe(&p, vision_model) {
                    Ok(t) => text.push_str(&format!(
                        "\n\nIMAGE TRANSCRIPTION ({img}):\n{t}"
                    )),
                    Err(e) => eprintln!("vision transcription failed for {img}: {e}"),
                }
            }
        }
    }

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
Usage:\n  hkwatch check [--headless] [--contest <slug>] [--no-ring]\n  hkwatch watch [--headless] [--contest <slug>] [--interval <secs>] [--no-solve] [--skip-current] [--no-ring] [--no-vision] [--vision-model <provider/model>]\n  hkwatch solve <slug> [--headless] [--contest <slug>] [--no-vision] [--vision-model <provider/model>]\n  hkwatch status\n\n\
Flags:\n  --no-solve       start with solve mode OFF (report + ring, never auto-solve); watching continues\n  --skip-current   mark challenges already listed on first poll as seen, solve only future new ones\n  --no-ring        disable the ring sound (also HKWATCH_RING=0)\n  --no-vision      skip transcribing statement images with the vision model\n  --vision-model   vision model for statement images (default {VISION_MODEL}; also HKWATCH_VISION_MODEL env)\n\n\
Runtime:\n  During `watch`, send SIGUSR1 (kill -USR1 <pid>) to toggle solve mode ON/OFF without stopping the watcher (Unix only; Windows: restart with --no-solve).\n\n\
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
    let mut skip_current = false;
    let mut ring = std::env::var("HKWATCH_RING").map(|v| v != "0").unwrap_or(true);
    let mut vision_model = std::env::var("HKWATCH_VISION_MODEL")
        .unwrap_or_else(|_| VISION_MODEL.to_string());
    let mut use_vision = true;

    let mut iter = args.iter();
    let cmd = iter.next().unwrap();
    let cmd: &str = cmd;
    let mut rest: Vec<String> = Vec::new();
    while let Some(a) = iter.next() {
        match a.as_str() {
            "--headless" => headless = true,
            "--no-solve" => SOLVE_ON.store(false, Ordering::SeqCst),
            "--skip-current" => skip_current = true,
            "--no-ring" => ring = false,
            "--no-vision" => use_vision = false,
            "--vision-model" => {
                vision_model = iter
                    .next()
                    .expect("--vision-model requires a value")
                    .clone()
            }
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
            #[cfg(unix)]
            setup_solve_toggle();
            if let Err(e) = watch_persistent(
                &contest,
                headless,
                interval,
                &model,
                &vision_model,
                use_vision,
                skip_current,
                ring,
            ) {
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
            match solve(&slug, ch.as_ref(), &model, &vision_model, use_vision) {
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
