# MUDDLE PITFALL index

MUDDLE uses PITFALL as a compact index over shared text/ASCII play-surface
doctrine: product-neutral room/session contracts, explicit host adapters,
portable transcripts and saves, shared client snapshots, and RUNE contract
evidence. The index cites existing roles, compatibility docs, wave closeouts,
RUNE fixtures, and tests without moving product rules into MUDDLE.

| Namespace | Kind | Path | Owner |
|---|---|---|---|
| `muddle` | `principles` | [muddle-principles.md](muddle-principles.md) | MUDDLE maintainers |
| `muddle` | `invariants` | [muddle-invariants.md](muddle-invariants.md) | MUDDLE maintainers |
| `muddle` | `pitfalls` | [muddle-pitfalls.md](muddle-pitfalls.md) | MUDDLE maintainers |

## Integration

- ROLES: `.roles/ROLE.md` routes product-boundary, core-contract,
  host-adapter, playtest, and validation concerns to matching reviewers.
- VTRACE: MUDDLE does not yet carry repo-local `docs/vtrace/` records; PITFALL
  cites compatibility docs, RUNE descriptors, wave pulses, role docs, and tests
  until trace rows are added.
- Tests: `cargo test --quiet`, package tests, CLI/window/Macroquad host-list
  smokes, RUNE fixture comparison, command-replay save tests, and PITFALL
  validators are the executable evidence hooks.
