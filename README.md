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

## HackerRank login

The contest may be login-gated, so the watcher can authenticate before
fetching. Credentials are loaded from `.hkwatch.env` at the repo root (or from
the `HKWATCH_USERNAME` / `HKWATCH_PASSWORD` env vars):

```sh
HKWATCH_USERNAME=UC193_TerraForge
HKWATCH_PASSWORD=#Terra_Forge@12345
```

When both vars are present, `hkwatch` automatically passes `--login` to
`fetch.js`, which signs into HackerRank with the stealth browser before reading
the challenge list. Keep `.hkwatch.env` private — it contains credentials.

## Usage

Run from the repo root.

```sh
# One poll: report new challenges
./watcher/target/release/hkwatch check

# Watch loop: polls every 15s, auto-solves each new challenge.
# The stealth browser is launched ONCE and the page is reloaded in place —
# the browser is never closed between polls, so the login session persists.
./watcher/target/release/hkwatch watch

# Watch with custom interval
./watcher/target/release/hkwatch watch --interval 30

# Solve a specific challenge (statement looked up from last fetch)
./watcher/target/release/hkwatch solve <challenge-slug>

# List challenges seen so far
./watcher/target/release/hkwatch status
```

### Options

| Flag | Meaning |
| --- | --- |
| `--contest <slug>` | Contest slug (default `jpuraxtreme-3-0-inter-univeristy-section`) |
| `--headless` | Run the stealth browser headless |
| `--interval <secs>` | Poll interval for `watch` (default `15`) |
| `--model <provider/model>` | Solver model (default `opencode-go/deepseek-v4-flash`); also `HKWATCH_MODEL` env var |

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
