# W055: Operator Array Arithmetic Consolidation And OxFunc Seam Narrowing

## Purpose
Consolidate the new local operator array-arithmetic floor into a cleaner, less duplicated implementation shape and determine whether ordinary arithmetic operator semantics should narrow across the OxFml <-> OxFunc seam.

This workset exists because the current local fix is a correct immediate floor for the exercised failure family, but it still leaves two risks visible:
1. ordinary operator coercion and elementwise array behavior are duplicated in OxFml-local evaluator code even though the current seam doctrine says operator semantics belong to OxFunc,
2. the current regression floor is still narrower than the broader operator semantic space that Excel exposes across literals, references, spills, and dynamic-array-producing expressions.

## Position and Dependencies
- **Depends on**: `W049`, `W050`
- **Blocks**: any honest claim that ordinary arithmetic operator semantics are consolidated rather than only patched locally
- **Cross-repo**: possible OxFunc seam narrowing if the current local consolidation exposes a real shared-ownership mismatch

## Scope
### In scope
1. Audit the current ordinary arithmetic operator path in OxFml for duplicated coercion, array-lift, and result-class behavior.
2. Consolidate the current OxFml-local implementation so there is one narrow operator arithmetic path rather than an accreted local patch family.
3. Expand the deterministic local operator corpus beyond the current array-literal-only floor to include:
   - reference-derived arrays,
   - spill-derived arrays,
   - array/scalar and scalar/array asymmetry,
   - mismatched-shape outcomes,
   - internal element-error propagation.
4. Determine whether the current OxFml/OxFunc seam docs require a narrower shared operator-semantic surface or whether the consolidation can remain purely OxFml-local.
5. If a real seam mismatch is found, draft the exact narrowing candidate and handoff packet rather than leaving the issue as note-only concern.
6. Keep replay and retained-witness coverage aligned with any broadened operator corpus.

### Out of scope
1. Full empirical cross-build/channel execution of `P2-FML-012`.
2. Broad redesign of all OxFml/OxFunc operator transport.
3. Reopening unrelated callable, formatting, or host-query seam packets.
4. Claiming full Excel operator parity beyond the declared consolidation and first broadened witness floor.

## Deliverables
1. A consolidated operator arithmetic implementation path in OxFml with reduced duplication and clearer ownership boundaries.
2. A broadened deterministic local regression corpus for ordinary operator array-lift behavior.
3. Replay-host and retained-witness artifacts aligned to the broadened operator corpus.
4. An explicit OxFml decision record:
   - no seam change required, or
   - exact OxFml <-> OxFunc seam narrowing proposal with concrete packet/text.
5. If needed, a handoff packet and register entry for OxFunc.

## Gate Model
### Entry gate
- The immediate local failure family `={1,2,3;2,3,4} * -1` is fixed and replay-backed locally, but the resulting implementation still duplicates operator semantics that the current seam doctrine treats as OxFunc-owned.

### Exit gate
- The ordinary arithmetic operator path is consolidated locally without duplicated coercion logic scattered across multiple ad hoc helpers.
- The broadened local operator corpus covers more than array literals alone.
- Replay-host and retained-witness coverage exist for the broadened floor.
- A concrete decision is recorded on whether a seam narrowing is required.
- If seam narrowing is required, the exact proposed text and impact are captured in a handoff-ready packet.
- If seam narrowing is not required, the workset records why the current OxFml-local ownership is still doctrine-consistent for the admitted floor.

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
  - broaden operator corpus beyond array literals
  - consolidate duplicated local arithmetic semantics
  - decide whether seam narrowing is required
  - handoff to OxFunc if required
- claim_confidence: draft
