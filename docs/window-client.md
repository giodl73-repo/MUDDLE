# MUDDLE Local Window Client

Status: first local browser-backed window surface.

The window client is intentionally thin: it starts a local HTTP server, opens a
browser window when requested, and drives the same `MuddleHost` +
`MuddleSession` contracts used by `muddle-cli`. It is not a new engine and does
not own product rules.

## Run

```powershell
cargo run -p muddle-window -- --open
cargo run -p muddle-window -- --host portfolio-showcase --open
cargo run -p muddle-window -- --host mock-labyrinth --open
cargo run -p muddle-window -- --host banish-pilgrim-loss --open
cargo run -p muddle-window -- --host amaze-silverstream --open
```

If `--open` is omitted, open the printed local URL manually.

```powershell
cargo run -p muddle-window -- --addr 127.0.0.1:4777
cargo run -p muddle-window -- --host banish-pilgrim-loss --save pilgrim-loss.window.muddle --transcript pilgrim-loss.window.txt --open
cargo run -p muddle-window -- --host banish-pilgrim-loss --load pilgrim-loss.window.muddle --save pilgrim-loss.window.muddle --transcript pilgrim-loss.window.txt --open
```

The front screen lists every mounted host, grouped by registration category, and
includes a filter box that matches host name, category, description, and
suggested commands. Choosing a host starts a fresh local session for that host;
the in-window **Change host** button returns to the chooser. The default
`portfolio-showcase` host is a browsable catalog for MUDDLE-backed and adjacent
systems currently visible in the portfolio: games, Knowledge Systems, Design
Labs, and infrastructure.

Product repos can depend on the `muddle-window` crate and call
`run_muddle_window_hosts_from_env_args()` with their own categorized
`MuddleHost` registrations. This mirrors the existing reusable `muddle-cli`
runner shape.
The runner accepts `--save`, `--load`, and `--transcript`; saves use the same
command-replay format as the CLI and include optional host-owned checkpoints
when the mounted host implements checkpoint export/import. Transcripts use the
same transcript renderer.
Host-provided command hints render as clickable action buttons, so every
compatible host can expose its next legal/common commands without custom window
code. The command box keeps browser-local command recall with Up/Down keys so
typed and clicked commands can be replayed without leaving the current window.
The browser view also renders the full turn history and links to `/transcript`,
which returns the same text transcript format used by `muddle-cli`. The layout
collapses to a single column on narrower browser windows, and the command form
stays sticky near the bottom of the viewport so long histories do not push input
out of reach. Failed local HTTP requests render a visible in-window status
message so local runner failures are not silent.
The **Save now** button writes the configured `--save` and/or `--transcript`
paths immediately; Ctrl+S triggers the same action from the keyboard. The
**Reload save** button reloads the configured `--save` path without restarting
the local server; Ctrl+R triggers reload. When configured, the active save and
transcript output paths also render copy buttons. The save-slot controls write,
load, and delete named sibling save files derived from the configured `--save` path;
slot names may use letters, numbers, dash, and underscore. The slot list shows
each slot's path, byte size, and modified time, and includes a **Copy path**
button for copying the resolved slot file path. A browser-local slot filter
matches saved slots by name or path and shows the visible/total slot count.
Selecting a slot, or pressing **Use slot** in a slot detail row, fills the
slot-name field and marks that slot as the load/export/delete target.
Persistence buttons are disabled with tooltips when the current runner has no
matching `--save`, `--transcript`, or existing slot target. The import/export controls copy
the current command-replay save text into a browser text area and can import
compatible save text back into the currently mounted host; Ctrl+E exports and
Ctrl+I imports. **Export slot text** copies the selected slot's command-replay
save text into the same browser text area without loading that slot. Save,
reload, slot, import, and export actions report visible success details for the
affected path, slot, or save-text byte count.
The **Restart host** button resets the currently selected host to a fresh
session while preserving any `--save` and `--transcript` output paths.

## Boundary

MUDDLE owns the local renderer, command entry, room panel display, transcript
count, turn history, local transcript endpoint, persistence controls, and
host/session handoff. BANISH, AMAZE, and future products still own their content,
rules, maps, puzzles, simulations, and win conditions.

## Validation

```powershell
cargo test --quiet
cargo run -p muddle-window -- --list-hosts
cargo run -p muddle-window -- --addr 127.0.0.1:4777
git grep -n "muddle-window" -- README.md PRODUCT_PLAN.md docs\window-client.md Cargo.toml
```
