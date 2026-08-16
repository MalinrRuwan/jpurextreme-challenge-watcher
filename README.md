# hkwatch — HackerRank Challenge Watcher

Watches a HackerRank contest page for newly posted challenges, and for each new
challenge: creates `challenges/<slug>/`, has `opencode2` (deepseek-v4-flash)
write a Rust solution, and runs its sample tests.

## How it works

1. **`cloak/fetch.js`** — Playwright + `puppeteer-extra-plugin-stealth` (bypasses
   HackerRank's Akamai "Access Denied" bot block). Loads the contest's
   challenges page, then pulls the challenge list and full statements via
   HackerRank's internal REST API from inside the browser session (so any
   login cookies apply). It also downloads any `<img>` embedded in a statement
   to `challenges/<slug>/statement_<n>.<ext>`.
2. **`watcher/`** — a Rust CLI (`hkwatch`) that polls, diffs, and dispatches.
3. **`challenges/`** — runtime-only, one folder per solved challenge
   (`main.rs`, `tests.sh`). Generated on disk as challenges are fetched/solved;
   it is **gitignored** and never part of the source tree. Deleting it is safe —
   the watcher recreates it.
4. **`state/`** — runtime-only bookkeeping: `.seen.json` (challenges already
   handled) and `.last_fetch.json` (last fetched payload). Gitignored like
   `challenges/`.

Already-solved detection: each poll includes the site's `solved` flag per
challenge. Challenges the account has already solved on HackerRank are marked
seen **without** being solved again, so the watcher never re-solves
submissions that already have a score.

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
| `--parallel <N>` | Max simultaneous `opencode2` solves when several new challenges are detected (default `2`; also `HKWATCH_PARALLEL`) |
| `--show-lb` | Print the two-column team leaderboard on every poll (also `HKWATCH_SHOW_LB=1`) |

When multiple unsolved challenges are found in one poll, `hkwatch watch` runs up
to `--parallel N` solves concurrently (each in its own thread + `opencode2`
process). `.seen.json` writes are mutex-guarded so parallel completions can't
corrupt state, and each challenge's report prints atomically.

### Split-screen TUI

Running `watch` in a real terminal automatically enables the split-screen UI
(`--tui` forces it on, `--no-tui` forces it off, e.g. when piping output):

```
┌LOGS (123 lines)─────────────────────┐┌LEADERBOARD (67)──────────────┐
│Watching … every 15s                 ││  RK TEAM                 SCORE│
│NEW: [the-great-crypto-heist] …      ││   1 UC97_SJNovaris        240│
│Solving … with …                     ││   2 UC94_Novatrix         240│
│OK: … all tests passed.              ││   …                        …│
└─────────────────────────────────────┘└──────────────────────────────┘
 solve: ON  |  q quit   ↑/↓ scroll   s toggle solve
```

- **Left column**: live scrolling log of polls, detections, solves, and results.
- **Right column**: the team leaderboard, updated on every poll, ranked by score
  descending then time ascending (lowest time wins).
- Keys: `q` quit, `↑`/`↓` scroll the log, `s` toggle solve mode (same as
  `SIGUSR1` on Unix).

When stdout isn't a terminal (piped/redirected), `watch` falls back to plain
text logs instead.

### Leaderboard

`hkwatch leaderboard` fetches the contest leaderboard through the stealth
browser session and prints it in a two-column layout (rank/team/score/time).
Ranking is **score descending, then time ascending** (lowest time wins),
independent of the rank HackerRank reports. `--show-lb` (or `HKWATCH_SHOW_LB=1`)
prints the same table between polls in plain watch mode.

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

# Team leaderboard: two-column, ranked by score desc then time asc (lowest time wins)
./watcher/target/release/hkwatch leaderboard --headless

# Watch + print the leaderboard on every poll
./watcher/target/release/hkwatch watch --headless --show-lb

# Use a different solver model
./watcher/target/release/hkwatch watch --headless --model opencode/deepseek-v4-flash-free

# Solve several new challenges concurrently (default parallel=2)
./watcher/target/release/hkwatch watch --headless --parallel 4
HKWATCH_PARALLEL=4 ./watcher/target/release/hkwatch watch --headless

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

### Statement images → vision model

Some statements embed a diagram/grid as an image (e.g. a map). Before solving,
`hkwatch` sends every downloaded `statement_<n>.<ext>` image to a vision model
via `opencode2 run -m <vision-model> -f <image>`, and appends the transcription
to the solver prompt.

```sh
# Default vision model (qwen 3.7 Plus), run whenever a statement has images
./watcher/target/release/hkwatch watch --headless

# Skip image transcription entirely
./watcher/target/release/hkwatch solve <slug> --no-vision

# Use a different vision model
./watcher/target/release/hkwatch watch --headless --vision-model opencode-go/qwen3.7-max
HKWATCH_VISION_MODEL=opencode-go/qwen3.7-max ./watcher/target/release/hkwatch watch --headless
```

| Flag / env | Meaning |
| --- | --- |
| `--no-vision` | Don't transcribe statement images |
| `--vision-model <provider/model>` | Vision model (default `opencode-go/qwen3.7-plus`; `HKWATCH_VISION_MODEL` env) |

Note: transcription is only as good as the vision model. If a problem's diagram
is misread, solve with `--no-vision` and check the statement manually.

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

- `state/.seen.json` is the local source of truth for "already seen" slugs;
  delete a slug from it to force a re-solve. The `challenges/` and `state/`
  folders are runtime state and are gitignored.
- The default solver `opencode-go/deepseek-v4-flash` was chosen because the
  requested `opencode/deepseek-v4-flash` is out of balance on the current
  opencode workspace (HTTP 401 Insufficient balance).
- `fetch.js` in `--watch` mode keeps one browser session alive: it reloads the
  contest page every `<interval>` seconds and prints one JSON line per poll to
  stdout. `hkwatch watch` reads those lines, diffs against `.seen.json`, and
  solves anything new. If the fetcher crashes it is restarted after 5s.
- In one-shot mode (`hkwatch check`), a fresh browser is launched per call.
