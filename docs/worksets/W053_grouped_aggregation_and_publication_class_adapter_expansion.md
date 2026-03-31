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
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: partial
- open_lanes:
  - real OxFml-backed adapter cases now exist for:
    - `GROUPBY` default callable lane via inline `LAMBDA(x,SUM(x))`
    - `GROUPBY` built-in aggregation callable lane via bare `SUM`
    - `GROUPBY` visible-header lane via bare built-in aggregation callable carriage
    - `GROUPBY` hierarchical-subtotal lane via bare built-in aggregation callable carriage
    - `GROUPBY` sort-sensitive lane for both inline `LAMBDA(...)` and bare built-in aggregation callable carriage
    - `GROUPBY` filtered descending-value sort lane via bare built-in aggregation callable carriage
    - `GROUPBY` tabular-subtotal runtime rejection lane as an explicit adapter-visible evaluation failure
    - `PIVOTBY` default callable lane via inline `LAMBDA(x,SUM(x))`
    - `PIVOTBY` built-in aggregation callable lane via bare `SUM`
    - `PIVOTBY` visible-header band lane via bare built-in aggregation callable carriage
    - `PIVOTBY` row/column-total sort lane via bare built-in aggregation callable carriage
    - `PIVOTBY` filter-and-zero-totals-sensitive lane for both inline `LAMBDA(...)` and bare built-in aggregation callable carriage
  - helper bind-time rejection parity is now exercised for duplicate `LET` names, duplicate `LAMBDA` parameter names, and malformed `LAMBDA` parameter declarations, including a deterministic fixture corpus in `crates/oxfml_core/tests/fixtures/w053_grouped_aggregation_cases.json`
  - `HYPERLINK` publication intent, generic extended top-level return-surface preservation, explicit `_webimage` rich-value packet classification, and end-to-end local `IMAGE(...)` evaluator/host/adapter evidence are now exercised locally
  - no new concrete OxFunc mismatch was exposed by the widened local `GROUPBY` / `PIVOTBY` / publication-class corpus
  - OxFunc now treats the widened `W053` corpus as sufficient for the current grouped-aggregation regression floor; future reopening is mismatch-driven
  - no further honest local `W053` execution remains on the OxFml side beyond future mismatch-driven reopen; the workset remains `integration_completeness: partial` only because surrounding shared seam packets still need broader promotion/freeze work
- claim_confidence: provisional
