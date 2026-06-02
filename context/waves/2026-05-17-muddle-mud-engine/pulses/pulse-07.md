# Pulse 07 - RUNE client contracts

## Status

Done.

## Goal

Expose MUDDLE's shared client, visual, command, and persistence records as
retained RUNE descriptor evidence.

## Work

- Add pinned RUNE descriptor dependencies to `muddle-core`.
- Derive RUNE contracts for stable product-neutral MUDDLE data records.
- Add `muddle_core::rune_descriptor_collection()` and retained evidence at
  `docs\rune\client_contracts.json`.
- Document the RUNE boundary in `docs\rune\README.md`.

## Outcome

AI and portfolio tooling can now inspect MUDDLE's shared client contract shape
without source scraping or product-specific host rules.

## Validation

```powershell
cargo fmt --check
cargo test --quiet
git diff --check
```

