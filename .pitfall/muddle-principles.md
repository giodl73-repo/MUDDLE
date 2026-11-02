# MUDDLE Principles

## MUDDLE-P-01: Interaction Shape, Not Game Rules

**Status:** ACTIVE

**Statement:** MUDDLE owns shared room-command UX, sessions, transcripts,
persistence shape, renderer contracts, and host adapter APIs; BANISH, AMAZE, and
other products own content, rules, maps, locks, simulations, and win conditions.

**Decision rule:** A feature belongs in MUDDLE only when it remains useful for
multiple hosts without importing one product's domain semantics.

**Evidence:** `README.md`, `PRODUCT_PLAN.md`, `.roles/ROLE.md`, and
`docs/compatibility.md`.

## MUDDLE-P-02: Explicit Host Adapters Beat Plugin Magic

**Status:** ACTIVE

**Statement:** Hosts mount through explicit Rust crates and `MuddleHost`
contracts, not a generic plugin loader that can hide ownership, versioning, or
rule boundaries.

**Decision rule:** New product integrations must name the adapter crate, the
product-owned handoff surface, and the commands/outcomes exposed to MUDDLE.

**Evidence:** `README.md`, `PRODUCT_PLAN.md`, and
`.roles/host-adapter-engineer.md`.

## MUDDLE-P-03: Transcript And Save Contracts Are Player Memory

**Status:** ACTIVE

**Statement:** Transcripts, command replay saves, and optional host checkpoints
are durable player-session artifacts, not incidental test logs.

**Decision rule:** Persistence changes must preserve command ordering, transcript
meaning, invalid-save rejection, and host-owned checkpoint boundaries or document
an explicit compatibility change.

**Evidence:** `docs/compatibility.md`, `.roles/playtest-designer.md`, and
`crates/muddle-core/src/lib.rs`.

## MUDDLE-P-04: One Snapshot Feeds Many Clients

**Status:** ACTIVE

**Statement:** CLI, browser window, Macroquad, and future engine clients consume
the shared `MuddleClientSnapshot` and reusable controls instead of forking
product rules in renderers.

**Decision rule:** Renderer work must prove it can display host-provided state
and actions without becoming authoritative for product behavior.

**Evidence:** `README.md`, `docs/window-client.md`, `docs/engine-clients.md`,
and `crates/muddle-core/src/lib.rs`.

## MUDDLE-P-05: RUNE Descriptors Are Contract Evidence

**Status:** ACTIVE

**Statement:** Retained RUNE descriptors expose AI-readable shared client,
visual, command, control, history, and session-save contracts.

**Decision rule:** Descriptor-generating changes must update retained fixtures
and keep product rules, checkpoint bodies, puzzle content, and renderer behavior
outside the descriptor scope.

**Evidence:** `docs/rune/README.md`,
`docs/rune/client_contracts.json`, and `crates/muddle-core/src/lib.rs`.
