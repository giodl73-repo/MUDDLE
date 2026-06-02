# MUDDLE RUNE contracts

MUDDLE exposes its shared room-command, client snapshot, visual node, reusable
control, and command-replay save records as RUNE descriptor evidence. This gives
AI agents and portfolio tooling a stable data contract for the surfaces consumed
by CLI, browser, native Macroquad, and product-owned launchers.

## Retained evidence

- `docs\rune\client_contracts.json` is generated from
  `muddle_core::rune_descriptor_collection()`.
- The collection id is `muddle.client_contracts`.
- The first slice covers room/session primitives, client snapshots, controls,
  visual nodes, panels, command hints, history entries, and session saves.

## Boundary

RUNE metadata describes MUDDLE's product-neutral UX and persistence contracts.
Product rules, host-owned checkpoint bodies, puzzle content, renderer behavior,
private playtest details, and downstream product conclusions remain outside
`muddle-core`.

