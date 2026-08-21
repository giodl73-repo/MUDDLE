# MUDDLE Review Panel

MUDDLE reviews keep the shared room-command interaction layer product-neutral
while host repositories retain their playable content and rules.

## Active Roles

| Role | Protects | Invoke when |
|---|---|---|
| [Product Steward](product-steward.md) | Scope, non-goals, and adoption order | Expanding product surface or host support |
| [Core Engineer](core-engineer.md) | Room, command, transcript, rendering, and persistence contracts | Changing `muddle-core` |
| [Host Adapter Engineer](host-adapter-engineer.md) | Clean product mounting boundaries | Adding or changing a host adapter |
| [Playtest Designer](playtest-designer.md) | Player vocabulary, readability, and transcript experience | Changing commands, rooms, or client presentation |
| [Validation Gatekeeper](validation-gatekeeper.md) | Deterministic fixtures and compatibility gates | Changing saves, transcripts, adapters, or release claims |

## Core Tensions

| Pulls | Against | Because |
|---|---|---|
| Product Steward | Host Adapter Engineer | A host request is not automatically a reusable MUDDLE capability. |
| Core Engineer | Playtest Designer | A clean shared abstraction can make the playable surface harder to understand. |
| Host Adapter Engineer | Validation Gatekeeper | Fast integration can bypass deterministic transcript and compatibility proof. |
| Playtest Designer | Product Steward | A richer experience can cross the product-neutral boundary. |

## Review Order

1. Validation Gatekeeper establishes reproducible behavior.
2. Product Steward and Core Engineer establish shared ownership.
3. Host Adapter Engineer proves a clean mount contract.
4. Playtest Designer evaluates the resulting player experience.

Product rules, silent fallback, and unverified save or transcript behavior are blocking.
