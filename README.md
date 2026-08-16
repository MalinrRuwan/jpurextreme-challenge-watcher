# hkwatch — HackerRank Challenge Watcher

Watches a HackerRank contest page for newly posted challenges, and for each new
challenge: creates `challenges/<slug>/`, has `opencode2` (deepseek-v4-flash)
write a Rust solution, and runs its sample tests.

## How it works

1. **`cloak/fetch.js`** — Playwright + `puppeteer-extra-plugin-stealth` (bypasses
   HackerRank's Akamai "Access Denied" bot block). Loads the contest's
   challenges page, then pulls the challenge list and full statements via
   HackerRank's internal REST API from inside the browser session (so any
   login cookies apply).
2. **`watcher/`** — a Rust CLI (`hkwatch`) that polls, diffs, and dispatches.
3. **`challenges/`** — one folder per challenge: `main.rs`, `tests.sh`, and the
   compiled `main` binary.

## Setup

```sh
cd cloak && npm install && npx playwright install chromium
cd ../watcher && cargo build --release
```

## Cross-platform setup (automatic)

Ready-made scripts install everything (Node, Rust, Playwright Chromium,
opencode2) and build or download the `hkwatch` binary:

```sh
# macOS / Linux / WSL / Git Bash
bash scripts/setup.sh

# Windows (PowerShell)
powershell -ExecutionPolicy Bypass -File scripts/setup.ps1
```

Each script ends with the same two next steps: edit `.hkwatch.env` with your
HackerRank credentials, then run `bin/hkwatch watch --headless --skip-current`.

## Release binaries

Prebuilt `hkwatch` binaries for all platforms are attached to every
[GitHub release](https://github.com/MalinrRuwan/jpurextreme-challenge-watcher/releases):

| File | Platform |
| --- | --- |
| `hkwatch-aarch64-macos` | macOS Apple Silicon |
| `hkwatch-x86_64-macos` | macOS Intel |
| `hkwatch-x86_64-linux` | Linux x86_64 |
| `hkwatch-x86_64-windows.exe` | Windows x86_64 |

The binary needs the repo layout next to it (it locates `cloak/fetch.js`
relative to its own path), plus `node` + Playwright + opencode2 — install those
with `scripts/setup.sh` / `scripts/setup.ps1`. New tags (`v*`) trigger
`.github/workflows/release.yml`, which builds all four binaries on GitHub
Actions and attaches them automatically.

## HackerRank login

The contest may be login-gated, so the watcher can authenticate before
fetching. Credentials are loaded from `.hkwatch.env` at the repo root (or from
the `HKWATCH_USERNAME` / `HKWATCH_PASSWORD` env vars):

```sh
HKWATCH_USERNAME=your_hackerrank_username
HKWATCH_PASSWORD=your_hackerrank_password
```

When both vars are present, `hkwatch` automatically passes `--login` to
`fetch.js`, which signs into HackerRank with the stealth browser before reading
the challenge list. Keep `.hkwatch.env` private — it contains credentials.
A template lives at `.hkwatch.env.example`.

## Usage

Run from the repo root.

```sh
# One poll: report new challenges
./bin/hkwatch check

# Watch loop: polls every 15s, auto-solves each new challenge.
# The stealth browser is launched ONCE and the page is reloaded in place —
# the browser is never closed between polls, so the login session persists.
./bin/hkwatch watch

# Watch with custom interval
./bin/hkwatch watch --interval 30

# Solve a specific challenge (statement looked up from last fetch)
./bin/hkwatch solve <challenge-slug>

# List challenges seen so far
./bin/hkwatch status
```

If you built from source instead of using `scripts/setup.sh`, the binary lives
at `watcher/target/release/hkwatch` — substitute that path.

### Options

| Flag | Meaning |
| --- | --- |
| `--contest <slug>` | Contest slug (default `jpuraxtreme-3-0-inter-univeristy-section`) |
| `--headless` | Run the stealth browser headless |
| `--interval <secs>` | Poll interval for `watch` (default `15`) |
| `--model <provider/model>` | Solver model (default `opencode-go/deepseek-v4-flash`); also `HKWATCH_MODEL` env var |
| `--no-solve` | Start with solve mode OFF: new challenges are reported + rung but never auto-solved; watching continues |
| `--no-ring` | Disable the ring sound (also `HKWATCH_RING=0`) |

### Runtime solve toggle (no restart)

While `watch` is running, send `SIGUSR1` to toggle auto-solving on/off. The
watcher keeps running and polling the whole time — only the solve step is
enabled/disabled.

```sh
hkwatch watch   # prints "kill -USR1 <pid>" on startup
kill -USR1 <pid>   # toggle solve mode OFF -> reports new challenges, doesn't solve
kill -USR1 <pid>   # toggle solve mode back ON
```

While OFF, detected challenges are marked as seen (so they aren't re-rung on
every poll) but not solved. To solve one later, run `hkwatch solve <slug>` or
remove its slug from `challenges/.seen.json` and let the watcher pick it up.

### Command combinations

Practical one-liners (run from the repo root, add `--headless` wherever you
don't want a visible browser window):

```sh
# Full auto: watch headless, solve every new challenge, ring on discovery
./watcher/target/release/hkwatch watch --headless

# Report-only: ring on new challenges but never auto-solve (watching continues)
./watcher/target/release/hkwatch watch --headless --no-solve

# Skip challenges already posted: only solve genuinely new ones that appear
./watcher/target/release/hkwatch watch --headless --skip-current

# Custom polling interval (faster or slower than the default 15s)
./watcher/target/release/hkwatch watch --headless --interval 5
./watcher/target/release/hkwatch watch --headless --interval 60

# Quiet report-only: no ring sound, headless, never solve
HKWATCH_RING=0 ./watcher/target/release/hkwatch watch --headless --no-solve

# One-shot check and manual control
./watcher/target/release/hkwatch check --headless
./watcher/target/release/hkwatch solve <challenge-slug>
./watcher/target/release/hkwatch status

# Use a different solver model
./watcher/target/release/hkwatch watch --headless --model opencode/deepseek-v4-flash-free

# Force a re-solve: drop the slug from .seen.json, then watch
# (or solve it directly from the last fetched statement)
./watcher/target/release/hkwatch solve <challenge-slug>
```

And the always-on recipe used in production:

```sh
./watcher/target/release/hkwatch watch --headless --skip-current
# toggle solving off/on mid-run without stopping:
kill -USR1 $(pgrep -f "hkwatch watch")
```

## Solve flow

For each new challenge slug, `hkwatch solve`:

1. Creates `challenges/<slug>/`.
2. Builds a prompt from the fetched problem statement (`body_html`,
   input/output format, constraints) and the absolute folder path.
3. Runs
   `opencode2 run -m <model> --auto --format json "<prompt>"` with cwd set to
   the challenge folder. The model writes `main.rs` and a `tests.sh` that
   compiles and checks every Sample Input/Output.
4. Runs `bash tests.sh` and reports `OK`/`FAIL`; marks the slug as seen in
   `challenges/.seen.json` regardless, so the watcher won't re-solve it.

## Notes

- `challenges/.seen.json` is the source of truth for "already seen" slugs;
  delete a slug from it to force a re-solve.
- The default solver `opencode-go/deepseek-v4-flash` was chosen because the
  requested `opencode/deepseek-v4-flash` is out of balance on the current
  opencode workspace (HTTP 401 Insufficient balance).
- `fetch.js` in `--watch` mode keeps one browser session alive: it reloads the
  contest page every `<interval>` seconds and prints one JSON line per poll to
  stdout. `hkwatch watch` reads those lines, diffs against `.seen.json`, and
  solves anything new. If the fetcher crashes it is restarted after 5s.
- In one-shot mode (`hkwatch check`), a fresh browser is launched per call.
