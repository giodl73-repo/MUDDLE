# MUDDLE engine clients

MUDDLE clients render the shared `MuddleClientSnapshot` from `muddle-core`. The
snapshot is the product-neutral contract between host/session logic and any
presentation layer. It includes raw play data plus reusable client controls for
text, image, button, and layout/group intent.

- MUDDLE owns rooms, commands, panels, turn history, saves, checkpoints, and
  presentation intent.
- The client owns input, concrete layout, rendering, audio, animation, and platform
  integration.
- The client sends commands into `MuddleSession::play_turn` and re-renders a new
  `MuddleClientSnapshot`.

`muddle-macroquad` is the first engine client. It uses Macroquad's game loop for
windowing, input, and drawing while the mounted host and MUDDLE session own the
game state.

## Macroquad core play parity

```powershell
cargo run -p muddle-macroquad -- --help
cargo run -p muddle-macroquad -- --list-hosts
cargo run -p muddle-macroquad
cargo run -p muddle-macroquad -- --host mock-labyrinth
cargo run -p muddle-macroquad -- --host banish-pilgrim-loss
cargo run -p muddle-macroquad -- --host amaze-silverstream
cargo run -p muddle-macroquad -- --host mock-labyrinth --save macroquad.muddle --transcript macroquad.txt
```

If no host is supplied, Macroquad opens a native host chooser. Type to filter by
host name, category, description, or suggested commands; use Up/Down to move the
selection and Enter to start. In play mode, Enter submits the typed command,
Up/Down recalls prior commands, F2 returns to host selection, F5 restarts the
current host, F6 writes configured `--save` and/or `--transcript` outputs, F7
reloads the configured `--save` path, and Escape quits. `--load`, `--save`, and
`--transcript` use the same command-replay save and transcript formats as
`muddle-cli` and `muddle-window`.

The engine client renders the same snapshot fields as the browser's core play
surface: active host metadata, room card, resources, inventory, objectives, map,
recent log, command hints, visible status, turn count, and recent history. It
does not own browser-only save-slot UX or product rules; those remain shared
MUDDLE session contracts and host responsibilities.

The Macroquad renderer now maps the snapshot into explicit game-client regions:
header, room, panels, commands, status, and history. Host command hints render as
native clickable buttons that submit the same command strings as typed input.
This keeps "what should be shown" reusable across clients while letting each
client decide how text, image placeholders, button controls, and layout groups
are drawn.
