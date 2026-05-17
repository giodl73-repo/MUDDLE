# MUDDLE Phases

| Phase | Goal | Exit signal |
|---|---|---|
| 1. Core rooms | Establish product-neutral rooms, exits, commands, sessions, and transcripts. | `cargo test --quiet` passes for `muddle-core`. |
| 2. Host adapters | Define adapter contracts for BANISH, AMAZE, and board-game hosts. | A host can mount a room and return deterministic command output. |
| 3. ASCII maps | Add deterministic map/card rendering. | Room graph renders in stable text output. |
| 4. Save/resume | Add portable session persistence contracts. | A transcript can restore current room and state. |
| 5. Review gates | Integrate with product validation and RALLY where appropriate. | Product repos can validate mounted play flows. |

