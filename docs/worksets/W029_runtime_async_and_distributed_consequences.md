# W029: Runtime Async and Distributed Consequences

## Purpose
Push the current OxFml-managed runtime beyond the local contention baseline so async, distributed, and stronger scheduler-facing consequence classes are planned and exercised without collapsing OxCalc-owned coordinator policy into OxFml.

## Position and Dependencies
- **Depends on**: `W028`
- **Blocks**: `W030`
- **Cross-repo**: OxCalc owns multi-node coordinator policy; OxFml owns evaluator/runtime artifact meaning, typed execution consequences, and replay-preserved runtime facts

## Scope
### In scope
1. Broaden async and distributed runtime consequence modeling above the current local contention floor.
2. Tighten scheduler-facing execution-restriction and capability-sensitive effect transport where OxCalc consumption depends on surfaced evaluator/runtime truth.
3. Add replay and formal consequences for broader multi-session or async runtime paths.
4. Clarify which consequences are evaluator/runtime facts versus coordinator policy.

### Out of scope
1. Full OxCalc graph scheduling policy.
2. Final distributed host/product semantics.
3. Broad pack-grade replay or formal promotion beyond what this runtime expansion directly exercises.

## Deliverables
1. A stronger async/distributed runtime consequence baseline for OxFml-managed session behavior.
2. Narrower surfaced execution-restriction and capability-sensitive effect transport for downstream consumption.
3. Replay and formal artifacts proving the new runtime consequence classes are typed and stable.

## Gate Model
### Entry gate
- `W028` has broadened commit/publication and topology consequences enough that broader runtime semantics have a meaningful artifact target.

### Exit gate
- Async or broader multi-session runtime consequence classes are explicit and exercised.
- Scheduler-facing effect transport is stronger than the current local contention-only floor.
- Replay and formal artifacts cover at least one new async or distributed-style runtime consequence family.

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
- open_lanes: full OxCalc-owned coordinator policy and product-level distributed semantics remain outside this workset scope
- claim_confidence: high
