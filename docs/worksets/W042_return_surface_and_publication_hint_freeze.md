# W042: Return Surface and Publication-Hint Freeze

## Purpose
Freeze the first shared returned-value surface between OxFml and OxFunc for the currently covered built-in scope, including the split between ordinary values, publication-aware presentation values, and typed host/provider outcome projection.

## Position and Dependencies
- **Depends on**: `W032`, `W030`, `W041`
- **Blocks**: none
- **Cross-repo**: successor packet corresponding to OxFunc `W048`; OxFml owns publication-hint and seam-significant returned carrier meaning; OxFunc owns function-semantic production of ordinary values and projection of typed host/provider outcomes

## Scope
### In scope
1. Freeze the first returned-value split for the already-covered scope.
2. Keep presentation-aware returned values distinct from typed host/provider outcome projection.
3. Align value/publication hints with current seam-significant formatting and host/provider behavior.
4. Add deterministic evidence for the frozen split.
5. Decide whether the current first-freeze factoring is sufficient as-is for implementation use.

### Out of scope
1. Broader rich-value model closure.
2. Full UI/rendering behavior.
3. Pack-grade publication replay promotion.

## Deliverables
1. A canonical first shared return-surface split.
2. Explicit handling of `ValueWithPresentation`-style results.
3. Deterministic evidence for ordinary value, presentation-aware value, and typed host/provider projection lanes.
4. An explicit list of any still-deferred richer return-surface factors.

## Gate Model
### Entry gate
- `W030` has already widened the semantic-format and display boundary enough for a first returned-value freeze.
- `W032` has already aligned the callable/provider seam enough that return-surface work is not guessing.

### Exit gate
- Ordinary value, presentation-aware value, and typed host/provider outcome projection are canonically separated.
- Remaining richer return-surface gaps are explicitly listed.
- At least one deterministic artifact exists per in-scope return family.

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
  - the first shared return-surface split is accepted directionally, but not yet canonically frozen on the OxFml side
  - deterministic evidence still needs to be grouped around the frozen returned-value families rather than inferred from earlier formatting/provider slices
  - richer return-surface factoring beyond the first freeze remains intentionally out of scope
  - OxFml still needs to answer with implementation-facing precision whether the current ordinary-value / `ValueWithPresentation` / typed host-provider projection split can be frozen as-is
  - current OxFunc reading is that this is now a freeze-and-consumer packet rather than a broad semantic-open lane, and the final note round leaves local execution as the remaining next step
- claim_confidence: draft
