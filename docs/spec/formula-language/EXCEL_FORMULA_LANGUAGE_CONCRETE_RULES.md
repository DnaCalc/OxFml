# Excel Formula Language Concrete Rules

## 1. Purpose
This document defines concrete worksheet-formula language rules for the Excel-first model.

It tightens `ECM-FML-001..004` into implementation-facing rule statements tied to:
1. requirement lanes (`XLS-CF-FL-*`), and
2. source evidence ids (`ECS-*`, `REFX-*`, `EMP-*`).

This rule corpus should be read together with:
1. `OXFML_FORMULA_ENGINE_ARCHITECTURE.md`
2. `OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
3. `../OXFML_FORMALIZATION_AND_VERIFICATION.md`

Working interpretation rule:
1. this document states the Excel-facing rule corpus,
2. the architecture documents define the intended OxFml internal model,
3. archived empirical plans are evidence support, not bootstrap authority.

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

## 3. Evidence Posture
Rule wording in this document is canonical only at the policy level. Wave-specific runs, matrices, and dated execution summaries are archive material.

Working rule:
1. implementation and design bootstrap should start from the rule statements in Section 2 plus the architecture documents,
2. rule status promotion and evidence review should use the conformance matrix together with archived empirical baselines,
3. wave summaries and run-pack-specific observations must not be treated as bootstrap authority.

Canonical archive pointer:
1. `archive/EXCEL_FORMULA_LANGUAGE_EMPIRICAL_BASELINES.md`

## 4. Operator Precedence Baseline (Worksheet Formula Context)
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

## 5. Helper-Form Coverage Baseline (Draft)
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

### 5.1 Compile-Time Reducibility Boundary (Planning Note)
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

### 5.2 OxFunc Boundary Relevance
Several formula-language rules have direct OxFunc-boundary consequences:
1. `FML-R-003`, `FML-R-004`, and `FML-R-005` affect how `@`, `#`, and spill-linked results survive into prepared evaluation structures,
2. `FML-R-008` and `FML-R-009` affect bind outputs and reference identity,
3. `FML-R-012` affects argument preparation and admission-vs-runtime-error classification.

These cross-lane effects must remain explicit during future tightening passes.

### 5.3 Function-Call Admission vs Runtime Error Boundary (Planning Note)
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

## 6. Open Items for Next Tightening Pass
1. Replicate scoped-name and precedence lanes across target channels/builds to verify current provisional policy wording.
2. Expand external/workbook reference lane to cover additional link-update policy variants and workbook-open/closed permutations across builds/channels (same-build baseline captured in `EMP-0011`).
3. Establish a true linked-data fixture path for `P2-FML-002` so dot-field semantics can be split by linked vs non-linked contexts.
4. Expand `P2-FML-008` spill-blocking/update scenarios to support `FML-R-005` promotion from `draft`.
5. Replicate argument-gap and normalization lanes across additional target builds/channels for status promotion to validated.
6. Execute `P2-FML-011` function-admission/coercion edge matrix (`SIN`/`ASIN` seeds), then split stable sub-rules from remaining provisional rows.

## 7. Conformance Matrix and Archive Evidence
This rule set is operationalized by:
1. `EXCEL_FORMULA_LANGUAGE_CONFORMANCE_MATRIX.csv` (rule status, evidence strength, probe bindings, promotion criteria).
2. `archive/EXCEL_FORMULA_LANGUAGE_EMPIRICAL_BASELINES.md` (wave summaries and dated baseline observations).
3. `archive/EXCEL_FORMULA_LANGUAGE_PASS2_PROBE_PLAN.md` (deferred empirical execution plan with scenario-level objectives).
4. `archive/EXCEL_FORMULA_LANGUAGE_PASS2_SCENARIO_SEED.csv` (seed scenario rows for pass-2 execution).

Primary unresolved closures currently depend on:
1. `P2-FML-001` convergence (double-comma argument-gap across builds/channels),
2. `P2-FML-002` linked-data semantic branch (true linked-data fixture still missing),
3. `P2-FML-006` link-update/open-state policy expansion,
4. `P2-FML-008` spill-blocking/update expansion for `FML-R-005`,
5. cross-build replay of `P2-FML-003`, `P2-FML-005`, `P2-FML-009`, and `P2-FML-010`.
6. `P2-FML-011` required-argument omission vs runtime error mapping (`FML-R-012`).
