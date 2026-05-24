# CURRENT_BLOCKERS.md — OxFml

Status: 1 active blocker.

Last reviewed: 2026-05-24 after non-table W074 oracle expansion and OxFml validation.

---

## Active Blockers

### BLK-FML-004: `FTC-0902` exact reduced `row(...)` witnesses currently collapse into the existing built-in-collision frontier

- **Status**: active
- **Impact**: blocks any safe OxFml patch for the current exact `FTC-0902` reduced witness set
- **Current state**: repo-local probes show the retained blocked witnesses use LET binder name `row`, which collides with built-in `ROW`; the exact blocked forms fail locally, but the same self-application / returned-lambda forms succeed immediately when renamed to a non-colliding helper such as `grow`, so the current evidence does not support a new generic higher-order callable patch
- **Exact unblock steps**: obtain one of: (a) a non-built-in exact witness that still fails, or (b) authoritative host evidence that built-in-colliding `row` should shadow `ROW` in this callable self-application lane despite the existing `FTC-0443/0444` collision frontier
- **Recommendation**: wait

---

## Resolved Blockers

### BLK-FML-008: W074 name/call freeze for W051/W056 host names

- **Status**: resolved for the W051/W056 host-name mapping rule
- **Impact**: no longer blocks OxCalc from mapping TreeCalc host names to the defined-name lane and lambda-valued host nodes to the defined-name-`LAMBDA` lane; OxCalc still needs to consume the handoff and exercise the path under W056
- **Current state**: Excel COM 16.0 black-box probes now cover the required current W074/CALC-005 row set for W051/W056 host-name mapping, including built-in versus defined-name, UDF versus defined-name, sheet versus workbook defined-name, defined-name `LAMBDA`, lexical locals, late UDF registration, UDF removal, defined-name mutation, table-name rows, and the 2026-05-24 non-table expansion plus callable lexical `SUM` frontier. Runtime/replay evidence covers explicit host references, host namespace version identity, product-neutral bare host-name packets mapped through the defined-name / defined-name-`LAMBDA` evaluator lane, registry/capability runtime formula-call identity, and DnaOneCalc no-host lexical guardrails. Final handoff is recorded in `docs/handoffs/HANDOFF_CALC_005_W074_NAME_CALL_FREEZE.md`.
- **Exact unblock steps**: completed for W051/W056 name/call mapping. Remaining W074 work moves to ordinary open lanes: broader bind/editor cache migration, future product-specific name-world extensions if admitted, and W036 structured-reference grammar/table semantics beyond the W056 packet slice.
- **Recommendation**: resolved
- **Opened**: 2026-05-22
- **Resolved**: 2026-05-24

### BLK-FML-009: OxFunc sibling compile failure blocks fml-ds0.15 OxFml validation

- **Status**: resolved
- **Impact**: previously blocked focused and full OxFml validation for `fml-ds0.15` / W074-W056 zero-row structured-table packet support because `cargo test -p oxfml_core ...` failed while compiling sibling `oxfunc_core`.
- **Current state**: OxFunc commit `8216511` repaired the sibling structured-table ReferenceLike compile surface. OxFml validation then completed successfully, including the zero-row structured-reference packet cases and full `cargo test -p oxfml_core`.
- **Exact unblock steps**: completed; reran full OxFml validation after the OxFunc unblock.
- **Recommendation**: resolved
- **Opened**: 2026-05-23
- **Resolved**: 2026-05-23

### BLK-FML-007: OxFunc registry compile failure blocks OxFml validation

- **Status**: resolved
- **Impact**: previously blocked full `cargo test -p oxfml_core` validation for the W074 docs/spec tranche because the build failed in sibling `oxfunc_core` before OxFml tests executed
- **Current state**: OxFunc W093 now provides the missing registry symbols. A follow-up OxFml helper-slot fix also cleared the prior full OxFunc integration panic in `ftc_0907_and_map_true_array_scalarizes_to_true_through_adapter`.
- **Exact unblock steps**: completed; reran `cargo test -p oxfml_core` from OxFml and `cargo test --manifest-path crates\oxfunc_core\Cargo.toml` from OxFunc successfully.
- **Recommendation**: resolved
- **Opened**: 2026-05-22
- **Resolved**: 2026-05-22

### BLK-FML-005: Locale expansion requires final OxFunc FormatProfile semantics

- **Status**: resolved
- **Impact**: previously blocked the date-parser, currency-placement, format-code-token-policy, and locale-prefix portions of DNA OneCalc `HANDOFF_OXFML_LOCALE_EXPANSION.md` and `HANDOFF_OXFML_CUSTOM_FORMAT_GRAMMAR.md`
- **Current state**: OxFunc W094 exposes the needed profile semantics: short-date order/pattern, currency placement/spacing/negative pattern, invariant format-code token fields, and `LocaleProfileId::from_excel_lcid(lcid)`. OxFml consumes those fields for locale-keyed date names, profile-driven short-date parsing, currency parsing/rendering, invariant custom-format numeric tokens, and `[$-LCID]` locale-prefix custom-format rendering. Existing localized `TEXT(...)` separator-context fixtures now express their token policy explicitly through the profile fields rather than relying on separator inference.
- **Exact unblock steps**: resolved; consumed the OxFunc final `FormatProfile` surface and reran focused OxFml locale/custom-format evidence.
- **Recommendation**: resolved
- **Opened**: 2026-05-04
- **Resolved**: 2026-05-06

---

### BLK-FML-006: Lambda `invoke_many` batching requires OxFunc callable-helper trait and loop coordination

- **Status**: resolved
- **Impact**: previously blocked OxFml bead `fml-0wg.6` / DnaOneCalc `HANDOFF_OXFML_LAMBDA_INVOCATION_PERF.md`
- **Current state**: OxFunc W095 added the owned `CallableInvoker::invoke_many(...)` seam plus `CallableInvocationBatch` / `CallableBatchMode` and wired `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY` through it. OxFml now specializes `OxFmlCallableInvoker::invoke_many(...)` for local callables, hoisting registry lookup, binding clone, resolver setup, trace buffer reuse, and local helper-binding slot setup out of the per-iteration path while preserving per-invocation arity and recursion checks.
- **Exact unblock steps**: completed; consumed the OxFunc W095 seam and reran focused higher-order callable evidence plus full `oxfml_core` validation.
- **Recommendation**: resolved
- **Opened**: 2026-05-06
- **Resolved**: 2026-05-06

---

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
