# MUDDLE

MUDDLE is a shared text/ASCII interaction engine for playable rooms, commands,
state, transcripts, save/resume surfaces, and host adapters.

It is not a game by itself. It is the reusable UX layer that lets product repos
host their own playable experiences:

| Host repo | Experience |
|---|---|
| BANISH | Video-game / moving-settlement MUD-like launcher and play surface. |
| AMAZE | Escape-room rooms, locks, clues, and puzzle state. |
| TIGRIS and board-game repos | Tables, turns, pieces, and scenario rooms. |

## Workspace

MUDDLE is a Cargo workspace so the repo root stays explicit and Cargo does not
join parent workspaces accidentally.

```powershell
cargo test --quiet
```

Current crates:

| Crate | Purpose |
|---|---|
| `muddle-core` | Product-neutral rooms, exits, commands, sessions, ASCII cards, transcripts, and host adapter contracts. |
| `muddle-cli` | First playable command-line renderer for local adapter proof and transcript review. |

## UX direction

MUDDLE starts with CLI because it is deterministic, scriptable, and useful for
testing host adapters. A richer TUI/window renderer should come later as another
surface over the same `muddle-core` session and host contracts, not as a
separate engine.

```powershell
cargo run -p muddle-cli
```

## Host extension model

BANISH, AMAZE, and other products do not get loaded as generic plugins in the
first wave. They provide explicit adapters that implement `MuddleHost`.

| Layer | Responsibility |
|---|---|
| `muddle-core` | Defines `MuddleHost`, `MuddleRoom`, `MuddleCommand`, sessions, outcomes, and transcript behavior. |
| Host adapter crate | Converts BANISH/AMAZE/board-game state into MUDDLE rooms and command outcomes. |
| Renderer | CLI first, richer TUI/window later; both call the same session and host APIs. |

The first integration shape should be in-process and explicit:

```rust
let mut host = banish_muddle::BanishHost::new(...);
let mut session = MuddleSession::for_host(&host)?;
session.play_turn(&mut host, MuddleCommand::parse("look"))?;
```

This keeps domain rules in the host repo. BANISH owns settlement state and
verbs; AMAZE owns locks, clues, and puzzle state. MUDDLE only requires that the
host can expose the current room and produce deterministic command outcomes.

## Product boundary

MUDDLE owns interaction shape, not domain rules.

- RALLY remains the deterministic simulation, metrics, validation, and replay
  substrate.
- BANISH owns settlement/game content.
- AMAZE owns escape-room content.
- Board-game repos own rules, pieces, and scenarios.

## Non-goals

- Do not build networking/MMO infrastructure yet.
- Do not put BANISH, AMAZE, or board-game rules into MUDDLE core.
- Do not require a graphical renderer before text/ASCII contracts are stable.
- Do not make MUDDLE responsible for RALLY metrics or validation semantics.
