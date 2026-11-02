# Product-owned browser snapshot transport

Named consumer: TIGRIS's existing Parliament AI MUDDLE host. Its portable library
foundation is on feat/wasm-parliament-table at 33bcaa0, with 18 original tests
and 14 preserved native controls. The browser needs the already-qualified MUDDLE
snapshot transport while keeping Parliament rules in TIGRIS. The current private
serializer would otherwise require a product-owned copy of MUDDLE's wire shape.

Expose that exact serializer as a documented Rust API, snapshot_json, from the
existing standalone browser/player crate. A default-enabled player-entry feature
keeps the current WASM evaluate export unchanged. Rust consumers can disable
that entry export while reusing serialization; no dynamic host registration,
product policy, new native model, schema conversion or rule interpretation.

## Agent role review (not human signoff)

| Lens | Disposition |
|---|---|
| Validation Gatekeeper | Preserve all native source, workspace lock, 103 native/WASM response controls and 18 original CLI byte pairs. Test an external Rust caller, both feature builds and full current hosted/source/live delivery. |
| Product Steward | This exposes existing product-neutral transport for a concrete existing adopter. It does not mount TIGRIS in MUDDLE's published player or claim product adoption. |
| Host Adapter Engineer | TIGRIS retains host construction, rules and its own numeric checkpoint admission. MUDDLE retains original client snapshot fields, nested controls and visual nodes. No product-owned wire fork or renderer dependence. |
| Playtest Designer | Existing player text/controls/saves stay unchanged; native table interaction remains a separate TIGRIS qualification task. |

Root Cargo files, all crates, native renderers/hosts, RUNE fixtures and policy
surfaces remain frozen at current main 4af0e741 (the native foundation remains
e04fd1c0). The shared helper adds no dependency or default behavior change.
TIGRIS currently pins an older MUDDLE core cf13a86b; any subsequent adoption must
explicitly qualify that dependency change in TIGRIS rather than infer identity.

Qualification: external public-API test; strict host/WASM and no-default checks;
full existing player native/WASM/CLI controls; exact software/legal source/site
verification, real outside-Git replay, hosted/current-main/live checks and exact
served hashes. Only then update the MUDDLE portfolio snapshot and permit the
TIGRIS consumer to pin the pushed helper revision.
