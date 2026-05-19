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

## Engine strategy

The portfolio should treat "game engine" as a render/input target, not as the
owner of game rules. MUDDLE keeps the portable room/session/snapshot contract;
product repos keep domain state and scene direction; engine clients turn that
state into a concrete play surface.

| Engine target | Best use | Should not own |
|---|---|---|
| `muddle-macroquad` | Fast Rust-native desktop play, deterministic visual smoke, keyboard/mouse loops, lightweight 2D scene rendering, and product-owned native launchers. | Product rules, puzzle truth, scene authorship, or asset pipeline decisions. |
| `muddle-window` | Accessible local browser play, inspectable JSON/state endpoints, persistence UX, and fast UI iteration. | Native game feel, engine timing, or final scene composition. |
| Full scene engine candidate, for example Godot | Designer-authored scenes, hand-placed rooms/tables/world maps, animation timelines, stronger UI layout, imported art/audio assets, and export polish. | Canonical game state if it would fork MUDDLE/product rules. |

Macroquad is therefore the current **testable native prototype engine**. It is
excellent for proving that product-owned scenes have enough nodes, frames,
animations, input, save/load, and replay coverage. It is not expected to become a
full editor or asset-authoring environment. If a product needs hand-authored
rooms, animation timelines, lighting, imported sprite atlases, or designer-owned
placement workflows, add a full scene-engine adapter beside Macroquad rather
than replacing the portable MUDDLE/product contract.

The next engine decision should be evidence-based: keep Macroquad as the native
smoke and prototype target, then introduce a Godot-style adapter only after one
product scene has a specific authoring need that Macroquad cannot express
cleanly.

## What goes into building a game engine

A game engine is not one feature. It is a stack of systems that repeatedly turns
assets, input, rules, and state into frames, sound, saved progress, and player
feedback.

| Engine subsystem | What it does | MUDDLE portfolio stance |
|---|---|---|
| Main loop | Runs update/render ticks, input polling, timing, pause/resume, and exit. | Macroquad owns this for the native client. |
| Platform/window layer | Opens windows, handles display size, fullscreen, OS events, files, and clipboard. | Engine client owns it; product repos should not. |
| Input system | Normalizes keyboard, mouse, controller, touch, focus, and command shortcuts. | Engine client maps input into MUDDLE commands/actions. |
| Renderer | Draws sprites, text, shapes, cameras, layers, materials, shaders, UI, and post-processing. | Macroquad provides a lightweight renderer; future full engines may add richer scene rendering. |
| Scene graph / ECS | Stores objects, transforms, parent/child hierarchy, components, tags, and update order. | Product-owned visual nodes are the current portable scene description. |
| Asset pipeline | Imports, validates, packs, caches, reloads, and versions textures, fonts, audio, animation, and data. | Mostly missing today; this is a major reason to consider a full scene engine later. |
| Animation | Handles sprite sheets, tweening, timelines, skeletal rigs, particles, transitions, and state machines. | Macroquad has basic hooks; richer authored animation belongs in a future adapter if needed. |
| Physics/collision | Simulates movement, triggers, raycasts, constraints, hitboxes, and spatial queries. | Not central to current MUDDLE slices; add only when a product needs it. |
| Audio | Plays music, sound effects, spatial audio, mixing, ducking, and transitions. | Not yet a core gate; product scene direction should define audio needs first. |
| UI system | Lays out panels, buttons, text, menus, HUD, accessibility, focus, and localization. | MUDDLE snapshots provide intent; each client renders concrete UI. |
| Scripting/gameplay layer | Executes game rules, events, dialogue, quests, puzzles, AI, and state transitions. | Product repos plus MUDDLE hosts own this; the engine should not fork the rules. |
| Persistence | Saves/loads state, checkpoints, slots, migrations, transcripts, and replay data. | MUDDLE owns portable command-replay/checkpoint contracts; clients expose UX. |
| Tooling/editor | Lets designers place objects, edit scenes, preview animation, inspect state, and tune assets. | Macroquad does not provide this; Godot-style tooling would matter here. |
| Build/export pipeline | Packages assets and binaries for desktop/web/mobile, with versioning and reproducible builds. | TRACKER/product launchers cover early native builds; full export polish is later. |
| Testing/telemetry | Runs deterministic smokes, screenshots, replay checks, performance metrics, and diagnostics. | Visual smoke/persona harnesses are already valuable and should stay engine-agnostic. |

The strategic question is which of these we want to build ourselves. For this
portfolio, the answer should be selective: build the portable gameplay contract,
host adapters, validation, and product-owned scene descriptions; reuse existing
engines/frameworks for windowing, rendering, input, animation tooling, and asset
pipelines whenever those become the bottleneck.

Macroquad is a thin engine/framework: it gives us main loop, window/input,
drawing, and enough rendering to validate native play. Godot/Unity/Unreal are
full engines: they add editor tooling, scene graphs, asset import, animation,
physics, audio, UI, and export workflows. Building a full engine ourselves would
mean owning all of that surface area, which is usually only worth it when the
engine itself is the product.

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
