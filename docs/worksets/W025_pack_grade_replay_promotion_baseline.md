# W025: Pack-Grade Replay Promotion Baseline

## Purpose
Define and exercise the first OxFml-side promotion path from retained-local replay evidence toward pack-grade replay governance without claiming `cap.C4.distill_valid` or `cap.C5.pack_valid` prematurely.

## Position and Dependencies
- **Depends on**: `W023`, `W024`
- **Blocks**: `W028`, `W029`, `W030`
- **Cross-repo**: Foundation replay governance remains authoritative for capability, registry, lifecycle, and promotion policy; OxFml remains authoritative for local artifact meaning and replay-safe transform boundaries

## Scope
### In scope
1. Define OxFml-local promotion criteria from retained-local replay evidence to pack-grade candidate evidence.
2. Tighten bundle/index, lifecycle, quarantine, and retained-witness rules for non-local promotion readiness.
3. Establish the minimum machine-readable planning and evidence surfaces needed for future pack-grade replay promotion.
4. Make explicit which current local witness families are nearest to promotion and which are still ineligible.

### Out of scope
1. Claiming `cap.C4.distill_valid`.
2. Claiming `cap.C5.pack_valid`.
3. Foundation-side registry or doctrine changes.
4. Declaring formula, bind, fence, or capability-view rewrites replay-safe beyond currently canonical OxFml limits.

## Deliverables
1. A pack-grade promotion baseline for OxFml replay families with explicit eligibility, exclusion, and quarantine rules.
2. Updated replay-planning docs that separate retained-local maturity from promotion-grade maturity.
3. Machine-readable planning/evidence surfaces identifying candidate families, required predicates, and promotion blockers.

## Gate Model
### Entry gate
- `W023` has established a materially broader retained-local replay floor and stronger local lifecycle evidence.
- `W024` has established host-policy and empirical-pack planning surfaces that must remain aligned with replay promotion criteria.

### Exit gate
- OxFml has an explicit promotion baseline that is stronger than retained-local policy alone.
- At least one replay family is classified through a promotion-readiness rubric without overclaiming `cap.C4` or `cap.C5`.
- The live replay docs make the residual path from local retained witnesses to pack-grade promotion explicit.

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
- open_lanes: promotion-grade evidence and any non-local replay corpus remain open outside this baseline
- claim_confidence: high
