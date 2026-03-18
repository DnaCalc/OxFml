# W021: Async Runtime and Scheduler Surface Baseline

## Purpose
Broaden the FEC/F3E runtime beyond the current Stage 2 local contention floor into an async- and distribution-aware baseline without leaking OxCalc-owned global scheduler policy into OxFml.

## Position and Dependencies
- **Depends on**: `W020`, `W015`
- **Blocks**: `W022`, `W023`, `W024`
- **Cross-repo**: coordinator-facing seams remain OxFml-owned and may require ad hoc OxCalc handoff if materially sharpened

## Scope
### In scope
1. Broaden the managed runtime and session model for multi-locus, retry-sensitive, or async-coupled evaluator paths.
2. Tighten typed effect, reject, and trace handling where scheduler-facing correctness depends on surfaced evaluator facts.
3. Broaden the publication pipeline so commit-bundle construction, open-session-to-commit wiring, and richer topology-fact emission are explicit owners rather than incidental side effects.
4. Add deterministic local runtime fixtures for the broader async or distribution-sensitive baseline that OxFml can own locally.
5. Update seam/runtime docs to distinguish the broadened OxFml-managed baseline from OxCalc-owned scheduler policy.

### Out of scope
1. Full distributed execution.
2. OxCalc global scheduling policy.
3. Pack-grade replay promotion.

## Deliverables
1. A stronger exercised managed runtime baseline for async- or retry-sensitive paths.
2. Stronger commit/publication wiring with richer topology and publication-fact surfacing.
3. Deterministic runtime fixtures for the broadened reject/effect/trace/publication surface.
4. Updated seam/runtime docs reflecting the broader async-facing local floor.

## Gate Model
### Entry gate
- `W020` has broadened the semantic surface enough that the next runtime lanes are representative rather than synthetic.

### Exit gate
- Async- or retry-sensitive runtime paths are exercised locally with typed rejects, effects, and traces.
- Commit/publication wiring and richer topology-fact emission are explicitly exercised rather than remaining incidental or ownerless.
- Runtime no-publish and cleanup rules remain explicit under those broader paths.
- The live seam docs reflect the stronger OxFml-managed runtime baseline.

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
- open_lanes: broader distributed execution and OxCalc-owned global scheduler policy remain outside this workset scope
- claim_confidence: validated
