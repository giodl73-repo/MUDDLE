# MUDDLE Invariants

## MUDDLE-I-01: Product Rules Stay In Host Repos

**Status:** VERIFIED

**Invariant:** BANISH, AMAZE, and future product rules remain in product-owned
hosts and handoff APIs; MUDDLE core accepts rooms, commands, outcomes, snapshots,
and optional checkpoints.

**Why it matters:** The shared play surface can support many portfolio games
only if it avoids becoming a product-specific game engine.

**Test:** `cargo test --quiet`.

**Evidence:** `README.md`, `PRODUCT_PLAN.md`, and
`.roles/product-steward.md`.

## MUDDLE-I-02: Command-Replay Saves Stay Compatible

**Status:** VERIFIED

**Invariant:** `MUDDLE_SESSION_V1` saves preserve current room, command order,
field escaping, optional host checkpoints, and invalid-save rejection.

**Why it matters:** Save/resume portability is a user-visible contract across
CLI, browser, Macroquad, and product-owned launchers.

**Test:** `cargo test -p muddle-core`.

**Evidence:** `docs/compatibility.md` and `crates/muddle-core/src/lib.rs`.

## MUDDLE-I-03: CLI Window And Macroquad Share Snapshot Semantics

**Status:** VERIFIED

**Invariant:** CLI, `muddle-window`, and `muddle-macroquad` consume the same
host/session/snapshot contracts and expose host selection without renderer-owned
rules.

**Why it matters:** A shared UX engine is valuable only when multiple clients can
render the same product state without separate gameplay implementations.

**Test:** `cargo run -p muddle-window -- --list-hosts` and
`cargo run -p muddle-macroquad -- --list-hosts`.

**Evidence:** `README.md`, `docs/window-client.md`, and
`docs/engine-clients.md`.

## MUDDLE-I-04: Host Checkpoints Stay Host-Owned

**Status:** VERIFIED

**Invariant:** MUDDLE stores and transports optional checkpoint payloads, but the
mounted host exports, imports, interprets, and validates checkpoint bodies.

**Why it matters:** Shared persistence can resume stateful products without
teaching MUDDLE product-specific state machines.

**Test:** `cargo test -p muddle-core -p muddle-mock-sim`.

**Evidence:** `docs/compatibility.md`, `README.md`, and
`crates/muddle-core/src/lib.rs`.

## MUDDLE-I-05: RUNE Fixture Matches Generated Descriptors

**Status:** VERIFIED

**Invariant:** `docs/rune/client_contracts.json` matches
`muddle_core::rune_descriptor_collection()` for shared client and persistence
contracts.

**Why it matters:** Agents and downstream tooling need retained descriptor
evidence that matches the code instead of stale contract prose.

**Test:** `cargo test -p muddle-core`.

**Evidence:** `docs/rune/README.md`,
`docs/rune/client_contracts.json`, and `crates/muddle-core/src/lib.rs`.
