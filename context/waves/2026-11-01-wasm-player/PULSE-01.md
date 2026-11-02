# Pulse 01 — playable native contracts in an actual WASM worker

The existing seven-crate workspace passes 91 tests, formatting and strict
all-target lint before adapter work. Native core/hosts/CLI/window/Macroquad,
root workspace/lock, RUNE fixtures, policy and product plan remain unchanged
relative to canonical e04fd1c. That preserved commit includes prior native cleanup
beyond documentation; all five matching BANISH consumer tests pass, including
the four required foundation rehearsals. The original BANISH checkout stays
unchanged. Its inherited local patch/lock mismatch failed --locked before
compilation; only an owned rehearsal checkout regenerated the lock offline,
then repeated all five checks with --locked. Exact local paths are evidence,
not portable build dependencies. See portability.json and private retained logs.

The new unpublished browser companion calls the existing Rust host/session/
snapshot/save/transcript APIs. Four adapter tests and strict host/WASM lint pass.
Eighty-three repeated host/WASM outcomes match exactly, including three built-in
paths, checkpoint/replay imports, native save errors and transport refusals.
Eighteen pairs of native CLI save/transcript bytes independently match the adapter.
The first helper failure is retained: an empty command script incorrectly had
one newline, which native parsing treats as look. Correcting it to zero bytes
changes no native behavior or equality assertion. Blank native commands remain
look; the UI explicitly skips blank script lines, and sends typed commands to
the host without CLI quit/exit interception.

Six real browser cases pass: all controls and actual 64 MiB memory-growth refusal,
stateful mock success and failed attempts, both adapter-spike paths, native save/
transcript downloads and checkpoint resume, failed imports retaining accepted
session, authored static native rooms, inert text, exact BOM/CRLF original versus
explicit LF copy, keyboard recall/filtering/mobile and unavailable-WASM routes.
Desktop/mobile captures were visually inspected. Native assets/visual references
stay inert; this text client exposes remaining snapshot/control/visual intent
without claiming Macroquad graphics or product-owned BANISH/AMAZE adoption.

Each request reconstructs/replays a bounded host, avoiding hidden mutable worker
state. Candidate input and accepted session stay distinct. Native save identity
is untagged; native resume replays commands/checks room, then imports checkpoint.
The same accepted transcript is not proof of checkpoint provenance or state.
Those native semantics are visible rather than repaired in browser JavaScript.

Still required: expanded actual lifecycle/resource coverage, named expectations,
complete source/legal artifact, outside-Git source replay, exact final/default
hosted qualification, fresh live proof and separate TRACKER snapshot. A passing
prototype is not a complete release. Only synthetic QA is uploaded at this stage;
compiled distribution waits for complete scoped source/legal packaging.
