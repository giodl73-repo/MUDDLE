# Pulse 01 - Workspace and Core Rooms

## Status

Done.

## Goal

Create the MUDDLE workspace and first product-neutral room/session model.

## Work

- Create a standalone Cargo workspace at the repo root.
- Add `crates/muddle-core`.
- Define rooms, exits, commands, turns, and sessions.
- Add deterministic ASCII room-card rendering.
- Add tests for command parsing, card rendering, and transcript recording.

## Decision

MUDDLE starts as shared infrastructure rather than part of RALLY. RALLY remains
the simulation/evaluation substrate; MUDDLE owns text/ASCII interaction UX.

## Validation

```powershell
cargo test --quiet
```

