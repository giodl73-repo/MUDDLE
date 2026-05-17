# Pulse 06 - AMAZE Adapter Spike

## Status

Active.

## Goal

Mount an AMAZE Silverstream escape-room-shaped host through MUDDLE so the shared
CLI can play a clue/lock/escape path by adapter name.

## Work

- Add `muddle-amaze-spike` with `AmazeSilverstreamHost` implementing
  `MuddleHost`.
- Register `amaze-silverstream` in `muddle-cli`.
- Cover clue discovery, signal alignment, hatch unlocking, exit movement,
  resources, objectives, command hints, and map panels with tests.
- Keep this spike in MUDDLE only until AMAZE exposes a reusable library adapter
  crate.

## Decision

AMAZE has an escape-room harness and room contracts, but the MUDDLE adapter
boundary should be proven through the shared host interface first. The spike
models a Silverstream-style room with a hidden clue, signal lock, hatch, hints,
and resettable escape path.

## Validation

```powershell
cargo test --quiet
cargo run -p muddle-cli -- --host amaze-silverstream
```
