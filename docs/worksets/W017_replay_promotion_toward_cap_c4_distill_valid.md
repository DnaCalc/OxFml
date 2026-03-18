# W017: Replay Promotion Toward cap C4 Distill Valid

## Purpose
Broaden OxFml replay promotion beyond the current local rehearsal floor and establish the evidence needed to move honestly toward `cap.C4.distill_valid` without overclaiming pack-grade maturity.

## Position and Dependencies
- **Depends on**: `W015`, `W016`, `W012`
- **Blocks**: `W018`
- **Cross-repo**: Foundation replay governance remains authoritative for capability and lifecycle policy; OxFml remains authoritative for admissible reduction-unit meaning and replay-safe transform claims

## Scope
### In scope
1. Broaden reduced-witness coverage into more fixture families and more than one outcome class.
2. Add at least one irreducible or unsupported reduction outcome with explicit lifecycle handling.
3. Tighten retained-witness and promotion criteria for future pack-facing claims.
4. Add stronger replay-promotion docs and conformance evidence around the `cap.C4` boundary.

### Out of scope
1. Claiming `cap.C5.pack_valid`.
2. Full pack-grade promotion.
3. Declaring replay-safe rewrite families without local evidence.

## Deliverables
1. Broader reduced-witness and lifecycle evidence toward `cap.C4`.
2. Explicit irreducible or unsupported reduction evidence.
3. Updated replay-promotion planning docs that make the `cap.C4` residual honest and concrete.

## Gate Model
### Entry gate
- `W016` has established a stronger checked-formal and runtime floor for replay promotion claims.

### Exit gate
- The local replay-promotion floor is materially broader than the current retained-local rehearsal set.
- At least one irreducible or unsupported reduction path is exercised and documented.
- The live replay docs can state a sharper, evidence-backed residual for `cap.C4`.

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
- open_lanes: broader reduced-witness breadth and promotion-grade closure toward `cap.C4.distill_valid` remain outside this workset scope
- claim_confidence: validated
