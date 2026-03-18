# W033: Replay Promotion Toward cap.C4 Distill Valid

## Purpose
Broaden OxFml replay promotion beyond the current local promotion-readiness and retained-witness floor so the repo can move honestly toward `cap.C4.distill_valid` without overclaiming pack-grade or `cap.C5.pack_valid` maturity.

## Position and Dependencies
- **Depends on**: `W025`, `W028`, `W029`, `W030`
- **Blocks**: `W035`
- **Cross-repo**: Foundation replay governance remains authoritative for capability and lifecycle policy; OxFml remains authoritative for local artifact meaning, replay-safe transform boundaries, and retained witness interpretation

## Scope
### In scope
1. Broaden reduced-witness and retained-witness breadth into additional commit, topology, format/display, and async/runtime families.
2. Tighten promotion-readiness criteria from retained-local evidence toward stronger `cap.C4`-adjacent evidence.
3. Add at least one explicit irreducible or unsupported witness outcome beyond the current rehearsal floor.
4. Strengthen machine-readable promotion and lifecycle surfaces for non-pack-grade but promotion-facing evidence.

### Out of scope
1. Claiming `cap.C4.distill_valid`.
2. Claiming `cap.C5.pack_valid`.
3. Foundation-side registry or capability changes.
4. Formula, bind, fence, or capability-view rewrites beyond currently declared replay-safe limits.

## Deliverables
1. Broader local reduced-witness and retained-witness evidence across more seam families.
2. A sharper local residual toward `cap.C4.distill_valid`.
3. Updated replay-governance docs and machine-readable indices that make promotion blockers explicit.

## Gate Model
### Entry gate
- `W025` established promotion-readiness planning artifacts.
- `W028`, `W029`, and `W030` widened publication, runtime, and format/display consequence families worth promoting.

### Exit gate
- Reduced-witness breadth is materially broader than the current retained-local floor.
- At least one irreducible or unsupported witness outcome is explicitly carried in the local evidence set.
- The residual path toward `cap.C4.distill_valid` is sharper than the current local promotion-readiness baseline.

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
- open_lanes: `cap.C4.distill_valid`, `cap.C5.pack_valid`, and non-local pack promotion remain outside this workset scope
- claim_confidence: draft
