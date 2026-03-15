# W005: Replay and Formal Kickoff for Core Surfaces

## Purpose
Start the first actual assurance-authoring lane for OxFml core surfaces rather than only planning them.

## Position and Dependencies
- **Depends on**: `W002`, `W003`, `W004`
- **Blocks**: `W007`, `W008`, and later implementation-closure claims across parser/bind/seam features
- **Cross-repo**: Green-owned formal placement may need coordination; downstream consumers may be informed on an ad hoc basis where fixtures affect shared seams

## Scope
### In scope
1. Author the first replay/schema fixture artifacts for parse/bind, prepared-call/result, and FEC/F3E payload surfaces.
2. Author the first Lean-friendly ADT cut list and first TLA+ model cut list.
3. Bind the formal artifact register and fixture plan to specific artifact locations and naming conventions.
4. Update conformance docs to point at concrete witness artifacts rather than only planned witness families.
5. Ensure the first witness set includes at least one high-risk seam lane such as `@`/`SINGLE`, `LET`/`LAMBDA`, or execution-profile restrictions.
6. Author the first formula-oriented empirical oracle scaffolding plan and concrete scenario artifact shape.

### Out of scope
1. Full proof development.
2. Full Stage 2 concurrency model closure.
3. Broad empirical Excel-compat reruns.

## Deliverables
1. First concrete replay/schema fixture artifacts.
2. First formal artifact file/module plan with concrete destinations.
3. Updated assurance docs linking to concrete local artifacts.
4. Clear residual list of what remains planning versus authored assurance.
5. A first concrete empirical formula-oracle scenario shape for Excel-backed validation.

## Gate Model
### Entry gate
- Core parser/binder, OxFunc-boundary, and FEC/F3E runtime baselines are explicit enough to target with artifacts.
- Formal artifact register and schema replay fixture plan exist in the live bootstrap set.

### Exit gate
- At least one concrete local witness artifact exists for each major surface family targeted in scope.
- Assurance docs reference concrete artifacts rather than only future families.
- Remaining un-authored proof/model lanes are explicitly bounded.

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
- open_lanes: first local Lean and TLA+ skeleton artifacts are now authored, but no checked proofs or model runs exist yet; the witness corpus remains local rather than pack-grade; and cross-build Excel empirical refresh remains outside this kickoff workset
- claim_confidence: validated
