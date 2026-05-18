# MUDDLE Local Window Client

Status: first local browser-backed window surface.

The window client is intentionally thin: it starts a local HTTP server, opens a
browser window when requested, and drives the same `MuddleHost` +
`MuddleSession` contracts used by `muddle-cli`. It is not a new engine and does
not own product rules.

## Run

```powershell
cargo run -p muddle-window -- --open
cargo run -p muddle-window -- --host portfolio-showcase --open
cargo run -p muddle-window -- --host mock-labyrinth --open
cargo run -p muddle-window -- --host banish-pilgrim-loss --open
cargo run -p muddle-window -- --host amaze-silverstream --open
```

If `--open` is omitted, open the printed local URL manually.

```powershell
cargo run -p muddle-window -- --addr 127.0.0.1:4777
cargo run -p muddle-window -- --host banish-pilgrim-loss --save pilgrim-loss.window.muddle --transcript pilgrim-loss.window.txt --open
cargo run -p muddle-window -- --host banish-pilgrim-loss --load pilgrim-loss.window.muddle --save pilgrim-loss.window.muddle --transcript pilgrim-loss.window.txt --open
```

The front screen lists every mounted host. Choosing a host starts a fresh local
session for that host; the in-window **Change host** button returns to the
chooser. The default `portfolio-showcase` host is a browsable catalog for
MUDDLE-backed and adjacent systems currently visible in the portfolio: games,
Knowledge Systems, Design Labs, and infrastructure.

Product repos can depend on the `muddle-window` crate and call
`run_muddle_window_hosts_from_env_args()` with their own `MuddleHost`
registrations. This mirrors the existing reusable `muddle-cli` runner shape.
The runner accepts `--save`, `--load`, and `--transcript`; saves use the same
command-replay format as the CLI and include optional host-owned checkpoints
when the mounted host implements checkpoint export/import. Transcripts use the
same transcript renderer.
Host-provided command hints render as clickable action buttons, so every
compatible host can expose its next legal/common commands without custom window
code.
The browser view also renders the full turn history and links to `/transcript`,
which returns the same text transcript format used by `muddle-cli`.
The **Save now** button writes the configured `--save` and/or `--transcript`
paths immediately. The **Reload save** button reloads the configured `--save`
path without restarting the local server.
The **Restart host** button resets the currently selected host to a fresh
session while preserving any `--save` and `--transcript` output paths.

## Boundary

MUDDLE owns the local renderer, command entry, room panel display, transcript
count, turn history, local transcript endpoint, persistence controls, and
host/session handoff. BANISH, AMAZE, and future products still own their content,
rules, maps, puzzles, simulations, and win conditions.

## Validation

```powershell
cargo test --quiet
cargo run -p muddle-window -- --list-hosts
cargo run -p muddle-window -- --addr 127.0.0.1:4777
git grep -n "muddle-window" -- README.md PRODUCT_PLAN.md docs\window-client.md Cargo.toml
```
