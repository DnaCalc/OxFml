# W026: Library-Context Snapshot and Availability Taxonomy

## Purpose
Narrow the current OxFml/OxFunc seam around the minimum external library-context snapshot and the stage-aware availability taxonomy so parse, bind, semantic planning, runtime capability, and provider-failure paths stop depending on broad provisional wording.

## Position and Dependencies
- **Depends on**: `W020`, `W024`
- **Blocks**: `W027`, `W028`, `W030`
- **Cross-repo**: OxFunc remains the semantic owner of catalog truth; OxFml remains the owner of parser/binder/evaluator-facing transport, early-admission meaning, and replay-preserved staging

## Scope
### In scope
1. Define the minimum honest shared library-context snapshot field set for OxFml consumption.
2. Narrow the stage-aware availability taxonomy across parse/bind, semantic planning, runtime capability, and post-dispatch/provider-failure paths.
3. Clarify which availability and provider states belong in library context versus host capability view versus runtime results.
4. Add deterministic proving artifacts for the narrowed library-context and availability distinctions.

### Out of scope
1. Final callable-value carrier lock.
2. Full OxFunc catalog closure.
3. OxCalc-facing seam changes unless coordinator-visible consequences are actually introduced.
4. Broad grammar closure beyond what the library-context and availability taxonomy requires.

## Deliverables
1. A narrower canonical OxFml/OxFunc seam around library-context snapshot truth.
2. A stage-aware availability/provider taxonomy that is exercised and replay-visible.
3. Updated outbound OxFunc note and canonical docs that reduce current ambiguity without freezing unrelated transport shapes.

## Gate Model
### Entry gate
- `W020` has established the broader semantic and callable-value floor.
- Current OxFunc note exchange has converged on library-context snapshot and availability/provider taxonomy as the highest-value next narrowing topics.

### Exit gate
- The minimum library-context snapshot field set is explicit in canonical OxFml docs.
- Availability, feature-gate, compatibility, runtime-capability, and provider-failure states have a narrower staged reading than today.
- At least one deterministic proving artifact exists for each in-scope staged distinction.

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

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: partial
- open_lanes: final callable transport, broader catalog breadth, and any coordinator-visible downstream consequences remain open outside this narrowing workset
- claim_confidence: high
