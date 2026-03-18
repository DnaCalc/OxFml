# W014: Semantic Breadth and OxFunc Catalog Expansion

## Purpose
Broaden the OxFml semantic-plan and evaluator floor so more of the OxFunc-facing semantic boundary is exercised with richer provenance and catalog coverage.

## Position and Dependencies
- **Depends on**: `W013`, `W003`
- **Blocks**: `W015`, `W016`, `W017`, `W018`
- **Cross-repo**: OxFunc observation ledger remains a required inbound design input; downstream coordination is semantic, not API-shape-driven

## Scope
### In scope
1. Broaden semantic-plan coverage over more function families and helper/provenance lanes.
2. Widen prepared-call/result provenance beyond the current narrow floor.
3. Extend OxFunc metadata/catalog coverage for higher-risk formula lanes.
4. Add replay-backed evidence for the widened semantic boundary.

### Out of scope
1. Full OxFunc catalog closure.
2. Multi-node scheduler policy.
3. Pack-grade replay promotion.

## Deliverables
1. Broader semantic-plan and evaluator coverage over the OxFunc-facing boundary.
2. Richer prepared-call/result provenance exercised by tests and replay fixtures.
3. Updated boundary and artifact-shape docs for the widened semantic floor.

## Gate Model
### Entry gate
- `W013` has broadened the parser/binder floor enough to support richer semantic coverage.

### Exit gate
- More of the OxFunc-facing semantic boundary is exercised locally with typed provenance.
- High-risk helper/scalarization/reference lanes are better covered than the current narrow floor.
- Updated docs and replay artifacts reflect the widened semantic-plan surface.

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
- open_lanes: full OxFunc catalog closure and richer callable-value carriers remain outside this workset scope
- claim_confidence: validated
