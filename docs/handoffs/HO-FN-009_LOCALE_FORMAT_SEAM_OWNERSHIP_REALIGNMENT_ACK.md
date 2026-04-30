# HO-FN-009 Locale/Format Seam Ownership Realignment Ack

## Direction
- **From**: `OxFml`
- **To**: `OxFunc`
- **Responds to**: `../OxFunc/docs/handoffs/HO-FN-009_locale_format_seam_ownership_realignment.md`
- **Source workset**: `OxFunc/W082`

## Acknowledgement
OxFml agrees with the ownership split.

OxFunc owns locale-sensitive function semantics and the typed seam vocabulary:
1. `LocaleFormatContext`
2. `FormatProfile`
3. `WorkbookDateSystem`
4. `LocaleValueParser`
5. `FormatCodeEngine`

OxFml/FEC owns the concrete parser and formatter capability supplied into that seam.

## OxFml Owner
The current OxFml implementation owner is:
1. `crates/oxfml_core/src/format/engine.rs`
2. `OxFmlLocaleValueParser`
3. `OxFmlFormatCodeEngine`
4. OxFml-owned `FormatProfile` constructors
5. `oxfml_locale_context(...)`

## Call Paths Requiring LocaleFormatContext
The OxFml call paths that need explicit locale-format capability are:
1. evaluator `EvaluationContext`
2. `TypedContextQueryBundle`
3. `SingleFormulaHost`
4. `RuntimeEnvironment`
5. `RuntimeSessionFacade`
6. replay fixture and retained-witness runners
7. runtime and replay consumer facade tests
8. `test_support::oxfunc_adapter`
9. verification/publication formatting where a locale-aware display projection is requested

## Migration Note Owner
Downstream callers own supplying workbook/host locale profile and workbook date system through `TypedContextQueryBundle`.

DNA OneCalc is the immediate caller-facing migration owner for the corpus/programmatic verification lane. Missing locale context remains an explicit missing-capability condition; OxFml must not add a hidden fallback locale to create verification truth.

## OxFunc Follow-Up
No OxFunc-side seam change is requested by this acknowledgement.

## Adjacent Non-Blocking Observation
OxFml reviewed the ordinary array-valued adapter path requested in the handoff prompt.

Current local evidence shows ordinary array-valued arguments are preserved before OxFunc:
1. `crates/oxfml_core/tests/w049_oxfunc_adapter_tests.rs` classifies `=SUM(A1:A2)` as `ArrayLike`.
2. The evaluator passes ordinary array values as `CallArgValue::Eval(EvalValue::Array(...))`.
3. Scalarization remains tied to explicit implicit-intersection/caller-context paths, not ordinary value-required array arguments.

This observation is not part of the locale-format blocker.
