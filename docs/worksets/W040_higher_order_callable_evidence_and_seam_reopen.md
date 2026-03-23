# W040: Higher-Order Callable Evidence and Seam Reopen

## Purpose
Collect local OxFml evidence for higher-order callable and helper lanes that are currently visible in OxFunc but not yet exercised locally in OxFml, so the OxFml/OxFunc callable seam can be reopened on evidence rather than note pressure alone.

## Position and Dependencies
- **Depends on**: `W032`, `W038`
- **Blocks**: future callable seam-lock follow-up after `W032`
- **Cross-repo**: OxFml remains authoritative for helper syntax, lexical scope, and callable preservation; OxFunc remains authoritative for callable semantic value behavior across the shared seam

## Scope
### In scope
1. Add local OxFml evidence for higher-order helper lanes such as `MAP`, `REDUCE`, and `SCAN`.
2. Decide whether additional lanes such as `BYROW`, `BYCOL`, `MAKEARRAY`, and `ISOMITTED` are admissible for the next local callable-evidence floor.
3. Determine whether higher-order callable lanes force changes to the minimum callable carrier, invocation boundary, or provenance split.
4. Add deterministic replay/proving artifacts for any newly admitted higher-order callable lane.
5. Reopen the OxFml/OxFunc callable seam only where the new local evidence materially changes the current `LET` / `LAMBDA` prep note.

### Out of scope
1. Final UDF or product callable ABI.
2. Final worksheet publication policy for callable values.
3. Coordinator-visible callable consequences unless they arise from newly exercised local evidence.
4. Broad function-family implementation work not needed to produce local callable seam evidence.

## Deliverables
1. A local OxFml evidence floor for at least one higher-order callable family beyond the current `LET` / `LAMBDA` baseline.
2. Deterministic replay/proving artifacts for that higher-order callable floor.
3. An explicit decision on whether the current callable carrier and invocation boundary remain sufficient.
4. A narrower seam-reopen note only if the new evidence actually changes the current callable boundary posture.

## Gate Model
### Entry gate
- `W032` has narrowed the first callable carrier/provenance split as far as current local evidence honestly permits.
- `W038` has kept name/external-name callable carrier pressure explicit rather than implicit.

### Exit gate
- At least one higher-order callable lane beyond the current `LET` / `LAMBDA` floor is exercised locally with deterministic evidence.
- Any effect on callable carrier, provenance split, or invocation boundary is stated explicitly.
- If no boundary change is forced, the workset closes with an explicit “no seam reopen needed yet” outcome.

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: none
- claim_confidence: validated

## Closure Reading
1. Local deterministic evidence now exists for `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY` at both semantic-plan and runtime level: OxFml consumes their `W044` catalog rows directly, and inline as well as helper-bound local lambdas execute end-to-end through the typed callable-invoker seam.
2. Local deterministic evidence now also exists for the currently honest `ISOMITTED` floor:
   - direct present-argument `ISOMITTED(1) -> FALSE`,
   - present lambda-parameter `LAMBDA(a,ISOMITTED(a))(3) -> FALSE`,
   - higher-order present-argument `MAP(SEQUENCE(2),LAMBDA(a,ISOMITTED(a))) -> {FALSE;FALSE}`,
   - direct lambda under-application remains a distinct arity-mismatch failure rather than an omitted-placeholder lane.
3. Adopted defined-name callable preservation is stronger and no longer flattened into the helper origin, and higher-order execution through a defined-name callable carrier is exercised for `MAP`.
4. This wider evidence does not force a broader seam reopen. The remaining callable narrowing is the smaller carrier-versus-provenance freeze already owned by `W032`, `W041`, `W042`, and `W043`.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | yes |
| 2 | Conformance matrix rows updated? | yes |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes |
| 4 | Cross-repo impact assessed and handoff filed if needed? | yes |
| 5 | All required tests pass? | yes |
| 6 | No known semantic gaps remain in declared scope? | yes |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | yes |
