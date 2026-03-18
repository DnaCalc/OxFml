# W028: Commit, Publication, and Topology Breadth

## Purpose
Broaden the current commit/publication floor so OxFml’s publication pipeline, topology facts, and retained/reduced witness projections are materially stronger than the current narrow exercised slice.

## Position and Dependencies
- **Depends on**: `W021`, `W023`, `W025`, `W026`
- **Blocks**: `W029`, `W030`
- **Cross-repo**: OxFml remains authoritative for candidate, commit, reject, fence, topology-fact, and publication-consequence meaning; narrower OxCalc-facing handoff may be required if coordinator-visible clauses change materially

## Scope
### In scope
1. Broaden commit-bundle construction beyond the current narrow accepted-candidate publication slice.
2. Strengthen open-session-to-commit runtime wiring and publication-consequence coverage.
3. Broaden topology-fact encoding, including dependency additions, removals, and reclassifications where evaluator/runtime truth depends on them.
4. Tighten retained/reduced replay-family projection rules for publication and topology consequences.

### Out of scope
1. Full distributed coordinator semantics.
2. Pack-grade replay promotion beyond the promotion baseline from `W025`.
3. Unbounded future topology taxonomy outside exercised evidence.

## Deliverables
1. A broader commit/publication pipeline baseline in canonical docs and exercised code.
2. Narrower topology-fact and dependency-consequence coverage across commit and replay families.
3. If needed, a narrower OxCalc-facing seam packet for any coordinator-visible clause that materially changes.

## Gate Model
### Entry gate
- `W025` has established a clearer replay-promotion baseline.
- `W026` has narrowed library-context and availability meanings that can affect publication and runtime consequence typing.

### Exit gate
- Commit/publication wiring is broader than the current narrow candidate-to-commit slice.
- Topology-fact encoding is materially richer, including dependency consequence classes exercised through deterministic artifacts.
- Replay-family projection rules for publication and topology consequences are explicit and no longer relying on generic summary prose.

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
- open_lanes: full distributed publication policy and pack-grade replay breadth remain open outside this workset
- claim_confidence: high
