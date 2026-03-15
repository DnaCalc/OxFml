# W008: Single-Formula Host and Empirical Oracle Bootstrap

## Purpose
Turn the OxFml test-ladder host model into an implementation-start proving host plus an explicit empirical-oracle scaffold that can later feed DNA OneCalc specification work.

## Position and Dependencies
- **Depends on**: `W002`, `W003`, `W004`, `W005`, `W006`
- **Blocks**: later DNA OneCalc host specification and broader formula-oracle proving claims
- **Cross-repo**: OxFunc-backed execution is required; no standing OxCalc dependency is required

## Scope
### In scope
1. Narrow the single-formula host artifact model and runtime helper surface.
2. Define the mutable defined-name input model for full update and full recalc proving.
3. Define the first candidate, commit, reject, and trace capture rules for the proving host.
4. Define the first empirical Excel-oracle scenario artifact shape and curation rules.
5. Make explicit how the minimal local bootstrap evaluator and OxFunc-backed execution both plug into the same proving-host ladder.

### Out of scope
1. Multi-formula dependency graphs.
2. OxCalc scheduler policy.
3. Full DNA OneCalc host specification.

## Deliverables
1. A narrowed single-formula proving-host baseline.
2. A concrete host artifact shape for mutable defined-name input scenarios.
3. A concrete empirical-oracle scenario and evidence shape for formula-level validation.
4. A clearer bridge from OxFml proving-host work to later DNA OneCalc specification work.

## Gate Model
### Entry gate
- Parser/binder, semantic-plan, and FEC/F3E runtime baselines are explicit enough to drive one-formula host runs.
- Test ladder and replay/planning docs are in the live bootstrap set.

### Exit gate
- Single-formula host artifact and helper surfaces are explicit enough for implementation-start.
- Defined-name update and full recalc semantics are explicit enough for proving scenarios.
- Empirical-oracle scenario artifacts are explicit enough for first run authoring.

### Sequence exit gate
- Completion of `W008` is the current user-authorized AutoRun sequence exit gate for `W002 -> W008`.

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
- open_lanes: the proving host remains single-formula only by design; direct Excel-run corpus authoring is still outside the current local fixture floor; and later DNA OneCalc specification work remains downstream of this bootstrap
- claim_confidence: validated
