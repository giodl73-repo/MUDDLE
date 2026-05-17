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

## Destination

MUDDLE's destination is one shared play surface where a player can launch and
play any supported BANISH game or AMAZE escape room through the same commands,
session model, transcript, and renderer.

The acceptance target is:

1. `muddle-cli` can select a mounted BANISH or AMAZE adapter.
2. The same session loop can enter rooms, inspect state, issue host-defined
   verbs, move between rooms, and record a transcript.
3. BANISH and AMAZE keep their own content, rules, maps, locks, simulations, and
   win conditions.
4. MUDDLE owns only the shared room-command UX, renderer contracts, transcript,
   save/resume shape, and host adapter API.

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
| `muddle-mock-sim` | In-repo labyrinth mock host that exercises BANISH-like resources and AMAZE-like locks without depending on either repo. |
| `muddle-cli` | First playable command-line renderer for local adapter proof and transcript review. |

## UX direction

MUDDLE starts with CLI because it is deterministic, scriptable, and useful for
testing host adapters. A richer TUI/window renderer should come later as another
surface over the same `muddle-core` session and host contracts, not as a
separate engine.

```powershell
cargo run -p muddle-cli
```

The CLI currently mounts `muddle-mock-sim`, a tiny labyrinth with a camp ember,
glyph antechamber, sealed gate, and echo vault. That lets MUDDLE test a stateful
host inside this repo before integrating BANISH or AMAZE.

## Host extension model

BANISH, AMAZE, and other products do not get loaded as generic plugins in the
first wave. They provide explicit adapters that implement `MuddleHost`.

| Layer | Responsibility |
|---|---|
| `muddle-core` | Defines `MuddleHost`, `MuddleRoom`, `MuddleCommand`, sessions, outcomes, and transcript behavior. |
| `muddle-mock-sim` | Proves host-owned mutable labyrinth state, resources, locks, and command outcomes inside the MUDDLE workspace. |
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
