# MUDDLE Roles

MUDDLE work is split by responsibility so the shared interaction layer stays
product-neutral while host repos can still extend it for their own playable
experiences.

| Role | Owns | Does not own |
|---|---|---|
| Product steward | Product boundary, wave scope, non-goals, host adoption order. | Product-specific room content or rules. |
| Core engineer | Stable room, command, transcript, rendering, and persistence contracts. | BANISH, AMAZE, or board-game domain logic. |
| Host adapter engineer | Mount contracts that let product repos expose rooms through MUDDLE. | RALLY metrics semantics or host simulation internals. |
| Scene experience director | Product-owned scene composition: physical surfaces, props, placement zones, stateful visual nodes, and product fantasy. | Shared renderer mechanics or product-neutral contracts. |
| Playtest designer | Player-facing command vocabulary, room readability, scene comprehension, and transcript review. | Engine-only abstractions without playable evidence. |
| Validation gatekeeper | Deterministic tests, transcript fixtures, compatibility checks, and release gates. | Silent fallback behavior or unverified integrations. |

## Working rule

Every MUDDLE change should name the role it primarily advances and keep that
role inside the boundary above. If a change needs product rules, it belongs in a
host repo first and should come back to MUDDLE only as a proven reusable
contract.

For native scene work, the scene experience director belongs in the product repo:
BANISH owns the playable world surface, AMAZE owns escape-room walls and puzzle
placements, TIGRIS owns the tabletop, and QUEST owns dungeon/table encounter
composition. MUDDLE only graduates reusable renderer support after those product
scenes prove a shared need.
