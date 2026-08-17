# MUDDLE compatibility policy

MUDDLE is a pre-1.0 shared interaction foundation. Compatibility is deliberate
because product repositories mount their own games through its host, session,
snapshot, renderer, transcript, and persistence contracts.

## Protected contract

The protected surface includes:

- public `muddle-core` room, exit, command, turn, session, host, outcome,
  resource, inventory, hint, panel, visual-node, control, and client-snapshot
  APIs;
- the required and optional methods of `MuddleHost`, including checkpoint
  export and import behavior;
- command parsing and normalization, room transitions, transcript ordering,
  replay behavior, and public `MuddleError` meanings;
- `MUDDLE_SESSION_V1`, field escaping, command ordering, optional host
  checkpoints, and invalid-save rejection;
- `muddle.client_contracts` RUNE descriptors, versioned `v0` records,
  invariants, field stability metadata, and retained fixture;
- shared snapshot semantics consumed by CLI, browser-window, and Macroquad
  renderers; and
- the boundary that product repositories own rules, content, checkpoints,
  maps, win conditions, and domain-specific commands.

Internal refactoring is compatible only when these observable contracts remain
stable.

## Versioning rules

- Additive APIs or optional snapshot data may remain within the current `0.y`
  line when existing hosts, saves, and renderers retain their behavior.
- Breaking APIs, host requirements, command behavior, error meanings, snapshot
  fields, save encoding, replay semantics, or renderer controls require a
  minor-version bump while the affected crate is below `1.0`.
- Breaking RUNE descriptor changes require a new descriptor or collection
  version and an updated retained fixture.
- Prefer deprecation plus migration notes before removing a public item.
- A breaking change must identify affected hosts and include adapter or save
  migration guidance.
- Downstream repositories should pin commits for reproducible evidence.
  Branch consumers must run the downstream rehearsal before updating.

## Foundation tests

From the MUDDLE repository:

```powershell
cargo test -p muddle-core
cargo test -p muddle-mock-sim
cargo test -p muddle-cli
cargo test -p muddle-window
cargo test -p muddle-macroquad
```

These protect host/session behavior, command replay, checkpoints, snapshots,
controls, renderers, errors, and the retained RUNE contract fixture.

## Downstream breakage rehearsal

BANISH is the required first external consumer rehearsal because its Pilgrim
Loss host implements the real MUDDLE host boundary and exercises stateful play,
visual scenes, command-replay saves, and host-owned checkpoints.

From the BANISH repository:

```powershell
python tools\repo_map.py write-cargo-config
cargo test -p banish product_owned_muddle_host_plays_pilgrim_loss_path
cargo test -p banish product_owned_muddle_host_emits_visual_scene_nodes
cargo test -p banish product_owned_muddle_host_resumes_from_command_save
cargo test -p banish product_owned_muddle_host_resumes_from_checkpoint_save
```

The generated, ignored Cargo config patches MUDDLE and RALLY to their sibling
checkouts. A compile failure exposes public API breakage. Play, snapshot, replay,
or checkpoint failures expose behavioral or persistence drift.

MUDDLE foundation changes are not ready until the affected foundation tests and
the BANISH rehearsal pass.
