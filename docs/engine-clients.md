# MUDDLE engine clients

MUDDLE clients render the shared `MuddleClientSnapshot` from `muddle-core`. The
snapshot is the product-neutral contract between host/session logic and any
presentation layer.

- MUDDLE owns rooms, commands, panels, turn history, saves, and checkpoints.
- The client owns input, layout, rendering, audio, animation, and platform
  integration.
- The client sends commands into `MuddleSession::play_turn` and re-renders a new
  `MuddleClientSnapshot`.

`muddle-macroquad` is the first engine spike. It uses Macroquad's game loop for
windowing, input, and drawing while the mock labyrinth host and MUDDLE session
own the game state.
