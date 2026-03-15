# W007: Execution Profiles and Concurrency Contract Baseline

## Purpose
Narrow the OxFml execution-profile and concurrency-facing contract early enough that concurrent and async core-engine work does not have to infer formula-level safety constraints after the fact.

## Position and Dependencies
- **Depends on**: `W003`, `W004`, `W005`
- **Blocks**: later concurrent-evaluation implementation claims and Stage 2 concurrency-hardening claims
- **Cross-repo**: may emit ad hoc seam observations to OxCalc where surfaced execution restrictions affect coordinator scheduling assumptions

## Scope
### In scope
1. Narrow the execution-profile vocabulary carried by `SemanticPlan` and surfaced evaluator facts.
2. Separate thread-safe, thread-affine, host-query, async, serial-only, and single-flight-style lanes at the semantic-contract level.
3. Define the first scheduler-facing consumption rule for execution restrictions without turning OxFml into scheduler-policy owner.
4. Define the first replay and TLA+-visible witness families for execution restrictions and fence-sensitive concurrency outcomes.
5. Identify high-risk function families that must declare execution restrictions early.

### Out of scope
1. Full scheduler policy.
2. Final Stage 2 concurrency implementation.
3. Broad performance tuning.

## Deliverables
1. A narrowed execution-profile vocabulary for formula and call surfaces.
2. A clearer OxFml-to-core rule for surfaced execution restrictions.
3. A first witness plan for replay and TLA+ scenarios involving execution restrictions.
4. An explicit residual list of concurrency lanes still deferred.

## Gate Model
### Entry gate
- `SemanticPlan` and FEC/F3E runtime baselines are explicit enough to carry execution restrictions.
- High-risk concurrency lanes are already called out in the live bootstrap set.

### Exit gate
- Execution-profile vocabulary is explicit enough for implementation-start.
- Scheduler-facing consumption boundaries are explicit enough to avoid hidden policy leakage.
- First replay/model witness families for execution restrictions are declared.

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
- open_lanes: the current execution-profile vocabulary is exercised locally but still backed by a narrow OxFunc trait registry; no checked TLA+ concurrency model runs exist yet beyond the local sequential lifecycle skeleton; and Stage 2 contention semantics remain intentionally deferred
- claim_confidence: validated
