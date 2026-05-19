# MUDDLE engine clients

MUDDLE clients render the shared `MuddleClientSnapshot` from `muddle-core`. The
snapshot is the product-neutral contract between host/session logic and any
presentation layer. It includes raw play data plus reusable client controls for
text, image, button, and layout/group intent.

- MUDDLE owns rooms, commands, panels, turn history, saves, checkpoints, and
  presentation intent.
- Product hosts own scene experience and scene design: what the player is
  looking at, why it matters, which physical surfaces/props/tokens exist, and
  how those nodes express product fantasy and rules.
- The client owns input, concrete layout, rendering, audio, animation, and platform
  integration.
- The client sends commands into `MuddleSession::play_turn` and re-renders a new
  `MuddleClientSnapshot`.

`muddle-macroquad` is the first engine client. It uses Macroquad's game loop for
windowing, input, and drawing while the mounted host and MUDDLE session own the
game state. Its crate now exposes the same run loop as reusable library functions
so product repos can ship native launchers over their own hosts without copying
renderer code or registering product rules inside MUDDLE.

## Macroquad core play parity

```powershell
cargo run -p muddle-macroquad -- --help
cargo run -p muddle-macroquad -- --list-hosts
cargo run -p muddle-macroquad
cargo run -p muddle-macroquad -- --host mock-labyrinth
cargo run -p muddle-macroquad -- --host banish-pilgrim-loss
cargo run -p muddle-macroquad -- --host amaze-silverstream
cargo run -p muddle-macroquad -- --host mock-labyrinth --save macroquad.muddle --transcript macroquad.txt --import portable.muddle --export exported.muddle
```

If no host is supplied, Macroquad opens a native host chooser. Type to filter by
host name, category, description, or suggested commands; use Up/Down to move the
selection and Enter to start. In play mode, Enter submits the typed command,
Up/Down recalls prior commands, F2 returns to host selection, F5 restarts the
current host, F6 writes configured `--save` and/or `--transcript` outputs, F7
reloads the configured `--save` path, F8 opens a native save-slot screen, F11
exports current command-replay save text to the configured `--export` path, F12
imports command-replay save text from the configured `--import` path, and Escape
quits. `--load`, `--save`, `--import`, `--export`, and `--transcript` use the
same command-replay save and transcript formats as `muddle-cli` and
`muddle-window`.

The save-slot screen uses the same sibling-file convention as `muddle-window`
(`base.slot-name.ext`). Type to filter existing slots or name a new one, use
Up/Down to select, F6 saves/overwrites the selected or typed slot, F9 cycles
sorting by name/newest/oldest/largest, Enter/F10 loads it, Delete removes it,
F11 exports the selected command-replay save text to `--export`, and Escape
returns to play. Slot rows show name, byte size, modified timestamp, and
resolved path so native players can inspect slot details without the browser UI.

The engine client renders the same snapshot fields as the browser's core play
surface: active host metadata, room card, resources, inventory, objectives, map,
recent log, command hints, visible status, turn count, and recent history. It
also renders a native Persistence panel with configured save, transcript,
import, and export paths plus availability cues for F6/F7/F11/F12 actions,
without owning product rules; those remain shared MUDDLE session contracts and
host responsibilities.

The Macroquad renderer now maps the snapshot's reusable controls into explicit
game-client regions: header, room, panels, commands, status, and history. Host
command hints render as native clickable buttons that submit the same command
strings as typed input. This keeps "what should be shown" reusable across
clients while letting each client decide how text, image placeholders, button
controls, and layout groups are drawn.

Scene ownership is intentionally split. BANISH, AMAZE, TIGRIS, QUEST, or any
future host must deliver the actual scene direction through product-owned visual
nodes: tabletop planes, escape-room walls, world maps, actors, props, puzzle
fixtures, HUD elements, and stateful frames. `muddle-macroquad` should make those
nodes read better with product-neutral depth, color, layout, input, smoke gates,
and animation hooks, but it should not decide what a BANISH trail, AMAZE wall, or
TIGRIS table is supposed to be.

Product-owned native launchers should call `run_muddle_macroquad_hosts` with
their own `MuddleClientHostRegistration` list and a `MuddleMacroquadRunConfig`
screen title. The product binary owns its Macroquad window attribute and may set
default save/transcript/import/export paths before entering the shared run loop.
