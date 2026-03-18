# W030: Semantic Formatting and Display Boundary Closure

## Purpose
Narrow the current semantic-format versus display-facing boundary so formatting-sensitive formula semantics, publication consequences, host-query pressure, and retained/empirical evidence no longer rely on broad residual wording.

## Position and Dependencies
- **Depends on**: `W025`, `W026`, `W028`, `W029`
- **Blocks**: later broader empirical-pack promotion and any stronger coordinator-facing reading of format/display consequences
- **Cross-repo**: OxFml remains authoritative for formula-semantic formatting and seam-significant publication consequences; OxCalc remains a consumer of coordinator-relevant format/display consequences rather than their semantic owner

## Scope
### In scope
1. Broaden semantic-formatting family coverage beyond the current local slice.
2. Narrow the semantic-format versus display-facing publication boundary in canonical OxFml docs.
3. Add deterministic replay, proving-host, and empirical-oracle artifacts for richer formatting-sensitive and host-query-sensitive publication cases.
4. Clarify which format/display consequences are seam-significant and which remain downstream/UI-local.

### Out of scope
1. Full display/rendering product behavior.
2. Pack-grade empirical promotion.
3. Coordinator-facing handoff unless canonical clause changes become materially narrower than today.

## Deliverables
1. A stronger semantic-formatting and display-boundary baseline in canonical OxFml docs.
2. Broader exercised formatting-sensitive publication and replay evidence.
3. A clearer downstream reading for OxCalc and DNA OneCalc of which format/display consequences are seam-significant.

## Gate Model
### Entry gate
- `W025` has established stronger replay-promotion planning.
- `W026` has narrowed availability/provider meaning that can affect formatting-sensitive host or service lanes.
- `W028` and `W029` have broadened publication and runtime consequence surfaces enough that a narrower format/display closure is meaningful.

### Exit gate
- The semantic-format versus display-facing boundary is narrower than the current residual wording.
- Formatting-sensitive publication consequences are broader and better exercised than the current local floor.
- Replay, proving-host, and empirical evidence exists for at least one newly narrowed seam-significant format/display family.

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
- open_lanes: full product display behavior and pack-grade empirical promotion remain outside this workset scope
- claim_confidence: high
