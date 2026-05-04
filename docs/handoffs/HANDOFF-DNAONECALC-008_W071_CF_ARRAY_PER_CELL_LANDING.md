*Posted by Codex agent on behalf of @govert*

# HANDOFF-DNAONECALC-008: W071 CF Array Per-Cell Landing

Status: `filed`
Source repo/workset: `OxFml/W071`
Target repo/workset: `DnaOneCalc/TBD`
Filed date: `2026-05-04`
Related inbound: `../DnaOneCalc/docs/HANDOFF_OXFML_CF_ARRAY_PER_CELL.md`

## Purpose

Record OxFml's response to the DnaOneCalc request that conditional-formatting rules on array results publish per-cell outcomes instead of evaluating once against the whole array value.

## OxFml Landing Summary

`VerificationPublicationSurface` now carries:

```rust
pub array_cell_format: Option<ArrayCellFormatGrid>
```

The carrier contains one row-major `ArrayCellFormat` per array cell:

```rust
pub struct ArrayCellFormat {
    pub effective_display_text: String,
    pub effective_font_color: Option<String>,
    pub effective_fill_color: Option<String>,
    pub data_bar: Option<DataBarFill>,
    pub icon: Option<CfIcon>,
}
```

For W071, OxFml populates per-cell display/font/fill outcomes for already-supported scalar/operator/predicate rules. `data_bar` and `icon` are carrier slots for W072 aggregate visualization work and remain `None` until those computations land.

## Supported Per-Cell Rules In This Landing

1. Operator rules such as `cell_value` / `greaterThan`.
2. Predicate rules from W070: `blanks`, `noBlanks`, `errors`, `noErrors`, and `dates`.
3. Relative date rules reuse the same `now_serial` supplied to publication.
4. `ArrayCellValue::EmptyCell` is treated as blank for `blanks`.

For 1x1 arrays, `array_cell_format.rows[0][0]` matches the whole-cell CF fields.

## Evidence

Relevant OxFml files:

1. `crates/oxfml_core/src/publication/mod.rs`
2. `crates/oxfml_core/src/lib.rs`
3. `crates/oxfml_core/tests/conditional_formatting_array_tests.rs`

Focused validation:

1. `cargo test -p oxfml_core --test conditional_formatting_array_tests` - passed, 4 tests.
2. `cargo test -p oxfml_core --test conditional_formatting_predicate_tests` - passed, 4 tests.

## DnaOneCalc Follow-Up

DNA OneCalc can now bridge `array_cell_format` to the result-hero array browser:

1. apply per-cell `effective_font_color` / `effective_fill_color`,
2. use per-cell `effective_display_text` when present,
3. keep visualization `<NOT IMPL>` markers for aggregate families until W072 lands.

## Non-Claims

This handoff does not claim:

1. aggregate-context visualization rules,
2. data-bar or icon computation,
3. DNA OneCalc bridge/UI rendering,
4. worksheet-range CF semantics.
