# W023: Replay Promotion and Retained Witness Sets

## Purpose
Broaden OxFml replay promotion beyond the current local rehearsal floor so retained local witness sets and stronger `cap.C4`-adjacent evidence can be stated honestly without claiming pack-grade maturity.

## Position and Dependencies
- **Depends on**: `W021`, `W022`, `W017`
- **Blocks**: `W024`
- **Cross-repo**: Foundation replay governance remains authoritative for capability and lifecycle policy; OxFml remains authoritative for local witness meaning and replay-safe transform claims

## Scope
### In scope
1. Broaden retained local witness coverage across more fixture families and more than one reduction outcome class.
2. Tighten retained-witness selection, quarantine, and promotion criteria for stronger local replay claims.
3. Add stronger local conformance evidence around the `cap.C4` boundary without claiming it.
4. Update replay governance docs so the residual path to pack-grade promotion remains explicit.

### Out of scope
1. Claiming `cap.C4.distill_valid`.
2. Claiming `cap.C5.pack_valid`.
3. Green-owned or pack-grade promotion outside this repo.

## Deliverables
1. Broader retained local witness sets and lifecycle evidence.
2. Sharper local replay-promotion criteria and stronger conformance evidence.
3. Updated replay-governance docs with honest residuals toward `cap.C4`.

## Gate Model
### Entry gate
- `W022` has strengthened the checked-formal and replay/runtime floor enough that broader retained witness promotion is meaningful.

### Exit gate
- The retained local replay floor is materially broader than the current rehearsal set.
- Lifecycle, quarantine, and retained-witness selection rules are exercised over more than one family.
- The live replay docs state a sharper local residual toward `cap.C4` without overclaiming it.

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
- open_lanes: broader retained witness breadth beyond the current local families and any claim toward `cap.C4.distill_valid` remain outside this workset scope
- claim_confidence: validated
