# BUGREP-FML-001: Exponentiation Formula Corpus Mismatches

## Intake
- **Report id**: `BUGREP-FML-001`
- **Filed**: 2026-04-06
- **Source channel**: foundation_formula_corpus
- **Reporter/source**: Foundation formula corpus tests
- **Reported against ref**: `2dd48c72412797f01e34d4e4b9a1146cbddcf3cd`
- **Reported against kind**: commit
- **Canonical bug id**: `BUG-FML-001`
- **Status**: triaged

## Observed Symptom
Exponentiation-related worksheet formulas produce OxFml results that diverge from Excel in the current corpus run.

## Reproduction
Observed corpus rows:

| case_id | formula | oxfml_value | excel_value |
|---|---|---:|---:|
| `FTC-0003` | `=2^3^2` | `2` | `64` |
| `FTC-0004` | `=-2^2` | `-2` | `4` |
| `FTC-0008` | `=2^2*3` | `2` | `12` |
| `FTC-0010` | `=1+2*3^2` | `7` | `19` |

## Initial Ownership Read
- **Initial classification**: OxFml-owned bug
- **Reason**: current implementation evidence indicates the `^` operator is not represented in the OxFml token/parser/binder/evaluator stack even though the local formula-language baseline includes exponentiation.

## Links
1. `docs/bugs/streams/BUG-FML-001_exponentiation_formulas_diverge_from_excel.md`
2. `docs/spec/formula-language/EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md`
3. `docs/spec/formula-language/archive/EXCEL_FORMULA_LANGUAGE_EMPIRICAL_BASELINES.md`

## Triage Notes
The corpus `expected` values and explanatory notes are not treated as authoritative intake data for this report. Only case id, formula, OxFml value, and Excel value are carried forward.
