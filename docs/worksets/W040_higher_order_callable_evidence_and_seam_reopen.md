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

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | |
| 2 | Conformance matrix rows updated? | |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | |
| 4 | Cross-repo impact assessed and handoff filed if needed? | |
| 5 | All required tests pass? | |
| 6 | No known semantic gaps remain in declared scope? | |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | |

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - local deterministic evidence now exists for `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY` at both semantic-plan and runtime level: OxFml consumes their `W044` catalog rows directly, and inline as well as helper-bound local lambdas now execute end-to-end through the typed callable-invoker seam
  - `ISOMITTED` remains unevidenced locally beyond catalog-row intake, and broader higher-order family closure is still open
  - adopted defined-name callable preservation is now stronger and no longer flattened into the helper origin, and higher-order execution through a defined-name callable carrier is now exercised for `MAP`, but broader defined-name callable transport remains open
  - a narrower seam-reopen note to OxFunc is now justified for the minimum callable carrier/provenance split, but not yet for broader higher-order family closure
- claim_confidence: draft
