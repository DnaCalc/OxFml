# W059: Operator Semantic Dispatch Boundary Correction

## Purpose
Repair the ordinary-operator ownership split so OxFml continues to own grammar, precedence, and bound operator structure while OxFunc owns semantic execution of ordinary operator rows.

## Position and Dependencies
- **Depends on**: `W055`
- **Blocks**: `none`
- **Cross-repo**: `OxFunc operator-row inventory already exists; no new OxFunc semantic invention required for the arithmetic slice`

## Scope
### In scope
1. replace OxFml-local ordinary operator execution with OxFunc operator dispatch for admitted scalar and array-lifted value-operator rows,
2. preserve current OxFml parse/bind operator structure while narrowing semantic execution,
3. register the ownership-correction bug stream and connect it to this workset,
4. add deterministic regression coverage proving the admitted operator families remain stable after seam correction,
5. retain trace evidence showing the corrected `FUNC.OP_*` dispatch path for arithmetic, unary, concat/compare, and explicit reference-operator slices.

### Out of scope
1. broad operator-catalog freeze changes in OxFunc,
2. future policy work about whether simple cell-to-cell ranges should preserve explicit range-operator provenance beyond current binder normalization.

## Deliverables
1. OxFml ordinary value-operator execution no longer computes admitted operator semantics locally.
2. Regression tests prove admitted unary, arithmetic, percent, concat, comparison, and explicit reference-operator rows still evaluate correctly through the OxFunc-backed path.
3. Trace evidence proves the exercised rows now lower into `FUNC.OP_*`.
4. Bug stream `BUG-FML-003` is linked and statused honestly.

## Current Operator Inventory
### Present in OxFml token/parser/binder today
1. binary arithmetic: `+`, `-`, `*`, `/`, `^`
2. unary prefix forms: `+`, `-`
3. reference operators: `:`, union-comma, intersection-whitespace
4. explicit implicit-intersection prefix: `@`
5. spill suffix: `#`
6. postfix percent: `%`
7. concatenation: `&`
8. comparison operators: `=`, `<>`, `<`, `<=`, `>`, `>=`

### Missing in OxFml token/parser/binder today
1. none within the admitted ordinary-operator family owned by this workset

### Present in OxFunc operator catalog today
1. arithmetic: `FUNC.OP_ADD`, `FUNC.OP_SUBTRACT`, `FUNC.OP_MULTIPLY`, `FUNC.OP_DIVIDE`, `FUNC.OP_POWER`, `FUNC.OP_NEGATE`, `FUNC.OP_UNARY_PLUS`, `FUNC.OP_PERCENT`
2. concat/comparisons: `FUNC.OP_CONCAT`, `FUNC.OP_EQUAL`, `FUNC.OP_NOT_EQUAL`, `FUNC.OP_LESS_THAN`, `FUNC.OP_LESS_EQUAL`, `FUNC.OP_GREATER_THAN`, `FUNC.OP_GREATER_EQUAL`
3. reference operators: `FUNC.OP_RANGE_REF`, `FUNC.OP_INTERSECTION_REF`, `FUNC.OP_UNION_REF`, `FUNC.OP_SPILL_REF`, trim-ref rows

### Present but still partial at the current boundary
1. scalar and array-lifted ordinary value operators now dispatch through OxFunc rows
2. unary prefix arithmetic now binds as explicit unary nodes and dispatches through OxFunc rows
3. explicit intersection, union, and spill reference operators now dispatch through OxFunc rows
4. simple cell-to-cell `:` ranges may still normalize earlier to area atoms instead of surviving as an explicit `OP_RANGE_REF` trace
5. union value-context materialization now flows through OxFunc-owned `ReferenceKind::MultiArea` resolver-driven combination semantics
6. mixed-sheet multi-area is a distinct construct from same-sheet multi-area and is intentionally outside the admitted local materialization lane
7. whole-row and whole-column references are preserved in reference-visible lanes; OxFml no longer applies a local occupied-extent materialization shortcut for value-only lanes

## Gate Model
### Entry gate
- `BUG-FML-003` registered
- current local arithmetic execution site identified

### Exit gate
- admitted ordinary operator rows dispatch to OxFunc operator ids
- local regression floor passes
- residual operator families remain explicitly documented

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | yes |
| 2 | Conformance matrix rows updated? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes |
| 4 | Cross-repo impact assessed and handoff filed if needed? | yes |
| 5 | All required tests pass? | yes |
| 6 | No known semantic gaps remain in declared scope? | yes |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | no |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | no |

## Status
- execution_state: in_progress
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: partial
- open_lanes:
  - conformance-matrix and worklist promotion remain open
  - simple range canonicalization versus explicit operator-provenance policy remains open beyond the completed ordinary-operator seam correction
  - whole-row and whole-column local value-only lanes now fail honestly without a fuller sheet model; broader host-backed or model-backed dereference remains open beyond this workset
- claim_confidence: medium
