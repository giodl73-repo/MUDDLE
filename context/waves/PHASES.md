# MUDDLE Phases

| Phase | Goal | Exit signal |
|---|---|---|
| 1. Core rooms | Establish product-neutral rooms, exits, commands, sessions, and transcripts. | `cargo test --quiet` passes for `muddle-core`. |
| 2. Host adapters | Define adapter contracts for BANISH, AMAZE, and board-game hosts. | A host can mount a room and return deterministic command output. |
| 3. Labyrinth mock sim | Add an in-repo stateful labyrinth host to test BANISH/AMAZE-style rules without depending on either repo. | Mock sim tests prove state changes, locks, and movement through `MuddleHost`. |
| 4. CLI play surface | Add the first playable renderer over the host adapter seam. | `cargo run -p muddle-cli` can play a fixture room loop. |
| 5. Host selection | Let the CLI mount named host adapters without renderer-specific rules. | `muddle-cli --host mock-labyrinth` selects the mock host; BANISH and AMAZE names are added by adapter spikes. |
| 6. BANISH adapter spike | Mount one BANISH playable surface through `MuddleHost`. | `muddle-cli --host banish-pilgrim-loss` drives a transcriptable Pilgrim Loss launcher path. |
| 7. AMAZE adapter spike | Mount one AMAZE escape room through `MuddleHost`. | `muddle-cli --host amaze-silverstream` drives a transcriptable clue/lock/escape path. |
| 8. Save/resume | Add portable session persistence contracts. | A transcript can restore current room and state. |
| 9. Play panels | Add deterministic status/resource, objective, command-hint, and map panels beyond room cards. | Hosts render resource counts, goals, command hints, and ASCII maps through shared renderer contracts. |
| 10. Rich renderer | Add a richer TUI/window surface once CLI contracts are stable. | The richer surface reuses `muddle-core` without duplicating engine rules. |
| 11. Review gates | Integrate with product validation and RALLY where appropriate. | Product repos can validate mounted play flows. |

