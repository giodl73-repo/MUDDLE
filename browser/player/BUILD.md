# Build the static session player

Use Rust 1.95.0 with wasm32-unknown-unknown, wasm-bindgen CLI 0.2.127,
Python 3.11+ and Node 22 / Playwright 1.61.1. The standalone unpublished
browser/player workspace depends on unchanged sibling native crates and their
pinned RUNE Git dependencies, never on TRACKER paths.

`python browser/player/build-prototype.py --bindgen <absolute bindgen path>`
builds native/WASM controls and stages the prototype under
`target/player-prototype/MUDDLE`. Use an absolute CARGO_TARGET_DIR for an external
shared build cache. Serve target/player-prototype and open /MUDDLE/.

This candidate is not yet a qualified source/legal release or deployed artifact.
Complete package/rebuild/browser/hosted/live gates are recorded in its wave.
