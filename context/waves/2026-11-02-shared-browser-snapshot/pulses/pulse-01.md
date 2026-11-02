# Pulse 01: shared transport boundary

Scope entry before implementation: only browser/player's serializer visibility,
its documented Rust consumer entry-feature boundary, a consumer compilation/
transport test, and the software-source qualification record. Preserve all
original fields, nested controls/visual nodes, native sources and existing player
behavior. Existing wave roles establish ownership; no TIGRIS game policy enters
MUDDLE. Native/default/source/live gates are pending.

Implementation: snapshot_json is the original serializer with public visibility
and consumer documentation. The default-enabled player-entry gates only the
standalone evaluate WASM export/annotation; existing Rust evaluate remains
testable. An external integration test compares the public serializer with the
existing player over a native mock session. All five adapter tests pass with
and without default features; strict host and no-default WASM lint pass. A real
WASM build with defaults disabled exports only memory/data/heap markers, with
no evaluate entry. Native source/policy/lock guard remains green.

Full 103-response/18-native-CLI controls, hosted/source/default/live verification
remain pending. No TIGRIS adoption or new MUDDLE hosted product is claimed.
