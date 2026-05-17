# Pulse 03 - Adapter Seam and CLI

## Status

Active.

## Goal

Prove that MUDDLE can mount a host-provided room graph and expose it through a
simple playable CLI before investing in richer renderers.

## Work

- Add `MuddleHost` as the product-neutral adapter trait.
- Add `MuddleCommandOutcome` and explicit host/session errors.
- Add `MuddleStaticHost` as an in-repo fixture host.
- Add `muddle-cli` as the first playable renderer over the core session and host
  contracts.
- Record CLI-first, richer-renderer-second as the UX direction.
- Record explicit in-process host adapters as the first loading model for
  BANISH, AMAZE, and board-game repos.

## Decision

MUDDLE will use CLI as the first renderer because it is deterministic,
scriptable, and transcript-friendly. A richer TUI/window should be added later
as a renderer over `muddle-core`, not as a separate engine.

Host repos should provide MUDDLE extensions as explicit adapter crates that
implement `MuddleHost`. Dynamic plugin loading is out of scope until an
in-process BANISH or AMAZE adapter proves the contract.

## Validation

```powershell
cargo test --quiet
cargo run -p muddle-cli
```
