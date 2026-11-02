# Build the static session player

Use Rust 1.95.0 with wasm32-unknown-unknown and rust-docs, wasm-bindgen CLI
0.2.127, Python 3.11+ and Node 22 / Playwright 1.61.1. The standalone unpublished
browser/player workspace depends on unchanged sibling native crates and pinned
RUNE Git dependencies, never TRACKER paths. Dependencies need network access;
they are not vendored. An absolute CARGO_TARGET_DIR selects a shared build cache.

From the source or full checkout root:

```powershell
python browser/player/build-prototype.py --bindgen C:/path/to/wasm-bindgen.exe
cd browser/player
npm ci
npx playwright install chromium
$env:MUDDLE_SITE_DIR='../../target/player-prototype'
npm run test:pages -- --grep-invert "packaged delivery|source/legal"
```

This builds native/WASM controls and stages only browser runtime/synthetic room
example and docs in target/player-prototype/MUDDLE. All 103 target responses are
repeated; eighteen original CLI save/transcript pairs independently match.
Native workspace tests and companion tests run normally; retained RUNE software
fixture is included. The native Macroquad CLI smokes require a display/Xvfb on
Linux; normal unit tests do not open a graphical window.

Rust host consumers may call `muddle_browser_player::snapshot_json` on a native
`MuddleClientSnapshot`. This is the same mapping used by the published player;
it does not validate state, fetch references or run host rules. Use
`default-features = false` when embedding the crate to disable the default
`player-entry` WASM export. All shared MUDDLE packages must resolve to one pinned
Git revision; consumers qualify any native dependency adoption independently.

Release maintainers use a clean Git checkout: verify-foundation.py, prototype
build, package-site.py, verify-site.py <site> --commit <full HEAD>, then full
nineteen browser cases with MUDDLE_SITE_DIR unset against target/player-release.
Finally verify-source.py --site target/player-release/MUDDLE --bindgen <path>
rebuilds outside Git in a path with spaces, verifies first-party manifest paths,
repeats 91 native/five adapter tests (default and no-default adapter matrices), both 103-response sets, eighteen original
CLI byte pairs and seventeen behavior cases. Source/legal delivery tests require
the complete packaged site. Cross-directory/platform binary identity is not
claimed; hashes are recorded separately. Complete source/notices/budgets/hashes
must qualify before binary artifact upload; default deployment uses that artifact.

Source scope and original grants are in SOURCE-SCOPE.md. Native Macroquad's
existing advisory stays open and is absent from the browser dependency closure.
