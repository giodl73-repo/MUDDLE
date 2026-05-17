# Pulse 04 - Destination and Host Mounting

## Status

Active.

## Goal

Set MUDDLE's destination: one shared play surface that can launch and play any
supported BANISH game or AMAZE escape room through host adapters.

## Work

- Define the destination acceptance target in the README and product plan.
- Make host selection the next CLI capability after the labyrinth mock sim.
- Name BANISH and AMAZE adapter spikes as required before richer rendering.
- Keep product rules in host repos and shared UX/session contracts in MUDDLE.

## Decision

MUDDLE should become the common play shell. BANISH and AMAZE should provide
explicit adapters that implement `MuddleHost`; `muddle-cli` and future richer
renderers should select adapters and drive the same session loop.

## Validation

```powershell
cargo test --quiet
cargo run -p muddle-cli
```
