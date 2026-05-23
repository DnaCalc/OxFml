# CURRENT_BLOCKERS.md — OxFml

Status: 3 active blockers.

Last reviewed: 2026-05-23 during fml-ds0.15 zero-row structured-table packet validation.

---

## Active Blockers

### BLK-FML-009: OxFunc sibling compile failure blocks fml-ds0.15 OxFml validation

- **Status**: active
- **Impact**: blocks focused and full OxFml validation for `fml-ds0.15` / W074-W056 zero-row structured-table packet support because `cargo test -p oxfml_core ...` fails while compiling sibling `oxfunc_core`
- **Current state**: OxFml zero-row packet implementation and focused tests were drafted, and `cargo fmt` completed. Focused `cargo test -p oxfml_core --test structured_reference_tests`, `cargo test -p oxfml_core --test runtime_consumer_facade_tests zero_row`, and `cargo test -p oxfml_core --test replay_consumer_facade_tests zero_row` all stopped before OxFml test execution due to sibling OxFunc compile errors in `surface_dispatch_by_index_generated.rs`, `columns_fn.rs`, `rows_fn.rs`, and `resolver.rs`.
- **Exact unblock steps**: repair or revert the sibling OxFunc compile regression in an OxFunc-scoped run, then rerun the three focused OxFml commands plus the required broader `cargo test -p oxfml_core` validation before closing `fml-ds0.15`.
- **Recommendation**: wait
- **Opened**: 2026-05-23

### BLK-FML-008: W074 name/call freeze still lacks full oracle and host-extension evidence

- **Status**: active
- **Impact**: blocks freezing the generic W074/CALC-005 name/call precedence rule for product host namespaces; does not block the current generic runtime/replay host-reference pass-through or DNA OneCalc lexical guardrail
- **Current state**: Excel COM 16.0 black-box probes on 2026-05-22 populated selected W074 matrix rows for built-in versus defined-name, UDF versus defined-name, sheet versus workbook defined-name, defined-name `LAMBDA`, lexical locals, late UDF registration, UDF removal, and defined-name/table-name collisions. The explicit host-reference row now has deterministic non-Excel OxCalc/OxFml runtime/replay evidence for `resolution_layer=explicit_host_ref`, source token/span, opaque selector, and prepared identity inputs. A narrow OxFml runtime/replay slice also proves that opt-in generic host formula context carries `host_namespace_version` through prepared identity and replay projection, and changing it changes the prepared key even when there are no explicit host-reference bind results. Bare host-name runtime evidence now admits a product-neutral `RuntimeHostNameBinding` packet mapped to the existing defined-name / defined-name-`LAMBDA` evaluator lane, keeps it separate from explicit host references, preserves source token/span and replay diagnostics, and includes host-name bind results plus host namespace/caller context in prepared identity and replay. This supports TreeCalc's defined-name-lane mapping but still does not freeze broader built-in/UDF/defined-name shadowing precedence. Capability-overlay denial now has focused OxFml/OxFunc registry/editor evidence plus runtime formula-call evidence: denied entries remain registry-present, runtime formula-call path blocks the denied call, capability overlay identity and denials change prepared identity, and replay projection carries registry snapshot/capability-denial identity. Runtime UDF registry-view admission now distinguishes registered UDF calls from unknown functions without implementing UDF invocation, changes prepared identity with registration, returns to `#NAME?`-style unknown classification after unregister/default registry, changes prepared identity on that transition, and preserves registry snapshot identity in replay projection. Registered-external reconciliation now has current W093/W052 agreement: descriptor-only `REGISTER.ID` / `CALL` mutation stays adjacent registered-external state and does not create bind-visible ordinary UDF entries without friendly worksheet-visible metadata. Table-context evidence now covers the W056 table-adjacent residuals: 2026-05-22 probes observed the workbook defined-name/table-name collision row, and 2026-05-23 Excel COM 16.0 build 20026 probes add table-only bare/call classification, sheet-defined-name/table-name collision, table/column rename formula rewrite, and table/UDF collision evidence. Local OxFml runtime/replay tests prove the generic table-context prepared identity inputs for table id, table range, row membership/order, exact header/totals refs, selected column id/ordinal/range, enclosing table, caller row, and unrelated catalog mutation without TreeCalc semantics.
- **Exact unblock steps**: execute the remaining residual W074 beads under `fml-ds0.6`: `fml-ds0.6.4` for non-table Excel oracle expansion across broader workbook/sheet/UDF/defined-name/LAMBDA combinations, then `fml-ds0.6.5` as the freeze audit and OxCalc handoff. Only after those beads produce observed evidence or exact typed blockers may the name/call rule move beyond provisional. Broader full structured-reference grammar/table semantics remain W036 work rather than this blocker.
- **Recommendation**: wait
- **Opened**: 2026-05-22

### BLK-FML-004: `FTC-0902` exact reduced `row(...)` witnesses currently collapse into the existing built-in-collision frontier

- **Status**: active
- **Impact**: blocks any safe OxFml patch for the current exact `FTC-0902` reduced witness set
- **Current state**: repo-local probes show the retained blocked witnesses use LET binder name `row`, which collides with built-in `ROW`; the exact blocked forms fail locally, but the same self-application / returned-lambda forms succeed immediately when renamed to a non-colliding helper such as `grow`, so the current evidence does not support a new generic higher-order callable patch
- **Exact unblock steps**: obtain one of: (a) a non-built-in exact witness that still fails, or (b) authoritative host evidence that built-in-colliding `row` should shadow `ROW` in this callable self-application lane despite the existing `FTC-0443/0444` collision frontier
- **Recommendation**: wait

---

## Resolved Blockers

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
