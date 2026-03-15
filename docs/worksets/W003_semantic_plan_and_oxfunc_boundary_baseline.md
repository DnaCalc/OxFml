# W003: Semantic Plan and OxFunc Boundary Baseline

## Purpose
Turn the OxFml/OxFunc seam from architecture-level wording into an implementation-start semantic-plan and prepared-call boundary.

## Position and Dependencies
- **Depends on**: `W002`
- **Blocks**: `W004`, `W005`, `W006`, `W007`, `W008`
- **Cross-repo**: informed by `../OxFunc/docs/upstream/NOTES_FOR_OXFML.md`; may require outbound observation updates if host-query or provenance minimums are narrowed

## Scope
### In scope
1. Define the first implementation-start semantic-plan surface tied to OxFunc metadata.
2. Tighten the minimum provenance vocabulary for prepared arguments and prepared results.
3. Tighten the host-query capability baseline for `CELL` / `INFO`-style lanes.
4. Define the first prepared-call/result replay fixture families.
5. Define the first execution-profile metadata surface that OxFml can expose to a concurrent core engine.
6. Define the boundary between the minimal local bootstrap evaluator and OxFunc-backed semantic execution.
7. Narrow the first code-facing public API surface for semantic-plan, evaluation, and execution-profile publication.

### Out of scope
1. Multi-node coordination policy.
2. FEC/F3E commit/reject runtime implementation.
3. Full function-catalog completeness in OxFunc.

## Deliverables
1. A narrowed semantic-plan baseline that code can target.
2. A smaller and more explicit prepared-call/result minimum vocabulary.
3. Replay-fixture planning for direct-scalar, array-like, reference-visible, `@`, `SINGLE`, helper-form, and host-query paths.
4. Updated OxFml/OxFunc boundary text where the current open lanes can be responsibly narrowed.
5. A first execution-profile vocabulary for concurrency- and async-relevant function lanes.
6. An explicit rule that OxFml uses OxFunc outputs for broader downstream function-semantic testing beyond the tiny local bootstrap kernel.
7. A public-surface baseline explicit enough for implementation-start across `compile_semantic_plan` and `evaluate`.

## Gate Model
### Entry gate
- Parser/binder artifact baseline is explicit enough to supply stable inputs to semantic planning.
- Current OxFunc upstream note has been reread against the live OxFml seam docs.
- The public API/runtime-service sketch is present in the live bootstrap set.

### Exit gate
- Semantic-plan and prepared-call/result surfaces are explicit enough for implementation-start.
- Minimum provenance vocabulary is narrower than the current open-ended baseline.
- Host-query boundary is explicit enough for early `CELL` / `INFO` integration planning.
- Replay fixture families for prepared-call/result semantics are declared.
- Execution-profile metadata is explicit enough that a core engine can consume scheduler-relevant formula profiles later without OxFml redesign.
- Semantic-plan and evaluate public-surface shapes are explicit enough for implementation-start.

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
- open_lanes: broader OxFunc catalog integration, richer callable-value carriers beyond the current replayable lambda summary, pack-grade replay promotion, and checked formal artifacts remain outside this baseline workset and continue in follow-on lanes
- claim_confidence: validated
