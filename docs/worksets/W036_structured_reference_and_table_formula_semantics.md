# W036: Structured Reference and Table Formula Semantics

## Purpose
Realize the structured-reference and table-formula lanes classified by `W031` so OxFml has an explicit parser, binder, evaluator, and proving-host floor for table-aware formula semantics rather than only a provisional acceptance rule.

## Position and Dependencies
- **Depends on**: `W031`, `W032`
- **Blocks**: `W038`, later broader table/host-policy work
- **Cross-repo**: OxFml owns grammar, bind, runtime, and seam-significant table semantics; OxCalc consumes resulting effects and OxFunc consumes prepared-call/reference consequences

## Scope
### In scope
1. Broaden structured-reference grammar and qualifier coverage.
2. Define table-context-sensitive bind and normalized reference shapes.
3. Exercise local evaluator/runtime behavior for the first nontrivial structured-reference families.
4. Classify formula-significant table surfaces that affect bind meaning, admission, or seam-significant effects.
5. Add deterministic replay/proving artifacts for the widened structured-reference floor.

### Out of scope
1. Every Excel table feature.
2. UI-only table styling.
3. Broad conditional-formatting or data-validation sublanguage work.

## Deliverables
1. Narrower canonical structured-reference and table-formula semantics in OxFml docs.
2. Wider parser/binder/evaluator coverage with deterministic replay evidence.
3. Explicit residual list for remaining table lanes.

## Gate Model
### Entry gate
- `W031` has classified the structured-reference lane as partial rather than implicitly complete.
- `W032` has kept provider/catalog pressure from distorting reference semantics.

### Exit gate
- Structured references are no longer only a provisional parse-acceptance lane.
- Table-context-sensitive bind meaning is canonical and replay-backed for the exercised local floor.
- Remaining table semantics are explicitly listed rather than implied.

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
- execution_state: planned
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes: richer totals-row semantics, broader table metadata carriers, and wider host-policy consequences remain outside this workset scope
- claim_confidence: draft
