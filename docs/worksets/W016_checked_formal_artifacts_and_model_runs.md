# W016: Checked Formal Artifacts and Model Runs

## Purpose
Turn the current local formal skeleton floor into checked local formal artifacts and model runs tied to the exercised OxFml runtime and replay clauses.

## Position and Dependencies
- **Depends on**: `W013`, `W014`, `W015`
- **Blocks**: `W017`
- **Cross-repo**: Foundation Green-owned doctrine remains authoritative for proof/model posture; this workset stays local to OxFml-owned clause families and evidence

## Scope
### In scope
1. Promote current local Lean/TLA+ skeletons into checked local artifacts or model runs.
2. Extend the clause-to-formal-artifact register where runtime/replay clauses are now exercised.
3. Add deterministic command or harness surfaces for local proof/model execution where practical.
4. Tighten docs so checked-formal versus skeleton-only status is explicit.

### Out of scope
1. Full proof closure for all OxFml semantics.
2. Green-owned promotion outside this repo.
3. Pack-grade replay promotion.

## Deliverables
1. Checked local formal artifacts or model runs for at least the current session lifecycle and publish/no-publish floor.
2. Updated formal planning docs that distinguish checked local artifacts from remaining skeleton-only lanes.
3. Clear residual list for the next formal gaps after the checked local floor exists.

## Gate Model
### Entry gate
- `W015` has stabilized the current runtime clause families that the local formal lane should target first.

### Exit gate
- At least one Lean-oriented and one TLA+-oriented local artifact move beyond skeleton-only status.
- The checked-formal status is explicit in the live formal planning docs.
- Residual unproven lanes remain visible rather than implied closed.

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
- open_lanes: broader proof closure and additional model families remain outside this workset scope
- claim_confidence: validated
