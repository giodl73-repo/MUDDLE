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
| `muddle-core` | Product-neutral rooms, exits, commands, sessions, ASCII cards, transcripts, host adapter contracts, and reusable client text/image/layout/button controls. |
| `muddle-amaze-spike` | AMAZE Silverstream adapter spike that proves escape-room clue/lock mounting before replacing it with AMAZE-owned APIs. |
| `muddle-banish-spike` | BANISH Pilgrim Loss adapter spike that proves launcher-style mounting before replacing it with BANISH-owned APIs. |
| `muddle-mock-sim` | In-repo labyrinth mock host that exercises BANISH-like resources and AMAZE-like locks without depending on either repo. |
| `muddle-cli` | First playable command-line renderer for local adapter proof and transcript review. |
| `muddle-macroquad` | Lightweight Macroquad game-window client with a native host chooser, command input, command recall, restart/change-host controls, save-slot controls, and shared snapshot rendering inside a Rust engine loop. |
| `muddle-window` | Reusable local browser-backed window runner, grouped/filterable host chooser, and portfolio catalog over the same host/session contracts. |

## UX direction

MUDDLE starts with CLI because it is deterministic, scriptable, and useful for
testing host adapters. Browser and game-engine clients consume the same
`MuddleClientSnapshot` from `muddle-core`: MUDDLE owns rooms, commands, panels,
turn history, saves, checkpoints, and reusable client controls for text, image,
button, and layout/group intent, while each client owns input, rendering, audio,
animation, and platform integration.

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
cargo run -p muddle-macroquad -- --list-hosts
cargo run -p muddle-macroquad
cargo run -p muddle-macroquad -- --host mock-labyrinth
cargo run -p muddle-macroquad -- --host banish-pilgrim-loss
cargo run -p muddle-macroquad -- --host mock-labyrinth --save macroquad.muddle --transcript macroquad.txt --import portable.muddle --export exported.muddle
```

The CLI currently supports the `mock-labyrinth` host, a tiny labyrinth with a
camp ember, glyph antechamber, sealed gate, and echo vault. That lets MUDDLE
test named host mounting inside this repo before integrating BANISH or AMAZE.
For repeatable smoke checks, `--script <path>` reads newline-delimited commands
from a file while still supporting `--save`, `--load`, and `--transcript`.

The local window client is documented in
[`docs\window-client.md`](docs/window-client.md). It starts a local HTTP server
and optionally opens the default browser with `--open`, while still using the
same host/session contracts as the CLI. Its front screen groups registered hosts
by category and includes a filter box so a growing portfolio can be narrowed by
game/system name, category, description, or suggested commands. The default
`portfolio-showcase` host browses games, Knowledge Systems, Design Labs, and
infrastructure already visible through MUDDLE. Product repos can reuse the same
window runner, just as they reuse the CLI runner. The reusable window runner
supports `--save`, `--load`, and `--transcript` for command-replay session
persistence with optional host-owned checkpoints, renders host-provided command
hints as clickable action buttons, supports Up/Down command recall in the
command box, shows the full turn history in the browser, and exposes
`/transcript` for the same transcript text as the CLI. `/state` also exposes the
shared reusable controls array so browser clients can consume the same
text/image/button/group intent as engine clients while keeping the existing JSON
fields for compatibility. The history view can be filtered by turn, room,
command, or response text and shows matching/total counts for long sessions. The
responsive browser layout collapses cleanly on narrower displays and keeps the
command form sticky during long sessions.
Browser-side request failures are shown in-window instead of disappearing into
the developer console. Keyboard shortcuts cover common
persistence actions: Ctrl+S saves, Ctrl+R reloads, Ctrl+E exports save text, and
Ctrl+I imports save text. The active save and transcript output paths can be
copied from the browser when configured. The window also has in-session
**Save now**, **Reload save**, named **Save slot**/**Load
slot**/**Delete slot**, save text **Export**/**Import**, and **Restart host**
controls for managing the current session while preserving configured
save/transcript paths. Unavailable persistence controls are disabled with
tooltips when no configured save/transcript path or existing slot target exists.
Persistence actions report browser-visible success details for the affected
paths, slots, or save-text byte counts. Save slots are sibling files derived
from the configured `--save` path and show their path, size, modified time, and
a copy-path action in the browser. The slot browser can filter saved slots by
name or path, sort by name/newest/oldest/largest, and shows how many slots
match. Selecting a slot, or pressing **Use slot** in its detail row, fills the
slot-name field so load/export/delete actions clearly target that slot. A
selected slot can also export its save text into the browser text area without
loading the slot, while import/export uses the same portable command-replay save
text.

The Macroquad client is documented in
[`docs\engine-clients.md`](docs/engine-clients.md). It opens a native game loop
over the same `MuddleClientSnapshot` contract and now starts with a filterable
host chooser when no host is provided. `--list-hosts` and `--host <name>` mirror
the browser/CLI host-selection shape for the in-repo mock, BANISH, and AMAZE
adapter spikes. It also accepts `--load`, `--save`, and `--transcript` for the
same command-replay save and transcript formats used by the CLI/window clients.
In play mode, Enter submits commands, Up/Down recalls command history, F2
returns to host selection, F5 restarts the current host, F6 saves configured
outputs, F7 reloads the configured save path, F8 opens native save-slot
browsing, F11 exports current save text to the configured `--export` path, F12
imports command-replay save text from the configured `--import` path, and Escape
quits. Macroquad renders room cards,
resource/inventory/objective/map panels, command hints, visible status, turn
count, recent history, and a native Persistence panel with configured paths and
F6/F7/F11/F12 availability cues from the shared controls attached to the
snapshot while leaving product rules in the mounted host. Its renderer now builds
explicit header, room, panel, command, status, and history regions from those
controls, and host command hints render as clickable native buttons rather than
only flat text. The save-slot screen uses the same sibling save-file
convention as the browser client and supports filter/select, save, load, delete,
export, slot detail inspection, and name/newest/oldest/largest sorting actions
from the native loop.

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
