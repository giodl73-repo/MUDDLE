# MUDDLE Local Window Client

Status: first local browser-backed window surface.

The window client is intentionally thin: it starts a local HTTP server, opens a
browser window when requested, and drives the same `MuddleHost` +
`MuddleSession` contracts used by `muddle-cli`. It is not a new engine and does
not own product rules.

## Run

```powershell
cargo run -p muddle-window -- --open
cargo run -p muddle-window -- --host mock-labyrinth --open
cargo run -p muddle-window -- --host banish-pilgrim-loss --open
cargo run -p muddle-window -- --host amaze-silverstream --open
```

If `--open` is omitted, open the printed local URL manually.

```powershell
cargo run -p muddle-window -- --addr 127.0.0.1:4777
```

## Boundary

MUDDLE owns the local renderer, command entry, room panel display, transcript
count, and host/session handoff. BANISH, AMAZE, and future products still own
their content, rules, maps, puzzles, simulations, and win conditions.

## Validation

```powershell
cargo test --quiet
cargo run -p muddle-window -- --list-hosts
git grep -n "muddle-window" -- README.md PRODUCT_PLAN.md docs\window-client.md Cargo.toml
```
