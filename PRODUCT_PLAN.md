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

## Role model

MUDDLE uses `.roles/` to keep responsibilities explicit:

| Role | Plan responsibility |
|---|---|
| Product steward | Protect the MUDDLE/RALLY/product boundary and wave scope. |
| Core engineer | Build deterministic product-neutral contracts in `muddle-core`. |
| Host adapter engineer | Define small mount contracts for BANISH, AMAZE, and board-game hosts. |
| Playtest designer | Keep command vocabulary, ASCII views, and transcripts player-readable. |
| Validation gatekeeper | Require deterministic tests, fixtures, and workspace isolation checks. |

## Current state

`muddle-core` currently supports:

- `MuddleRoom`
- `MuddleExit`
- `MuddleCommand`
- `MuddleTurn`
- `MuddleSession`
- ASCII room cards
- transcript recording

## Plan review

The current plan is correctly scoped for a shared UX engine: it starts with
local rooms, commands, deterministic ASCII output, and transcripts before
adapters or networking. The main gap is that the next pulse should prove a host
adapter seam with a tiny fixture before adding more renderer features. That
keeps MUDDLE from becoming BANISH-specific and gives AMAZE/board-game hosts the
same contract.

Recommended next sequence:

1. Finish role contracts and mark the workspace/core-room pulse complete.
2. Add a minimal host adapter trait and one in-repo fixture host.
3. Add transcript replay/save-resume fixtures against that adapter.
4. Only then expand ASCII maps beyond room cards.

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
