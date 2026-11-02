# MUDDLE Pitfalls

## MUDDLE-PF-01: Shared UX Becomes A Game Engine

**Status:** MITIGATED

**Pattern:** MUDDLE starts owning BANISH settlement rules, AMAZE puzzle logic,
board-game scoring, world maps, locks, simulations, or win conditions because
the shared renderer needs more context.

**Domain:** `muddle-core`, host adapters, runners, product spikes, and future
engine clients.

**Detection difficulty:** Renderer convenience can look product-neutral until a
second product needs a conflicting rule or state shape.

**Structural solution:** Keep product rules in host repos, require explicit
host adapters, and review feature ownership through Product Steward and Host
Adapter Engineer roles.

**Evidence:** `README.md`, `PRODUCT_PLAN.md`, `.roles/ROLE.md`, and
`docs/engine-clients.md`.

## MUDDLE-PF-02: Adapter Spike Becomes Product Authority

**Status:** MITIGATED

**Pattern:** `muddle-banish-spike` or `muddle-amaze-spike` becomes the source of
truth for product behavior instead of a proof fixture for product-owned handoff
APIs.

**Domain:** BANISH/AMAZE spike crates, `MuddleHost` registrations, CLI/window
host lists, and downstream rehearsals.

**Detection difficulty:** A playable spike feels authoritative even when it is
only carrying enough behavior to prove the shared adapter shape.

**Structural solution:** Treat spikes as adapter evidence, keep authoritative
behavior behind BANISH/AMAZE-owned APIs, and run documented downstream rehearsal
for foundation changes.

**Evidence:** `README.md`, `docs/compatibility.md`, and
`context/waves/2026-05-17-muddle-mud-engine/WAVE.md`.

## MUDDLE-PF-03: Renderer Forks Product Rules

**Status:** MITIGATED

**Pattern:** CLI, browser, Macroquad, or future engine code reimplements legal
commands, resources, locks, scene meaning, or win/loss state instead of rendering
host-provided snapshot fields.

**Domain:** `muddle-cli`, `muddle-window`, `muddle-macroquad`, reusable controls,
visual nodes, and product-owned launchers.

**Detection difficulty:** UI-specific shortcuts can hide duplicated rule checks
until transcript, save, or host parity breaks.

**Structural solution:** Make renderers consume `MuddleClientSnapshot` and
host-provided controls while leaving commands and outcomes with the mounted host.

**Evidence:** `docs/window-client.md`, `docs/engine-clients.md`, and
`crates/muddle-core/src/lib.rs`.

## MUDDLE-PF-04: Save Or Checkpoint Drift Breaks Resume

**Status:** MITIGATED

**Pattern:** A client changes command-replay save text, transcript ordering, or
checkpoint handling in a way that breaks replay, reload, import/export, or
product-owned checkpoint validation.

**Domain:** `MUDDLE_SESSION_V1`, `MuddleSessionSave`, CLI/window/Macroquad save
paths, save slots, and host checkpoints.

**Detection difficulty:** Fresh sessions can keep working while imported or
older saves fail only after a player tries to resume.

**Structural solution:** Keep save format compatibility documented and tested,
including legacy no-checkpoint saves, invalid-save rejection, and host checkpoint
round trips.

**Evidence:** `docs/compatibility.md`, `README.md`, and
`crates/muddle-core/src/lib.rs`.

## MUDDLE-PF-05: RUNE Descriptors Go Stale

**Status:** MITIGATED

**Pattern:** Shared client or persistence structs change while retained RUNE
descriptor fixtures still describe an older contract.

**Domain:** RUNE descriptor generation, `docs/rune/client_contracts.json`,
snapshot records, visual nodes, command hints, panels, history, and session
saves.

**Detection difficulty:** Rust code and manual docs can pass local review while
AI-readable contract evidence silently drifts.

**Structural solution:** Compare generated descriptors to the retained fixture
in tests and keep descriptor scope product-neutral.

**Evidence:** `docs/rune/README.md`, `docs/rune/client_contracts.json`, and
`crates/muddle-core/src/lib.rs`.

## MUDDLE-PF-06: Stale Integration Guidance Hides Completed Host Mounts

**Status:** MITIGATED

**Pattern:** README or product-plan guidance describes mock-only or future
adapter work after BANISH/AMAZE host spikes, window hosting, Macroquad parity,
checkpoint persistence, and RUNE evidence already exist.

**Domain:** README quick-start docs, product plan sequencing, portfolio PITFALL
adoption, and customer-facing repo status.

**Detection difficulty:** Tests and code pass while stale planning prose sends
agents toward duplicated adapter work or under-reports current customer value.

**Structural solution:** During PITFALL adoption, align README and product-plan
status with completed host mounting and make future work about product-owned
adoption rather than already-finished adapter proofs.

**Evidence:** PITFALL adoption updated `README.md` and `PRODUCT_PLAN.md` after
comparing them with host-list commands, wave pulses, engine-client docs, and
current crate layout.
