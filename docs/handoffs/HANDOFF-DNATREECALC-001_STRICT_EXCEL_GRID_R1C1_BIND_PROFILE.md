# HANDOFF-DNATREECALC-001_STRICT_EXCEL_GRID_R1C1_BIND_PROFILE

Status: Open
Direction: inbound
Source: DnaTreeCalc grid-planning promotion
Target: OxFml
Target workset: W077 strict-excel-grid BindProfile and R1C1 identity

Ask: Give bound formulas a **caller-independent symbolic reference form** so that all cells in a region with the same R1C1-relative formula share one bound artifact and one compiled plan, instantiated per cell by `(caller_row, caller_col)` — plus A1 `$` fidelity, grid bounds, and plan caching. All behavior changes sit behind a typed **`BindProfile`** so existing channels and the TreeCalc lineage are bit-for-bit unaffected.

Context: OxCalc `CORE_ENGINE_GRID_MODEL.md` makes the R1C1-relative normal form the canonical formula identity for `strict-excel-grid`. Template sharing is the load-bearing scale mechanism; file shared-formula regions, fill/flash-fill regions, and engine-coalesced regions must all bind to the same caller-independent artifact when their normal form is equal.

Evidence from recon:
- R1C1 relative parts currently resolve at bind.
- The bind context fingerprint includes caller position.
- A1 `$` fidelity is stripped/discarded in current binding paths.
- Evaluation already threads caller row/col and dereferences through the reference-system provider, so dereference-time offset resolution is the natural target.
- Non-default host reference syntax currently has an incremental green-tree reuse penalty that would become common under the grid profile.

## Required OxFml work

1. **`BindProfile` on `BindContext`**: `{ reference_syntax, a1_relative_to_caller, preserve_a1_dollar_fidelity, grid_bounds, symbolic_refs }`. Defaults must reproduce current behavior exactly; `strict-excel-grid` turns on the new semantics.
2. **Symbolic bound references**: preserve relative parts as offsets instead of resolving them to absolute coordinates during bind when `symbolic_refs` is true. Offset resolution moves to dereference/evaluation time, where caller coordinates are already available.
3. **Caller-independent identity**: under symbolic grid bind, exclude caller anchor from bind fingerprint/hash for anchor-relative formulas, so one `BoundFormula` and semantic plan can serve a template region.
4. **Compiled-plan caching**: cache `CompiledFormulaPlan` by bind hash plus catalog identity and instantiate per cell by caller coordinates; prepare cost scales with distinct templates, not cells.
5. **A1 `$` fidelity and caller-relative A1**: preserve per-axis absolute/relative flags from A1 entry and make R1C1 normal form derivable from A1 text.
6. **Grid bounds**: `GridBounds { max_row: 1_048_576, max_col: 16_384 }`; out-of-bounds entry or translation yields `#REF!` per OxCalc grid model.
7. **Translation/rebind API**: given a bound artifact and coordinate delta, produce the translated artifact or `#REF!`-substituted result for fill, paste, region stamping, and insert/delete shifting.
8. **Profile-parity housekeeping**: fix non-default syntax-profile reuse penalties and keep `FormulaChannelKind` orthogonal to profile selection.

## Acceptance shape

- Existing default channels and TreeCalc profile fixtures remain byte-for-byte equivalent at public outputs.
- Focused grid-profile fixtures prove two different caller cells with the same R1C1-relative normal form share caller-independent identity.
- A1 display/recomposition preserves `$` fidelity.
- Bounds fixtures reject or translate to `#REF!` deterministically.
- Workset W077 records exact public type/API names before implementation closes.