# CURRENT_BLOCKERS.md — OxFml

Status: no active blockers.

Last reviewed: 2026-04-01 after `W054` Epic E1 editor-facade refactor validation.

---

## Active Blockers

(none)

---

## Resolved Blockers

### BLK-FML-002: OxFunc `call_register_id_family` derive regression blocked 2026-03-22 validation

- **Status**: resolved
- **Impact**: blocked `W042` validation because `cargo test -p oxfml_core` could not compile the sibling `oxfunc_core` dependency
- **Current state**: OxFunc carried `Eq` derives on types containing `f64` in `../OxFunc/crates/oxfunc_core/src/functions/call_register_id_family.rs`; the minimal sibling unblock was to drop those `Eq` derives and rerun OxFml validation
- **Exact unblock steps**: completed; patched the sibling derive regression, reran focused OxFml tests, and validation resumed
- **Recommendation**: workaround
- **Opened**: 2026-03-22
- **Resolved**: 2026-03-22

### BLK-FML-001: OxFunc sibling compile failure blocks OxFml validation

- **Status**: resolved
- **Impact**: blocked `W004`, `W009`, and `W010` gate closure because required `cargo test -p oxfml_core` validation could not complete
- **Current state**: subsequent rerun of `cargo test -p oxfml_core` completed successfully after the sibling compile surface recovered
- **Exact unblock steps**: completed; rerun validation succeeded
- **Recommendation**: workaround
- **Opened**: 2026-03-16
- **Resolved**: 2026-03-16

### BLK-FML-003: Rust compiler ICE briefly interrupted full `cargo test -p oxfml_core` validation

- **Status**: resolved
- **Impact**: temporarily interrupted full-suite validation during `W054` Epic E1
- **Current state**: a later serial rerun of `cargo test -p oxfml_core` completed successfully with the E1 slice in place
- **Exact unblock steps**: completed; reran the full suite after the focused editor/refactor passes finished and the transient compiler failure did not recur
- **Recommendation**: workaround
- **Opened**: 2026-04-01
- **Resolved**: 2026-04-01

---

## Entry Template

```
### BLK-FML-NNN: <title>

- **Status**: active | resolved | closed
- **Impact**: <which worksets/features are blocked>
- **Current state**: <what has been attempted, what failed>
- **Exact unblock steps**: <specific actions needed>
- **Recommendation**: wait | escalate | workaround
- **Opened**: YYYY-MM-DD
- **Resolved**: YYYY-MM-DD (if applicable)
```
