# Excel Formula Language Concrete Rules

## 1. Purpose
This document defines concrete worksheet-formula language rules for the Excel-first model.

It tightens `ECM-FML-001..004` into implementation-facing rule statements tied to:
1. requirement lanes (`XLS-CF-FL-*`), and
2. source evidence ids (`ECS-*`, `REFX-*`, `EMP-*`).

## 2. Rule Set

| rule_id | statement | requirement_ids | evidence_ids | status |
|---|---|---|---|---|
| FML-R-001 | Formula parser must recognize reference operators `:`, `,`, and intersection (space) as distinct operators. | XLS-CF-FL-001;XLS-CF-FL-002 | ECS-003;ECS-008;ECS-EB-033;ECS-EB-040 | provisional |
| FML-R-002 | Reference operators are parsed in a precedence tier above arithmetic/comparison operators. | XLS-CF-FL-001 | ECS-003;ECS-008;ECS-EB-040 | provisional |
| FML-R-003 | `@` is parsed as explicit implicit-intersection operator syntax and must not be discarded during parse normalization. | XLS-CF-FL-003 | ECS-004;ECS-007;ECS-EB-038 | provisional |
| FML-R-004 | `#` is parsed as spilled-range suffix operator (`<ref>#`) and must reject malformed prefix usage such as `=#A1`. | XLS-CF-FL-004 | ECS-005;ECS-006 | provisional |
| FML-R-005 | Dynamic-array spill behavior must be represented at formula-language boundary with spill reference updates and visible spill errors. | XLS-CF-FL-005 | ECS-006;ECS-007;ECS-EB-038 | draft |
| FML-R-006 | Parser grammar coverage must stay aligned with the formal MS-XLSX grammar anchor; any observed widening must be explicit and version-scoped. | XLS-CF-FL-006 | ECS-008;ECS-009;REFX-001;ECS-EB-034;ECS-EB-036;EMP-0011 | provisional |
| FML-R-007 | Cell-formula storage/normalization behavior (entered text vs stored formula) must be captured explicitly in conformance outputs. | XLS-CF-FL-007 | ECS-009;REFX-001;ECS-EB-039;ECS-EB-038 | provisional |
| FML-R-008 | Workbook/sheet name resolution must follow Excel name-scope behavior and collision precedence. | XLS-CF-FL-008 | ECS-010;ECS-011;ECS-008;ECS-EB-035 | provisional |
| FML-R-009 | Structured references are first-class formula syntax (`Table[Col]`, `[@Col]`, qualifiers) and participate in normal parse/bind/eval. | XLS-CF-FL-009 | ECS-012;ECS-013;ECS-014;ECS-EB-037 | provisional |
| FML-R-010 | `=SUM(A1,,B1)` behavior is treated as build-scoped provisional ambiguity; parser policy must remain configurable until resolved. | XLS-CF-FL-010 | EMP-0001;ECS-EB-031 | provisional |
| FML-R-011 | Dot-field syntax (`=A1.Price`) is tracked as syntax-accepted in current evidence, with runtime semantics constrained by linked-data context. | XLS-CF-FL-011 | ECS-024;ECS-025;EMP-0002;ECS-EB-032 | provisional |
| FML-R-012 | Function-call conformance must distinguish formula-entry rejection from accepted-formula runtime errors, including required-argument omission and array-lifted element error behavior. | XLS-CF-FL-012 | ECS-008;ECS-109;ECS-113;ECS-114;ECS-115 | provisional |

## 3. Parse Acceptance Baseline (Wave 1)
Empirical registry anchor:
- `research/runs/20260228-180047-excel-compat-empirical-pass-01/outputs/formula_parse_wave1/`

Baseline outcomes from `ECS-EB-028`:
1. Accepted: range, union, intersection, `@`, `#`, `@`+`#`, LET, inline LAMBDA invoke, structured table references.
2. Rejected: malformed double-colon range token, malformed `#` prefix usage, malformed LAMBDA invocation, malformed structured-reference brackets.

Evidence file:
- `ECS-EB-028_formula_parse_acceptance_corpus_wave1.csv`

### 3.1 Seeded Parse Acceptance Matrix (Wave 1)
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

## 4. Normalization Baseline (Wave 1)
From `ECS-EB-029`:
1. Function names are normalized to canonical uppercase in stored formula text.
2. Structured-reference identifiers are normalized to canonical table/column casing in stored formula text.

Evidence file:
- `ECS-EB-029_formula_normalization_capture_wave1.csv`

### 4.1 Normalization Matrix (Wave 1)
| case_id | formula_input | stored_formula_final | normalization_changed |
|---|---|---|---|
| FPCW1-017 | `=sUm(a1:b2)` | `=SUM(A1:B2)` | true |
| FPCW1-018 | `=sum(tblparse[amount])` | `=SUM(TblParse[Amount])` | true |

## 5. Ambiguity and Provisional Lanes
From `ECS-EB-030`:
1. `=SUM(A1,,B1)` observed `accepted` in this environment (mismatch vs seeded reject expectation).
2. `=A1.Price` observed syntax acceptance with `#FIELD!` runtime outcome in non-linked-type context.
3. `=A1 B1` accepted and produced `#NULL!` in tested single-cell intersection scenario.

Evidence file:
- `ECS-EB-030_grammar_ambiguity_probe_wave1.csv`

### 5.1 Ambiguity Probe Matrix (Wave 1)
| case_id | formula_input | expected | observed | final_display_text | result_class |
|---|---|---|---|---|---|
| FPCW1-020 | `=SUM(A1,,B1)` | rejected | accepted | `5` | mismatch |
| FPCW1-013 | `=A1.Price` | probe | accepted | `#FIELD!` | probe |
| FPCW1-019 | `=A1 B1` | accepted | accepted | `#NULL!` | matches_expected |

## 6. Pass-2 Outcomes (20260302-070309)
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

### 6.1 Provisional Policy Wording (Pass-4 Sync)
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

## 7. Operator Precedence Baseline (Worksheet Formula Context)
Current precedence baseline for parser/evaluator alignment:
1. Reference operators (`:`, `,`, space intersection)
2. Unary `+`, unary `-`
3. `%`
4. `^`
5. `*`, `/`
6. `+`, `-`
7. `&`
8. Comparison operators (`=`, `<>`, `<`, `>`, `<=`, `>=`)

Anchor:
- `ECS-003` plus formal grammar cross-check via `ECS-008`.

## 8. Helper-Form Coverage Baseline (Draft)
| construct | sample_shape | source_class | evidence_ids | observed_state | notes |
|---|---|---|---|---|---|
| LET | `=LET(x,1,x+2)` | authoritative_behavioral + empirical | ECS-041;ECS-008 | wave1_accept | Baseline LET parse acceptance confirmed in wave1. |
| LAMBDA invoke | `=LAMBDA(x,x+1)(2)` | authoritative_behavioral + empirical | ECS-042;ECS-008 | wave1_accept | Inline invocation accepted in wave1. |
| LAMBDA malformed | `=LAMBDA(x,x+1)(1,2` | empirical | ECS-008 | wave1_reject | Malformed invocation rejected in wave1. |
| MAP | `=MAP(A1:A3,LAMBDA(x,x+1))` | authoritative_behavioral + empirical | ECS-041;ECS-042;ECS-EB-034 | pass2_accept | Accepted in pass-2 corpus. |
| BYROW | `=BYROW(A1:C3,LAMBDA(r,SUM(r)))` | authoritative_behavioral + empirical | ECS-041;ECS-042;ECS-EB-034 | pass2_accept | Accepted in pass-2 corpus. |
| BYCOL | `=BYCOL(A1:C3,LAMBDA(c,SUM(c)))` | authoritative_behavioral + empirical | ECS-041;ECS-042;ECS-EB-034 | pass2_accept | Accepted in pass-2 corpus. |
| SCAN | `=SCAN(0,A1:A3,LAMBDA(a,b,a+b))` | authoritative_behavioral + empirical | ECS-041;ECS-042;ECS-EB-034 | pass2_accept | Accepted in pass-2 corpus. |
| REDUCE | `=REDUCE(0,A1:A3,LAMBDA(a,b,a+b))` | authoritative_behavioral + empirical | ECS-041;ECS-042;ECS-EB-034 | pass2_accept | Accepted in pass-2 corpus. |

Coverage note:
1. Public formal grammar anchors are incomplete for some modern helper-form details.
2. Helper-form completeness therefore depends on mixed formal + behavioral + empirical evidence.

### 8.1 Compile-Time Reducibility Boundary (Planning Note)
Formula-language and function metadata together should be sufficient to classify whether an expression may be reduced before runtime evaluation.

Working rule:
1. A formula subtree is compile-time reducible only when:
   - all inputs are constant-closed, and
   - all functions/operators in the subtree are classified `const_foldable_when_closed`.
2. Subtrees containing reference-dependent or context-dependent functions must be deferred to runtime evaluation.

Illustrative examples:
1. `=SIN(4)` and `=SIN(2*PI())` can be reduced immediately after parse/bind if folding is enabled.
2. `=SIN(A1)` must wait for evaluation because argument resolution depends on runtime reference values.
3. `=ROW()` and `=NOW()` must not be treated as deterministic compile-time reductions because they depend on caller/time context.

Cross-lane dependency:
1. Final policy depends on function-definition metadata in `../../../../../OxFunc/docs/function-lane/EXCEL_FUNCTION_DEFINITION_PRELIM_SPEC.md` (not parser grammar alone).

### 8.2 Function-Call Admission vs Runtime Error Boundary (Planning Note)
This lane captures a missing-but-critical distinction:
1. parse-time formula rejection (`cannot enter formula` class), versus
2. accepted formula with runtime error result (`#VALUE!`, `#NUM!`, etc.).

Canonical seed examples:
1. `=SIN()` should be tracked as parse/admission failure (required-argument omission).
2. `=SIN("asd")` should be tracked as formula-accepted with runtime coercion/error outcome.
3. `=SIN({1,"asd",3})` should be tracked for array-lift/error propagation policy (`single error` vs `elementwise result array with internal error elements`).
4. `=ASIN(2)` should be tracked for numeric-domain error mapping (`#NUM!` expectation in common builds).

Evidence posture:
1. Current public sources provide only thin direct guidance for this lane.
2. Therefore this rule remains provisional until dedicated empirical matrices are promoted.

## 9. Open Items for Next Tightening Pass
1. Replicate scoped-name and precedence lanes across target channels/builds to verify current provisional policy wording.
2. Expand external/workbook reference lane to cover additional link-update policy variants and workbook-open/closed permutations across builds/channels (same-build baseline captured in `EMP-0011`).
3. Establish a true linked-data fixture path for `P2-FML-002` so dot-field semantics can be split by linked vs non-linked contexts.
4. Expand `P2-FML-008` spill-blocking/update scenarios to support `FML-R-005` promotion from `draft`.
5. Replicate argument-gap and normalization lanes across additional target builds/channels for status promotion to validated.
6. Execute `P2-FML-011` function-admission/coercion edge matrix (`SIN`/`ASIN` seeds), then split stable sub-rules from remaining provisional rows.

## 10. Conformance Matrix and Pass-2 Plan
This rule set is operationalized by:
1. `EXCEL_FORMULA_LANGUAGE_CONFORMANCE_MATRIX.csv` (rule status, evidence strength, probe bindings, promotion criteria).
2. `EXCEL_FORMULA_LANGUAGE_PASS2_PROBE_PLAN.md` (deferred empirical execution plan with scenario-level objectives).
3. `EXCEL_FORMULA_LANGUAGE_PASS2_SCENARIO_SEED.csv` (seed scenario rows for pass-2 execution).

Primary unresolved closures currently depend on:
1. `P2-FML-001` convergence (double-comma argument-gap across builds/channels),
2. `P2-FML-002` linked-data semantic branch (true linked-data fixture still missing),
3. `P2-FML-006` link-update/open-state policy expansion,
4. `P2-FML-008` spill-blocking/update expansion for `FML-R-005`,
5. cross-build replay of `P2-FML-003`, `P2-FML-005`, `P2-FML-009`, and `P2-FML-010`.
6. `P2-FML-011` required-argument omission vs runtime error mapping (`FML-R-012`).
