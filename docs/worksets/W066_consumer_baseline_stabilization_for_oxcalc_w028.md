# W066: Consumer Baseline Stabilization For OxCalc W028

## 1. Summary

This workset owns the narrow repo-local stabilization decision needed before OxCalc starts W028 against the current OxFml workspace.

The immediate outcome must be one of:
1. a validated committed OxFml HEAD that OxCalc can target for W028, or
2. an explicit quarantine decision stating that OxCalc W028 must pin to committed `9aca95a` and ignore the current dirty workspace.

## 2. Purpose

The current OxFml working tree has dirty changes touching consumer-facing runtime/replay surfaces, crate exports, evaluator/host behavior, tests, and blocker notes. OxCalc needs a settled consumer-facing baseline before W028 begins.

This workset prevents OxCalc from consuming an ambiguous dirty workspace and keeps broad callable repair out of the baseline-stabilization decision.

## 3. Position And Dependencies

- **Depends on**:
  - `W054_consumer_facing_interface_rearchitecture_and_facade_packaging.md`
  - current committed HEAD `9aca95a`
- **Blocks**:
  - OxCalc W028 baseline selection
- **Cross-repo**:
  - OxCalc must be told the selected baseline: either the validated OxFml commit hash or pin-to-`9aca95a` quarantine decision.

## 4. Scope

In scope:
1. audit dirty files touching:
   - `crates/oxfml_core/src/consumer/runtime/mod.rs`,
   - `crates/oxfml_core/src/consumer/replay/mod.rs`,
   - `crates/oxfml_core/src/lib.rs`,
   - `crates/oxfml_core/src/eval/mod.rs`,
   - `crates/oxfml_core/src/host/mod.rs`,
   - modified tests,
   - `CURRENT_BLOCKERS.md`,
2. classify changes as API-shape, behavior, formatting/test-only, or blocker/documentation,
3. run focused consumer/runtime/replay gates first,
4. decide whether a full `oxfml_core` gate is warranted before any promotion,
5. produce a clear OxCalc-consumable baseline decision.

Out of scope:
1. broad callable repair,
2. resolving `BLK-FML-004` unless it directly invalidates the OxCalc W028 baseline,
3. broad W028 publication/topology work,
4. unrelated semantic changes beyond deciding whether current dirty behavior is promotable or quarantined.

## 5. Deliverables

1. Dirty-file audit and classification.
2. Focused consumer/runtime/replay validation result.
3. Stabilization decision:
   - promote path: validated commit candidate and required remaining gates before commit, or
   - quarantine path: explicit instruction that OxCalc W028 pins `9aca95a`.
4. Assessment of whether `BLK-FML-004` blocks OxCalc W028.

## 6. Gate Model

### Entry Gate
1. Current dirty workspace is preserved for audit.
2. No broad semantic repair is attempted before focused consumer/runtime/replay gates.

### Focused Gate
Run first:
1. `cargo test -p oxfml_core --test runtime_consumer_facade_tests`
2. `cargo test -p oxfml_core --test replay_consumer_facade_tests`

### Promotion Gate For Outcome A
Before a committed OxCalc-consumable HEAD may be promoted:
1. `git diff --check`
2. `cargo fmt --all -- --check`
3. focused consumer/runtime/replay gates pass,
4. touched behavior tests pass,
5. full `cargo test -p oxfml_core` passes if focused gates or audit show behavior changes that could affect shared consumer semantics,
6. only intended files are staged,
7. pinned commit hash is reported to OxCalc.

### Quarantine Gate For Outcome B
If promotion is not safe in this workset:
1. identify the unpromoted dirty files and reason,
2. state that the dirty workspace is not an OxCalc baseline,
3. state that OxCalc W028 must pin current committed HEAD `9aca95a`,
4. keep `BLK-FML-004` classification explicit.

## 7. Current Audit Notes

Initial classification before focused validation:
1. API/facade files dirty:
   - `consumer/runtime/mod.rs`,
   - `consumer/replay/mod.rs`,
   - `lib.rs`,
2. behavior files dirty:
   - `eval/mod.rs`,
   - `host/mod.rs`,
3. evidence/tests dirty:
   - evaluator, host, runtime/replay facade, FTC, W047, W049 test files,
4. blocker/documentation dirty:
   - `CURRENT_BLOCKERS.md`, including active `BLK-FML-004`.

Focused validation result for the initial W066 pass:
1. `cargo test -p oxfml_core --test runtime_consumer_facade_tests`: passed, 41 passed.
2. `cargo test -p oxfml_core --test replay_consumer_facade_tests`: failed, 13 passed / 1 failed.
   - failing row: `replay_projection_service_emits_effective_display_text_for_programmatic_verification_cases`
   - observed mismatch: `FTC-0703 runtime projection view count`, expected `Some(3)`, observed `Some(4)`.
3. `git diff --check`: passed with line-ending warnings only.

Current stabilization decision:
1. Outcome A is not promotable in this pass because the focused replay consumer gate failed.
2. Outcome B is active for OxCalc W028 unless and until a later W066 pass repairs and validates the dirty replay baseline: OxCalc W028 should pin committed HEAD `9aca95a` and ignore the dirty OxFml workspace.
3. A full `cargo test -p oxfml_core` gate is not warranted before this quarantine decision because the narrower required consumer/replay gate already failed.
4. No broad callable repair is authorized or required for this baseline decision.

`BLK-FML-004` classification for W028:
1. `BLK-FML-004` does not block OxCalc W028 by default.
2. It only becomes W028-blocking if W028 specifically depends on exact `FTC-0902` `row(...)` built-in-colliding callable witnesses.
3. For ordinary `OxFml_V1` consumer runtime/replay facade consumption, it remains a watch lane rather than the cause of the W028 quarantine decision.

## 8. OxCalc W028 Consumer Verification List

Baseline recommendation remains quarantine-to-committed-HEAD because the dirty replay consumer gate failed. Against OxCalc's intake list:

1. Required public modules and facade symbols:
   - `RuntimeEnvironment`, `RuntimeFormulaRequest`, `RuntimeFormulaResult`: pass on committed `9aca95a`; public-line audit shows no dirty API-line delta.
   - `ReplayProjectionRequest`, `ReplayProjectionResult`, `ReplayProjectionService`: pass on committed `9aca95a`; public-line audit shows no dirty API-line delta.
2. Supporting source/binding/eval/interface/semantics/format symbols currently used by OxCalc:
   - pass for committed `9aca95a` as the only recommended W028 baseline.
   - dirty workspace is not promoted because behavior and tests are changed and replay validation failed.
3. OxFunc value/provider ownership:
   - pass; no W066 baseline decision moves OxFunc-owned value/provider types into OxFml.
4. Runtime result access:
   - diagnostics/trace events: pass through `RuntimeFormulaResult::{syntax_diagnostics, bind_diagnostics, trace_events}`.
   - returned value surface and typed host/provider outcomes: pass through `returned_value_surface`, `execution_outcome_surface`, `typed_query_bundle_spec`, and evaluation/provider carriers.
   - published/worksheet semantic value: pass through `published_worksheet_value`.
   - candidate id/value/consequence categories: pass through `candidate_result` and seam delta/effect fields.
   - commit/accept decision and fence refs: pass through `commit_decision`, managed commit results, and seam `FenceSnapshot` carriers.
   - reject record/code/no-publish distinction: pass through managed rejection result `reject_record` and seam `RejectRecord` / `RejectCode`; behavior expectation remains reject = deterministic no-publish.
   - formula/token/structure/bind/profile/library-context identity: pass through source/semantic-plan/execution-contract/library-context snapshot refs and seam identity fields.
   - replay/correlation handles: partial/pass for current floor through candidate ids, trace correlation ids, session ids, and replay projection metadata; broader W026/W028 correlation breadth remains canonical-but-narrower.
5. Behavior expectations:
   - evaluator success is not coordinator publication: pass in contract; OxCalc still owns coordinator publication policy above candidate/commit separation.
   - reject deterministic no-publish: pass in contract.
   - candidate/commit/reject identities distinct: pass for current seam floor.
   - value/topology/shape/effect consequences distinguishable: pass for current floor, with broader publication/topology breadth still canonical-but-narrower.
6. Runtime-effect families:
   - `DynamicDependency`, `ExecutionRestriction`, `CapabilitySensitive`, `ShapeTopology`: pass as preserved categories for the committed baseline/current contract, with `CapabilitySensitive` admitted but not necessarily exercised in every OxCalc first-floor path.
7. Dependency/topology facts:
   - static/resolved/unresolved/host-sensitive/dynamic-potential refs and surfaced dependency/topology/shape changes: partial/pass for current committed floor; broader retained/reduced projection breadth remains open-lane rather than W028-start blocker.
8. Replay projection:
   - source metadata: pass.
   - candidate/commit/reject ids: partial/pass; candidate and commit decision are surfaced, reject is surfaced through managed/reject paths rather than every projection family.
   - fence/library refs: partial/pass; library refs are explicit, fence refs are present in managed/session seam carriers and not universal in every projection result.
   - comparison views: dirty workspace failed focused replay validation (`FTC-0703` view count mismatch), so dirty workspace fails; committed `9aca95a` remains the recommended baseline.
   - trace/replay sidecar refs: partial/pass for current floor via trace event kinds, first-host capture packet, and replay metadata; broader sidecar breadth remains open-lane.

Net result:
1. Committed `9aca95a` is the recommended OxCalc W028 baseline.
2. Dirty workspace fails the OxCalc-consumer verification list because focused replay projection validation failed.
3. Do not promote or consume dirty W066-local state for W028.

## 9. Status

- `execution_state`: `in_progress`
- `scope_completeness`: `scope_partial`
- `target_completeness`: `target_partial`
- `integration_completeness`: `partial`
- `open_lanes`:
  - optional later repair of the dirty replay consumer projection mismatch
  - full promote path remains closed until focused replay consumer gate passes
  - OxCalc baseline notification: pin `9aca95a` for W028 under the current quarantine decision
- `claim_confidence`: `provisional`
