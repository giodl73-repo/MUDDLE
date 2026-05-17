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
| [Pulse 01 - Workspace and core rooms](pulses/pulse-01.md) | Done | Create the workspace and first product-neutral room/session model. |
| [Pulse 02 - Role contracts and plan review](pulses/pulse-02.md) | Done | Define MUDDLE delivery roles and sharpen the next adapter-first plan. |
| [Pulse 03 - Adapter seam and CLI](pulses/pulse-03.md) | Active | Prove the host adapter seam with a static fixture and first CLI renderer. |

## Validation

```powershell
cargo test --quiet
```

