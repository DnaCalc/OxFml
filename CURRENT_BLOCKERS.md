# CURRENT_BLOCKERS.md — OxFml

Status: 2 active blockers.

Last reviewed: 2026-05-04 after DNA OneCalc locale and custom-format grammar handoff intake.

---

## Active Blockers

### BLK-FML-004: `FTC-0902` exact reduced `row(...)` witnesses currently collapse into the existing built-in-collision frontier

- **Status**: active
- **Impact**: blocks any safe OxFml patch for the current exact `FTC-0902` reduced witness set
- **Current state**: repo-local probes show the retained blocked witnesses use LET binder name `row`, which collides with built-in `ROW`; the exact blocked forms fail locally, but the same self-application / returned-lambda forms succeed immediately when renamed to a non-colliding helper such as `grow`, so the current evidence does not support a new generic higher-order callable patch
- **Exact unblock steps**: obtain one of: (a) a non-built-in exact witness that still fails, or (b) authoritative host evidence that built-in-colliding `row` should shadow `ROW` in this callable self-application lane despite the existing `FTC-0443/0444` collision frontier
- **Recommendation**: wait

---

### BLK-FML-005: Locale expansion requires OxFunc locale-profile API breadth

- **Status**: active
- **Impact**: blocks the locale-specific portions of DNA OneCalc `HANDOFF_OXFML_LOCALE_EXPANSION.md` and the locale-prefix portion of `HANDOFF_OXFML_CUSTOM_FORMAT_GRAMMAR.md`
- **Current state**: OxFml can currently construct only `LocaleProfileId::EnUs` and `LocaleProfileId::CurrentExcelHost` profiles through the OxFunc locale-format seam. The requested locale-keyed month/weekday names, separators, currency symbols, parser branches, General rendering, and optional locale-prefix parsing need canonical OxFunc `LocaleProfileId` variants and profile constants before OxFml can avoid inventing a second locale registry.
- **Exact unblock steps**: OxFunc exposes canonical locale profile identities and format-profile constants for the requested locale set, including decimal/thousands/list separators, currency symbol, date separator, time separator, and date-system compatibility expectations. After that lands, OxFml can add locale-keyed parser/formatter tables and locale-prefix custom-format grammar coverage without violating ownership.
- **Recommendation**: escalate
- **Opened**: 2026-05-04

---

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
