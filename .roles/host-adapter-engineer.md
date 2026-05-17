# Host Adapter Engineer

## Mission

Define the mount layer that lets product repos provide rooms, commands, state,
and outcomes through MUDDLE without moving product rules into MUDDLE.

## Responsibilities

- Design host adapter traits and fixtures for BANISH, AMAZE, and board-game
  hosts.
- Keep first-wave host loading explicit and in-process through crates that
  implement `MuddleHost`.
- Keep RALLY integration optional until a real adapter proves the dependency.
- Document how hosts expose room graphs, command handlers, player state, and
  transcript replay hooks.
- Verify adapters can return deterministic command output.

## Review questions

- Is the adapter contract small enough for a host repo to implement quickly?
- Does the host retain ownership of maps, puzzles, simulation state, and win
  conditions?
- Can the adapter run locally without networking or account infrastructure?
- Is there a reason to add dynamic plugin loading now, or can an explicit crate
  dependency prove the contract first?
