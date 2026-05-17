# Pulse 02 - Role Contracts and Plan Review

## Status

Done.

## Goal

Create MUDDLE role contracts and review the product plan before expanding core
engine scope.

## Work

- Add `.roles/` with product, core, adapter, playtest, and validation ownership.
- Review `PRODUCT_PLAN.md` against the MUDDLE/RALLY/product boundary.
- Prioritize host adapter proof before larger renderer or persistence work.
- Keep first-wave validation anchored on deterministic local tests.

## Decision

MUDDLE should next prove a minimal host adapter seam with an in-repo fixture
host. ASCII maps and save/resume should build on that seam instead of expanding
as isolated engine features.

## Validation

```powershell
cargo test --quiet
```
