# W011: Reduced Witness Family Breadth

## Purpose
Broaden OxFml replay-valid reduced-witness coverage beyond the first local FEC reject case so witness distillation is exercised across multiple fixture families before any stronger promotion narrative is attempted.

## Position and Dependencies
- **Depends on**: `W010`
- **Blocks**: `W012`
- **Cross-repo**: Foundation remains authoritative for replay predicate and lifecycle registries; OxFml remains authoritative for reduction-unit meaning over typed formula and seam artifacts

## Scope
### In scope
1. Add additional reduced-witness artifact sets over more than one OxFml fixture family.
2. Exercise commit-accepted, reject, and non-commit lifecycle or execution-profile witness retention paths.
3. Bind each reduced witness to an explicit reduction manifest, witness bundle, and lifecycle record.
4. Extend local replay tests so each widened reduced witness proves replay-closed preservation for its retained predicate.
5. Tighten live replay docs so the broader local reduced-witness floor is visible and not confused with pack-grade maturity.

### Out of scope
1. Full pack-grade promotion.
2. Claiming `cap.C5.pack_valid`.
3. Declaring replay-safe formula-text, bind, fence, or capability-view rewrites.
4. Unifying subsystem schemas into one replay schema.

## Deliverables
1. At least two new reduced-witness artifact sets beyond the first local FEC reject case.
2. Passing local replay tests proving retained-predicate preservation for each new witness family.
3. Updated replay fixture and assurance docs describing the broadened local reduced-witness floor.

## Gate Model
### Entry gate
- `W010` has established the first local reduced witness and the baseline reduction/lifecycle policy.

### Exit gate
- Reduced witnesses exist across more than one source fixture family.
- The retained local witnesses prove accepted, rejected, or execution-restriction preservation without overclaiming replay-safe rewrites.
- The live OxFml replay docs describe the broader local reduced-witness floor and keep pack-grade promotion explicit as still open.

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
- open_lanes: reduced-witness coverage is now broader but still local, and pack-grade witness promotion remains outside this workset
- claim_confidence: validated
