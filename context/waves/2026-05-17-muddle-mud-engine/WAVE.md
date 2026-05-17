# MUDDLE MUD Engine Wave

## Status

Active.

## Goal

Create MUDDLE as a shared text/ASCII interaction engine for portfolio games.

## Scope

- Create a standalone Cargo workspace.
- Add `muddle-core` with room, exit, command, session, and transcript types.
- Add deterministic ASCII room-card rendering.
- Record product boundaries against RALLY, BANISH, AMAZE, and board-game repos.

## Non-goals

- No networking/MMO implementation.
- No BANISH, AMAZE, or board-game rules in core.
- No graphical rendering.
- No RALLY runtime dependency until an adapter is proven.

## Pulses

| Pulse | Status | Purpose |
|---|---|---|
| [Pulse 01 - Workspace and core rooms](pulses/pulse-01.md) | Active | Create the workspace and first product-neutral room/session model. |

## Validation

```powershell
cargo test --quiet
```

