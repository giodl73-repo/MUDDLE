# Pulse 02 — accepted session survives browser failures

Prototype candidate 1c4cfaaa875e1b7c50b44b3bccc20bb33ddceb8b passes hosted
player run 41409738142 and both inherited native Rust-gate runs (41409738080,
41409731951). Downloaded prototype QA independently confirms six browser checks,
83 exact native/WASM responses and eighteen original CLI save/transcript pairs.
No compiled binary/site is distributed by that artifact.

The expanded local suite passes sixteen cases without skipped, flaky or retried
outcomes. Actual 12-second startup and 10-second computation deadlines terminate
the worker and retain accepted session/request exports. Five-second deadlines
for both file slots preserve prior original/editor inputs; invalid UTF-8 does
likewise. Constructor failure, crash/restart, malformed reports with matching
request hash, CPU termination on candidate input/cancel, wrong IDs/generations,
queued ready and late terminated-worker messages have explicit lifecycle probes.
These synthetic fault workers are not game results or native control oracles.

Independent slot reads survive the other slot's input, while editor/example
intent invalidates older file completion. Whole-file byte bounds and 129-command
refusal retain accepted session and exports. Earlier six play/parity/input/export/
keyboard/mobile/failure-route cases remain in the same suite.

One local test failure (15 passed, one failed) is retained with result/context:
the example-after-file check used textarea textContent instead of its value.
The assertion now checks the actual textarea value for the same expected example;
the invalid-UTF8 checks also wait for a decoding-specific message. No runtime,
native behavior, time/resource bounds or intended expected outcome changed.

Remaining release gates are unchanged: named boundary/max-input controls and
complete admitted-maximum browser evidence; complete corresponding software and
original/dependency/Rust notices with budgets and source/artifact hashes;
independent outside-Git source rebuild/replay; final exact-head/default deploy
and fresh live proof; separate TRACKER snapshot. This is still a draft candidate,
not a complete source/legal or live release. Native Macroquad advisory remains
open and excluded from the browser dependency closure as recorded in Pulse 01.
