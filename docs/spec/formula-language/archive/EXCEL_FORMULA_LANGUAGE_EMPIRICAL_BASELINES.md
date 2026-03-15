# Excel Formula Language Empirical Baselines

Archive status:
1. This document retains wave-specific empirical observations and run-pack summaries.
2. It is not part of the canonical OxFml bootstrap read set.

## 1. Purpose
Preserve the historical empirical baseline that informed the live OxFml formula-language rule corpus.

The canonical rule statements live in `../EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md`.
This archive file keeps the supporting wave summaries, seeded matrices, and dated execution notes.

## 2. Parse Acceptance Baseline (Wave 1)
Empirical registry anchor:
- `research/runs/20260228-180047-excel-compat-empirical-pass-01/outputs/formula_parse_wave1/`

Baseline outcomes from `ECS-EB-028`:
1. Accepted: range, union, intersection, `@`, `#`, `@`+`#`, LET, inline LAMBDA invoke, structured table references.
2. Rejected: malformed double-colon range token, malformed `#` prefix usage, malformed LAMBDA invocation, malformed structured-reference brackets.

Evidence file:
- `ECS-EB-028_formula_parse_acceptance_corpus_wave1.csv`

### 2.1 Seeded Parse Acceptance Matrix (Wave 1)
| case_id | formula_input | expected | observed | notes |
|---|---|---|---|---|
| FPCW1-001 | `=SUM(A1:B2)` | accepted | accepted | Baseline range operator acceptance. |
| FPCW1-002 | `=SUM((A1:A2,B1:B2))` | accepted | accepted | Union reference acceptance in function context. |
| FPCW1-003 | `=SUM(A1:C1 A1:A3)` | accepted | accepted | Space intersection operator acceptance. |
| FPCW1-004 | `=SUM(A1::B2)` | rejected | rejected | Malformed range token rejected. |
| FPCW1-005 | `=@A1:A3` | accepted | accepted | Implicit intersection operator acceptance. |
| FPCW1-006 | `=A1#` | accepted | accepted | Spill-reference suffix acceptance. |
| FPCW1-007 | `=@A1#` | accepted | accepted | Combined `@` and `#` accepted. |
| FPCW1-008 | `=#A1` | rejected | rejected | Malformed prefix `#` rejected. |
| FPCW1-009 | `=LET(x,1,x+2)` | accepted | accepted | LET baseline acceptance. |
| FPCW1-010 | `=LAMBDA(x,x+1)(2)` | accepted | accepted | Inline LAMBDA invoke accepted. |
| FPCW1-011 | `=LAMBDA(x,x+1)(1,2` | rejected | rejected | Malformed LAMBDA invocation rejected. |
| FPCW1-016 | `=SUM(TblParse[[#All],[Amount])` | rejected | rejected | Malformed structured-ref bracket sequence rejected. |

## 3. Normalization Baseline (Wave 1)
From `ECS-EB-029`:
1. Function names are normalized to canonical uppercase in stored formula text.
2. Structured-reference identifiers are normalized to canonical table/column casing in stored formula text.

Evidence file:
- `ECS-EB-029_formula_normalization_capture_wave1.csv`

### 3.1 Normalization Matrix (Wave 1)
| case_id | formula_input | stored_formula_final | normalization_changed |
|---|---|---|---|
| FPCW1-017 | `=sUm(a1:b2)` | `=SUM(A1:B2)` | true |
| FPCW1-018 | `=sum(tblparse[amount])` | `=SUM(TblParse[Amount])` | true |

## 4. Ambiguity and Provisional Lanes
From `ECS-EB-030`:
1. `=SUM(A1,,B1)` observed `accepted` in this environment (mismatch vs seeded reject expectation).
2. `=A1.Price` observed syntax acceptance with `#FIELD!` runtime outcome in non-linked-type context.
3. `=A1 B1` accepted and produced `#NULL!` in tested single-cell intersection scenario.

Evidence file:
- `ECS-EB-030_grammar_ambiguity_probe_wave1.csv`

### 4.1 Ambiguity Probe Matrix (Wave 1)
| case_id | formula_input | expected | observed | final_display_text | result_class |
|---|---|---|---|---|---|
| FPCW1-020 | `=SUM(A1,,B1)` | rejected | accepted | `5` | mismatch |
| FPCW1-013 | `=A1.Price` | probe | accepted | `#FIELD!` | probe |
| FPCW1-019 | `=A1 B1` | accepted | accepted | `#NULL!` | matches_expected |

## 5. Pass-2 Outcomes (20260302-070309)
Empirical registry anchor:
- `research/runs/20260302-070309-excel-formula-language-pass2-pack-01/outputs/formula_parse_pass2/`

Execution summary:
1. Scenario rows executed: `37/37`.
2. Observed accepted: `35`.
3. Observed rejected: `2`.
4. Mismatch rows: `0`.
5. Run-failed rows: `0`.

Key behavior captures:
1. Argument-gap lane (`P2-FML-001`): `SUM` and `LET` gap variants parsed and evaluated as accepted (`FMLP2-001..005`).
2. Dot-field lane (`P2-FML-002`): parse accepted for both `A1.Price` and `FIELDVALUE(A1,"Price")`, returning `#FIELD!` in tested harness contexts (`FMLP2-006..009`).
3. Intersection/precedence lane (`P2-FML-003`/`P2-FML-010`):
   - `=A1 B1` and extra-space variant accepted with `#NULL!`.
   - `=-2^2` observed `4`.
   - `=1+2&3` observed `33`.
4. Helper-form lane (`P2-FML-004`): `MAP`, `BYROW`, `SCAN`, `BYCOL`, `REDUCE` parsed/evaluated as accepted; malformed LAMBDA shape rejected (`FMLP2-018`).
5. Structured reference lane (`P2-FML-007`): baseline accept/reject behaved as seeded.
6. `@`/`#` lane (`P2-FML-008`):
   - `=@A1#` stored as `=A1#` and evaluated against spill anchor.
   - `=@SEQUENCE(3)` stored as `=SEQUENCE(3)`.
   - `=A1#` on non-spill scalar produced `#REF!`.
7. Targeted lane rerun (`pass-2c`) added explicit lane controls:
   - linked-data conversion attempts are now captured as operation-trace evidence (`allowed_error` in current environment) for `FMLP2-008/009`,
   - dual-scope name setup produced `=MyName -> 4` and `=Sheet1!MyName -> 1` in current build (`FMLP2-019/020`),
   - workbook-present external reference resolved to `77` when support workbook was explicitly opened (`FMLP2-021`), while missing workbook remained `#REF!` (`FMLP2-022`).

Primary artifacts:
1. `FORMULA_PARSE_PASS2_RESULTS.csv`
2. `SEED_TO_EXECUTED_MAPPING_PASS2.csv`
3. `PASS2_EXECUTION_REPORT.md`
4. `MANUAL_PREP_PASS2B_REPORT.md`
5. `TARGETED_PASS2C_LANES_REPORT.md`

### 5.1 Provisional Policy Wording (Pass-4 Sync)
1. `FML-R-008` scoped-name behavior is now documented as a build-scoped provisional policy:
   - in current evidence, unqualified `=MyName` bound to the workbook-scoped name (`4`),
   - `=Sheet1!MyName` bound to the sheet-scoped name (`1`) and stored as a normalized workbook-qualified token.
2. `FML-R-006` external-reference behavior is now documented as:
   - parser accepts workbook references in both present and missing-workbook forms,
   - workbook-present resolution in this harness requires explicit support-workbook open (`open_support_workbook`),
   - missing workbook produced `#REF!` in the current build,
   - `update_links=0` and `update_links=3` produced the same observed value in this environment (`EMP-0011`).
3. `FML-R-011` dot-field behavior remains provisional with explicit runner limitation:
   - linked-data conversion attempts are trace-captured,
   - conversion currently fails in this environment (`allowed_error`),
   - non-linked branch captures `#FIELD!` for both `A1.Price` and `FIELDVALUE`.
