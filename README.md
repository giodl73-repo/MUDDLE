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
    save/resume shape, optional host checkpoints, and host adapter API.

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
| `muddle-amaze-spike` | AMAZE Silverstream adapter spike that proves escape-room clue/lock mounting before replacing it with AMAZE-owned APIs. |
| `muddle-banish-spike` | BANISH Pilgrim Loss adapter spike that proves launcher-style mounting before replacing it with BANISH-owned APIs. |
| `muddle-mock-sim` | In-repo labyrinth mock host that exercises BANISH-like resources and AMAZE-like locks without depending on either repo. |
| `muddle-cli` | First playable command-line renderer for local adapter proof and transcript review. |
| `muddle-window` | Reusable local browser-backed window runner, host chooser, and portfolio catalog over the same host/session contracts. |

## UX direction

MUDDLE starts with CLI because it is deterministic, scriptable, and useful for
testing host adapters. A richer TUI/window renderer should come later as another
surface over the same `muddle-core` session and host contracts, not as a
separate engine.

The shared play surface already has the first common game-screen panels:

| Panel | Purpose |
|---|---|
| Status/resources | Host-provided counts and state such as embers, locks, seeds, manifests, or session status. |
| Map | Host-provided ASCII location sketch with the current room marked as `@`. |
| Objectives | Host-provided current goals so the player knows what to pursue. |
| Commands | Contextual command hints so the player is not guessing verbs. |

```powershell
cargo run -p muddle-cli
cargo run -p muddle-cli -- --list-hosts
cargo run -p muddle-cli -- --host mock-labyrinth
cargo run -p muddle-cli -- --host banish-pilgrim-loss
cargo run -p muddle-cli -- --host amaze-silverstream
cargo run -p muddle-cli -- --host mock-labyrinth --script commands.txt --transcript play.txt
cargo run -p muddle-window -- --open
cargo run -p muddle-window -- --host portfolio-showcase --open
cargo run -p muddle-window -- --host banish-pilgrim-loss --open
cargo run -p muddle-window -- --host banish-pilgrim-loss --save pilgrim-loss.window.muddle --transcript pilgrim-loss.window.txt --open
```

The CLI currently supports the `mock-labyrinth` host, a tiny labyrinth with a
camp ember, glyph antechamber, sealed gate, and echo vault. That lets MUDDLE
test named host mounting inside this repo before integrating BANISH or AMAZE.
For repeatable smoke checks, `--script <path>` reads newline-delimited commands
from a file while still supporting `--save`, `--load`, and `--transcript`.

The local window client is documented in
[`docs\window-client.md`](docs/window-client.md). It starts a local HTTP server
and optionally opens the default browser with `--open`, while still using the
same host/session contracts as the CLI. Its front screen includes a
`portfolio-showcase` host for browsing games, Knowledge Systems, Design Labs,
and infrastructure already visible through MUDDLE. Product repos can reuse the
same window runner, just as they reuse the CLI runner. The reusable window
runner supports `--save`, `--load`, and `--transcript` for command-replay
session persistence with optional host-owned checkpoints, renders host-provided
command hints as clickable action buttons, shows the full turn history in the
browser, and exposes `/transcript` for the same transcript text as the CLI. The
window also has an in-session **Restart host** control for replaying the current
host while preserving configured save/transcript paths.

## Host extension model

BANISH, AMAZE, and other products do not get loaded as generic plugins in the
first wave. They provide explicit adapters that implement `MuddleHost`.

| Layer | Responsibility |
|---|---|
| `muddle-core` | Defines `MuddleHost`, `MuddleRoom`, `MuddleCommand`, sessions, outcomes, transcripts, and optional host checkpoint hooks. |
| `muddle-mock-sim` | Proves host-owned mutable labyrinth state, resources, locks, command outcomes, and checkpoint export/import inside the MUDDLE workspace. |
| `muddle-banish-spike` | Proves a BANISH-shaped launcher adapter surface; BANISH now exposes `pilgrim_loss_muddle_surface()` as the product-owned handoff API. |
| `muddle-amaze-spike` | Proves an AMAZE-shaped escape-room adapter surface; AMAZE now exposes `silverstream_muddle_surface()` as the product-owned handoff API. |
| Host adapter crate | Converts BANISH/AMAZE/board-game state into MUDDLE rooms and command outcomes. |
| Renderer | CLI and local window both select hosts and call the same session APIs. |

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
