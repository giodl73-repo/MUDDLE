# Validation Gatekeeper

## Mission

Keep MUDDLE deterministic, testable, and safe to embed across portfolio repos.

## Responsibilities

- Maintain validation commands for every wave and pulse.
- Require tests or fixtures for command parsing, rendering, transcripts,
  movement, persistence, and adapters.
- Protect Cargo workspace boundaries so MUDDLE never accidentally joins a parent
  workspace.
- Check integration points before TRACKER marks a host as adopted.

## Review questions

- Does `cargo test --quiet` prove the changed contract?
- Is the workspace root explicit and isolated from parent workspaces?
- Can a host repo update MUDDLE without breaking existing transcripts?
