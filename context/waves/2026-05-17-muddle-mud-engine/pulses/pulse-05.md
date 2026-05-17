# Pulse 05 - BANISH Adapter Spike

## Status

Active.

## Goal

Mount a BANISH Pilgrim Loss launcher-shaped host through MUDDLE so the shared CLI
can play a real product-style path by adapter name.

## Work

- Add `muddle-banish-spike` with a `BanishPilgrimLossHost` implementing
  `MuddleHost`.
- Register `banish-pilgrim-loss` in `muddle-cli`.
- Cover launcher choice, campaign brief inspection, manifest readiness, trail
  entry, and visible-loss resolution with tests.
- Add the first shared game-screen panels: host-provided resource/status counts
  and ASCII maps rendered by the CLI.
- Add objective and contextual command panels so players can see goals and valid
  verbs without guessing.
- Keep this spike in MUDDLE only until BANISH exposes a reusable library adapter
  crate.

## Decision

BANISH is currently binary-only, so MUDDLE cannot link clean BANISH APIs yet. The
spike mirrors the Pilgrim Loss launcher surface and proves the MUDDLE mounting
shape first; the follow-up is to move the adapter to BANISH once BANISH exposes a
library boundary.

## Validation

```powershell
cargo test --quiet
cargo run -p muddle-cli -- --host banish-pilgrim-loss
```
