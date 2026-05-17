# MUDDLE Product Plan

## Thesis

Portfolio games need one shared play surface before they need full engines.
MUDDLE provides a MUD-like text/ASCII UX layer that can host video-game,
escape-room, and board-game experiences through product-owned adapters.

## Product promise

A player can enter a room, read a stable text/ASCII view, issue commands, move
between rooms, inspect state, and resume from a transcript-backed session.

## Dependency placement

MUDDLE is shared infrastructure. It sits beside RALLY, not inside it:

| System | Responsibility |
|---|---|
| RALLY | Deterministic runs, metrics, comparison, validation, and replay contracts. |
| MUDDLE | Rooms, exits, commands, ASCII views, session state, transcripts, and host adapters. |
| Product repos | Domain rules, content, maps, puzzles, simulations, and player goals. |

## First wave

The first wave proves the product-neutral core:

1. Workspace and core room model.
2. Command parser and transcript session.
3. ASCII room card rendering.
4. Host adapter contract for RALLY/BANISH/AMAZE later.

## Current state

`muddle-core` currently supports:

- `MuddleRoom`
- `MuddleExit`
- `MuddleCommand`
- `MuddleTurn`
- `MuddleSession`
- ASCII room cards
- transcript recording

## Non-goals

- No MMO, sockets, accounts, or real-time multiplayer in the first wave.
- No product-specific game rules in `muddle-core`.
- No renderer beyond deterministic text/ASCII output until host contracts are
  proven.
- No runtime dependency on RALLY until a real adapter proves the boundary.

## Validation

```powershell
cargo test --quiet
```
