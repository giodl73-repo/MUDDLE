# MUDDLE Product Plan

## Thesis

Portfolio games need one shared play surface before they need full engines.
MUDDLE provides a MUD-like text/ASCII UX layer that can host video-game,
escape-room, and board-game experiences through product-owned adapters.

## Product promise

A player can enter a room, read a stable text/ASCII view, issue commands, move
between rooms, inspect state, and resume from a transcript-backed session.

## Destination goal

MUDDLE succeeds when one shared UX can play any supported BANISH game or AMAZE
escape room without becoming either product's engine.

| Destination capability | Acceptance signal |
|---|---|
| BANISH mounting | A BANISH adapter exposes a playable game surface through `MuddleHost`. |
| AMAZE mounting | An AMAZE adapter exposes escape rooms, clues, locks, and puzzle state through `MuddleHost`. |
| Shared renderer | `muddle-cli` can select and play either adapter without host-specific renderer code. |
| Local window client | `muddle-window` can open a browser-backed local window over the same host/session contracts and expose a reusable save/load/transcript-capable runner for product-owned hosts. |
| Portfolio catalog | `muddle-window` can browse systems already visible through MUDDLE before each product has a direct window mount. |
| Game-screen panels | Hosts can provide resource/status counts, objectives, command hints, clickable action buttons, and an ASCII map without custom renderer code. |
| Replay control | Players can restart the current window host without restarting the server or losing configured save/transcript paths. |
| Host checkpoints | Stateful hosts can attach product-owned checkpoint data to shared CLI/window saves without custom renderer logic. |
| Transcript portability | A playthrough transcript records room ids, commands, responses, and host outcomes consistently across BANISH and AMAZE. |
| Product boundary | BANISH/AMAZE rules stay in their repos; MUDDLE owns only shared UX/session contracts. |

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
4. Host adapter contract for product repos.
5. Labyrinth mock sim host for local BANISH/AMAZE-style adapter testing.
6. CLI renderer as the first playable surface.

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
- `MuddleHost`
- optional host checkpoint export/import hooks
- `MuddleStaticHost`
- `MuddleCommandOutcome`
- `MuddleResource`
- explicit host/session errors
- ASCII room cards
- host-provided status/resource panels
- host-provided inventory panels
- host-provided map panels
- host-provided objective panels
- host-provided command hint panels
- renderer-provided recent transcript/log panels
- transcript recording
- `muddle-mock-sim` stateful labyrinth fixture host
- `muddle-banish-spike` Pilgrim Loss launcher adapter spike
- `muddle-amaze-spike` Silverstream escape-room adapter spike
- BANISH-owned `pilgrim_loss_muddle_host()` backed by BANISH surface data
- AMAZE-owned `silverstream_muddle_host()` backed by AMAZE surface data
- reusable `muddle-cli` runner entry points for product-owned launchers
- portable command-transcript save/resume contracts with optional host-owned checkpoints
- readable transcript export from the shared runner
- save/resume regression coverage in MUDDLE, BANISH, and AMAZE
- product-owned `banish-muddle` and `amaze-muddle` binaries
- CLI fixture play loop with named host selection
- local browser-backed `muddle-window` client with named host selection
- reusable `muddle-window` runner for product-owned hosts
- window save/load/transcript parity with the CLI command-replay persistence model
- clickable window command buttons from host-provided command hints
- in-window current-host restart/reset control
- `portfolio-showcase` window host for browsing MUDDLE-backed games, Knowledge Systems, Design Labs, and infrastructure

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
3. Add a stateful labyrinth mock sim host that combines BANISH-like resources and
   AMAZE-like locks.
4. Add a CLI renderer as the first playable surface over that adapter.
5. Add adapter selection so `muddle-cli` can mount named hosts.
6. Keep MUDDLE's CLI dependency-neutral by sharing its runner library with
   product-owned binaries such as `banish-muddle` and `amaze-muddle`.
7. Expand panel contracts for richer maps after two
   host adapters prove the minimal status/map shape.
8. Use `muddle-window` as the first local window surface; keep richer native GUI
   or TUI work behind the same host/session boundary.

## Loading and extension model

MUDDLE should load host experiences through explicit adapter crates, not through
implicit plugin discovery in the first wave. Each product repo owns a small
MUDDLE-facing adapter:

| Host | Adapter responsibility |
|---|---|
| BANISH | Expose settlement/play rooms, available exits, BANISH-specific verbs, and command outcomes. |
| AMAZE | Expose escape rooms, locks, clues, puzzle state, and command outcomes. |
| Board-game hosts | Expose tables, seats, pieces, legal moves, and turn outcomes. |

The adapter implements `MuddleHost`; renderers such as `muddle-cli`, product
launcher binaries, or a future rich TUI/window only talk to MUDDLE crates. MUDDLE
must not depend back on product repos, so product-owned launchers consume the
reusable CLI runner instead of being registered directly inside MUDDLE's workspace
CLI. If a later use case needs dynamic loading, it should be added after at least
one in-process host adapter proves the contract.

## Non-goals

- No MMO, sockets, accounts, or real-time multiplayer in the first wave.
- No product-specific game rules in `muddle-core`.
- No renderer beyond deterministic text/ASCII output until host contracts are
  proven.
- No rich native GUI/TUI renderer until the local browser-backed window proves
  the host/session boundary.
- No dynamic plugin loading until explicit in-process adapters have been proven.
- No runtime dependency on RALLY until a real adapter proves the boundary.

## Validation

```powershell
cargo test --quiet
cargo run -p muddle-window -- --list-hosts
```
