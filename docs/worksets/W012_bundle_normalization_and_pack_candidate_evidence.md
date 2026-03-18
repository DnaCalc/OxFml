# W012: Bundle Normalization and Pack-Candidate Evidence

## Purpose
Add conservative OxFml-local normalized replay bundle evidence and pack-candidate rehearsal artifacts on top of the broadened reduced-witness floor, without claiming pack-grade promotion.

## Position and Dependencies
- **Depends on**: `W011`
- **Blocks**: later pack-grade replay promotion claims
- **Cross-repo**: Foundation remains authoritative for replay bundle governance and pack-facing capability levels; OxFml remains authoritative for artifact meaning, typed reject semantics, and local pack-candidate admissibility rules

## Scope
### In scope
1. Create local normalized replay bundle examples that preserve source schema ids, witness refs, and non-pack-eligible state.
2. Create a local pack-candidate index or bundle set that proves bundle-normalization evidence can be carried without overclaiming pack-grade maturity.
3. Bind current local reduced-witness artifacts into the normalized bundle evidence.
4. Add tests that validate source-schema preservation, capability-floor honesty, and non-pack-eligible local candidate state.
5. Tighten replay docs so local normalized bundle evidence is visible and clearly separated from true pack-grade promotion.

### Out of scope
1. Claiming `cap.C5.pack_valid`.
2. Promoting any OxFml witness to pack-grade status.
3. Declaring replay-safe formula-text, bind, fence, or capability-view rewrites.
4. Replacing OxFml-owned taxonomy or artifact semantics with generic replay bundle prose.

## Deliverables
1. Local normalized replay bundle artifacts tied to current OxFml witness families.
2. A local pack-candidate index or equivalent evidence structure referencing those normalized bundles.
3. Passing tests showing the bundle evidence remains local-only and non-pack-eligible.
4. Updated replay docs and planning docs reflecting the new local normalization floor and remaining promotion gap.

## Gate Model
### Entry gate
- `W011` has broadened local reduced-witness coverage beyond the first local FEC reject case.

### Exit gate
- At least two normalized local replay bundle artifacts exist and are tested.
- Local pack-candidate evidence preserves source schema ids, witness refs, and non-pack-eligible state.
- Updated docs explicitly distinguish local normalized bundle evidence from pack-grade promotion.

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
- open_lanes: normalized bundle evidence is now local and non-pack-eligible by design, and pack-grade promotion remains outside this workset
- claim_confidence: validated
