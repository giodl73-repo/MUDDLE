# Static Rust/WASM session player

## Entry and task

Canonical entry e04fd1c0a17ba92ef2ef258a4ebcdb9b9097bc8b, public main
b01df7f8cb8f56e157a50e9e363ed0cdd2213136. The prior PITFALL commit is preserved
on `pitfall/shared-play-surface-20261101`. It includes native cleanup beyond
documentation; native workspace checks and the documented BANISH foundation
rehearsal are required before publication. No additional native changes planned.

Play the existing mock labyrinth, BANISH adapter spike and AMAZE adapter spike
in a static Pages browser with the same Rust host/session/snapshot contracts.
Submit own commands, replay command sequences, import compatible native saves,
export native save/checkpoint and transcript text. An optional authored room
graph mounts through the existing MuddleStaticHost, not JavaScript game rules.
Render shared snapshot/control/visual intent as text with inert asset references;
do not claim Macroquad graphics or full product-owned game integration.

## Contract and scope

An independent unpublished companion workspace calls unchanged native crates.
Each bounded request reconstructs a fresh host, resumes native save if supplied,
then executes the supplied command attempts in order. Failed commands remain
visible and do not silently become successful turns. Saves retain native
checkpoint/replay semantics, including replay-before-checkpoint import, host
validation and untagged host identity. Users explicitly choose the matching host;
an accepted save is not cryptographically bound to a host or room authoring file.
Candidate imports/host changes are atomic: failures retain the accepted session.
Cancellation/restart terminates worker computation and retains committed input.

The three built-in hosts are existing MUDDLE fixtures/spikes, not authoritative
BANISH/AMAZE products. Product rules remain in hosts. No dynamic Rust/plugin
loading, network service, multiplayer, filesystem access, product adoption,
RUNE descriptor change or native persistence-format migration.

## Gates

1. Existing workspace tests/format/lint, RUNE fixture and native host smokes;
   documented four BANISH rehearsal tests for inherited foundation cleanup.
2. New companion tests, strict host/WASM lint and actual bounded WASM worker;
   repeat exact host/WASM responses and compare native CLI save/transcript bytes
   independently for each admitted built-in path.
3. Browser play paths, native error and save/checkpoint rejection, round-trip
   imports/exports, authored room graph, own commands, keyboard/mobile, stale
   read/worker/request fencing, real deadlines/termination, resource maxima.
4. Exact clean-Git source/workspace package, scoped original licenses and
   dependency/Rust notices; budgets/provenance/allowlist/hash inventory.
   Rebuild downloaded source outside Git with paths inside extracted software;
   repeat native/WASM controls and browser behaviors.
5. Read-only exact-head PR qualification, default tested-artifact deployment,
   fresh live functional and every-file hash proof. Child pushed first, separate
   TRACKER pointer/stocktake pulse; preserve owner baseline and other sessions.

## Review routing

Validation Gatekeeper establishes deterministic save/transcript/native parity;
Product Steward and Core Engineer protect foundation/product boundaries;
Host Adapter Engineer reviews explicit static mounting/checkpoint ownership;
Playtest Designer reviews useful commands, clear host identity and session memory.
AI-simulated lenses are software review, not independent human/product approval.

Status: baseline and implementation in progress. No deployment/readiness claim.
