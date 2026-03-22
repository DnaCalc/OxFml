# W041: Typed Context and Query Bundle Freeze

## Purpose
Freeze the first shared OxFml/OxFunc typed context and query bundle for the currently covered built-in function scope, so implementation work can depend on a stable first-pass host capability surface.

## Position and Dependencies
- **Depends on**: `W032`, `W030`, `W038`
- **Blocks**: none
- **Cross-repo**: successor packet corresponding to OxFunc `W047`; OxFml owns capability-scoped evaluator and host-query carrier meaning; OxFunc owns function-semantic consumption of those typed queries once supplied

## Scope
### In scope
1. Freeze the first shared typed context/query bundle for the currently covered seam-heavy rows.
2. Decide whether the current OxFunc query names/result partitioning is sufficient as-is for the first freeze.
3. Keep the bundle capability-scoped and typed rather than object-handle based.
4. Add deterministic local evidence and spec wording for the covered families.
5. Answer whether any first-pass capability families need an OxFml-side merge or split before promotion.

### Out of scope
1. Full host-application runtime lifecycle.
2. Broader distributed coordinator policy.
3. New provider families not already in the current covered OxFunc scope.

## Deliverables
1. A canonical first-pass typed context/query bundle shape.
2. Explicit OxFml wording for what query families are in the first freeze.
3. Deterministic evidence for the covered query families.
4. An explicit list of any still-unfrozen names or result partitions.

## Gate Model
### Entry gate
- `W032` has narrowed the catalog/callable seam enough that typed host/context surfaces are the next honest lock lane.

### Exit gate
- The first shared typed context/query bundle is explicit enough for implementation use.
- Any non-frozen query naming or partitioning is explicitly listed.
- At least one deterministic artifact exists per in-scope query family.

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
- open_lanes:
  - the first shared typed context/query bundle is not yet canonically frozen on the OxFml side
  - current OxFunc query names and result partitions are accepted as a first freeze candidate, but not yet formally promoted in OxFml docs
  - deterministic OxFml evidence still needs to be grouped around the first frozen bundle rather than scattered across earlier host/provider slices
  - OxFml still needs to answer with implementation-facing precision whether the current query families should remain exactly as named or be capability-family merged/split before promotion
  - current OxFunc reading is that this is now a freeze-and-consumer packet rather than a broad semantic-open lane, and the final note round leaves local execution as the remaining next step
- claim_confidence: draft
