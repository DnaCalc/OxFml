# W053: Grouped Aggregation And Publication-Class Adapter Expansion

## Purpose
Narrow the next OxFml <-> OxFunc adapter-expansion lane after the first pinned 45-scenario corpus by adding real grouped-aggregation adapter cases and publication-class return-surface evidence for functions that are no longer blocked on missing OxFunc kernels.

## Position and Dependencies
- **Depends on**: `W049`, `W050`, `W042`
- **Blocks**: none
- **Cross-repo**: bounded OxFml <-> OxFunc adapter-evidence packet for `GROUPBY`, `PIVOTBY`, helper bind-time rejection parity, and publication-sensitive result classes

## Scope
### In scope
1. Add real OxFml adapter cases for `GROUPBY`.
2. Add real OxFml adapter cases for `PIVOTBY`.
3. Add bind-time rejection adapter coverage for helper forms that Excel rejects before evaluation.
4. Tighten OxFml-side publication-class evidence for `HYPERLINK`.
5. Tighten OxFml-side result-class preservation for `IMAGE`.
6. Record any concrete OxFunc mismatch exposed by those bounded adapter and return-surface cases.

### Out of scope
1. Broad one-shot closure of the full `GROUPBY` / `PIVOTBY` option matrix.
2. Final generic callable ABI redesign.
3. Full rich-value model closure for publication-sensitive functions.
4. Broad UI/rendering behavior for host presentation.

## Deliverables
1. Deterministic adapter cases for `GROUPBY`:
   - built-in aggregation callable lane such as `SUM`
   - prepared lambda lane if admitted by the current carrier
   - at least one totals/filter/header/sort-sensitive lane
2. Deterministic adapter cases for `PIVOTBY`:
   - default callable-backed pivot lane
   - at least one totals/filter/header-band lane
3. Deterministic adapter cases for bind-time helper rejection:
   - duplicate `LET` names
   - duplicate `LAMBDA` parameter names
   - malformed helper lambda declarations already pinned in `W038`
4. Updated return-surface evidence for `HYPERLINK` publication intent and `IMAGE` rich-value/publication classification.
5. Updated OxFml -> OxFunc note text that records whether the bounded expansion exposed any concrete seam mismatch.

## Gate Model
### Entry gate
- `W049` / `W050` have already established a real OxFml-backed adapter and a pinned first-wave corpus.
- OxFunc now treats admitted helper-family lanes as real and is asking for bounded grouped-aggregation and publication-class expansion rather than another generic callable round.

### Exit gate
- At least one real `GROUPBY` and one real `PIVOTBY` adapter family exist through the live OxFml parser/binder/preparation/evaluation path.
- Bind-time helper rejection parity is explicitly exercised in adapter artifacts where Excel rejects before evaluation.
- `HYPERLINK` and `IMAGE` have explicit OxFml-side return-surface evidence that preserves publication intent or rich-value class without collapsing them to plain text.
- Any remaining disagreement with OxFunc is narrowed to concrete field/behavior mismatches rather than a broad adapter gap.

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
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - real OxFml-backed adapter cases now exist for:
    - `GROUPBY` default callable lane
    - `GROUPBY` sort-sensitive lane
    - `PIVOTBY` default callable lane
    - `PIVOTBY` filter-and-zero-totals-sensitive lane
  - helper bind-time rejection parity is now widened for duplicate `LAMBDA` parameter names and malformed `LAMBDA` parameter declarations, but broader malformed helper-form families are still not all exercised through the bounded adapter corpus
  - `HYPERLINK` publication intent and explicit rich-value packet classification are now evidenced locally, but `IMAGE` still lacks equivalent end-to-end evaluator/adapter evidence because there is no local admitted `IMAGE(...)` function lane exercised here yet
  - OxFunc has narrowed the ask and the first local slice is real, but the bounded expansion lane is not yet at gate
- claim_confidence: provisional
