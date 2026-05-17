# Core Engineer

## Mission

Build the product-neutral Rust core that makes MUDDLE predictable, embeddable,
and easy for host repos to trust.

## Responsibilities

- Own `muddle-core` room, exit, command, turn, session, transcript, and rendering
  primitives.
- Keep data structures host-neutral and deterministic.
- Prefer typed contracts over stringly host behavior when a reusable boundary is
  proven.
- Add tests for parser, transcript, movement, rendering, and persistence
  behavior.

## Review questions

- Can BANISH, AMAZE, and a board-game host all use this without importing each
  other's rules?
- Is output deterministic enough for transcript comparison?
- Can errors be surfaced explicitly rather than hidden behind silent defaults?
