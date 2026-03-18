# W004: FEC/F3E Runtime and Schema Fixture Baseline

## Purpose
Translate the rewritten FEC/F3E seam into an implementation-start runtime baseline plus the first concrete schema-fixture obligations.

## Position and Dependencies
- **Depends on**: `W001`, `W003`
- **Blocks**: `W005`, `W006`, `W007`, `W008`
- **Cross-repo**: recorded OxCalc seam handoffs remain available as ad hoc review inputs when coordinator-facing clauses mature further

## Scope
### In scope
1. Define the first implementation-facing runtime baseline for `prepare -> open_session -> capability_view -> execute -> commit`.
2. Tighten accepted-candidate, commit-bundle, reject-record, and trace-event runtime relationships.
3. Turn the schema replay fixture plan into explicit first fixture families and acceptance criteria.
4. Narrow coordinator-visible consequences enough for early implementation without leaking scheduler policy into the seam.
5. Define how scheduler-relevant execution-profile facts are surfaced without turning the seam into a scheduler API.
6. Define the single-formula host recalc behavior needed before broader DNA OneCalc host specification.

### Out of scope
1. Stage 2 concurrent/session-contention promotion.
2. Final wire encodings.
3. OxCalc-owned scheduler and dirty-closure policy.

## Deliverables
1. A code-start FEC/F3E runtime baseline.
2. A first accepted-candidate / commit / reject / trace fixture set definition.
3. Updated coordinator-facing handoff text if any additional mature seam clauses are promoted.
4. Clear statement of what remains deferred pending replay evidence or later ad hoc cross-repo review.
5. An explicit rule for surfaced execution-profile facts needed by concurrent or async scheduling.
6. A code-start proving-host rule set for one-formula full update/full recalc behavior with mutable defined-name inputs.

## Gate Model
### Entry gate
- OxFml minimum seam schemas and taxonomy docs are in the live bootstrap set.
- Recorded seam handoffs are reviewed before any new coordinator-facing promotion.

### Exit gate
- Runtime phase boundaries are explicit enough for implementation-start.
- Schema-fixture families are explicit enough for replay artifact authoring.
- Evaluator output versus committed publication remains clearly separated.
- Any newly mature coordinator-facing adjustments are recorded and handed off if needed.
- Scheduler-relevant execution restrictions are surfaced as typed facts without leaking scheduling policy into OxFml.

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
- open_lanes: the current witness corpus is still local rather than pack-grade, and Stage 2 contention/concurrency seams remain open by design outside this baseline
- claim_confidence: validated
