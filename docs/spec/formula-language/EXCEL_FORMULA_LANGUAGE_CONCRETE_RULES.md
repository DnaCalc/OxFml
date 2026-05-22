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
| FML-R-013 | Structured references require table-aware bind context; omitted table-name forms must not be resolved from syntax alone, and table identifiers must remain distinct from user-defined names. | XLS-CF-FL-009 | SPEC-discovered-ms-oe376-88e93023-48236;CONF-discovered-ms-oe376-220816-823374c7-1423 | provisional |
| FML-R-014 | `R1C1` formulas are a distinct formula channel and must use `R1C1`-style references rather than being treated only as an A1 display mode. | XLS-CF-FL-006 | CONF-discovered-ms-oe376-220816-823374c7-1434;SPEC-discovered-ms-oe376-88e93023-48474;SPEC-discovered-ms-oe376-88e93023-48487 | provisional |
| FML-R-015 | Name formulas and external-name formulas are distinct formula-bearing carriers; external-name formulas are narrower than generic external references and require explicit external-book identity. | XLS-CF-FL-008 | CONF-discovered-ms-oe376-220816-823374c7-0362;CONF-discovered-ms-oe376-220816-823374c7-0363;SPEC-discovered-ms-oe376-88e93023-48443;SPEC-discovered-ms-oe376-88e93023-48448;SPEC-discovered-ms-oe376-88e93023-48451 | provisional |
| FML-R-016 | Conditional-formatting and data-validation formulas are restricted formula-bearing sublanguages; they are similar but not safely identical, and their rule-host fields remain formula-semantic rather than display-only metadata. | XLS-CF-FL-006 | CONF-discovered-ms-oe376-220816-823374c7-1427;CONF-discovered-ms-oe376-220816-823374c7-1428;CONF-discovered-ms-oe376-220816-823374c7-1429;CONF-discovered-ms-oe376-220816-823374c7-1430;CONF-discovered-ms-oe376-220816-823374c7-1431 | provisional |
| FML-R-018 | Ordinary arithmetic operators must preserve array payloads and apply numeric coercion/error mapping elementwise for admitted array-lifted shapes rather than collapsing arrays to a single scalar before evaluation. | XLS-CF-FL-012 | evaluator_tests.rs;replay_fixture_tests.rs;replay_retained_and_host_policy_tests.rs | provisional |
| FML-R-019 | Bare-name and call-callee resolution across built-in functions, registered UDFs, workbook/sheet defined names, and defined-name `LAMBDA` values must be Excel-oracle-derived before product host namespaces map onto that lane. | XLS-CF-FL-008;XLS-CF-FL-012 | W074-CALC005-ORACLE-PLANNED | draft |

## 3. Current Local Floors For Newer Rule Families

### 3.1 `R1C1`
The current local `R1C1` floor is:
1. explicit `WorksheetR1C1` channel identity,
2. absolute and caller-anchor-relative cell-reference translation,
3. qualified area ranges built from that cell-reference floor,
4. preservation of absolute versus relative origin in normalized references.

Current residuals stay explicit in `OXFML_R1C1_FORMULA_CHANNEL.md`.

### 3.2 `CF` and `DV`
The current local `CF` / `DV` floor is:
1. distinct carrier kinds,
2. distinct restriction profiles,
3. explicit host-field facts for target ranges and rule/formula slots,
4. rejection of union, intersection, spill, and external-reference families for the admitted restricted floor.

Current residuals stay explicit in `OXFML_CF_DV_RESTRICTED_SUBLANGUAGES.md`.

## 4. Evidence Posture
Rule wording in this document is canonical only at the policy level. Wave-specific runs, matrices, and dated execution summaries are archive material.

### 3.3 Structured References
The current local structured-reference floor is:
1. explicit table-column selectors such as `Table1[Amount]`,
2. omitted-table-name current-row selectors such as `[@Amount]` with required enclosing-table context,
3. section-only selectors such as `Table1[#Headers]` and `Table1[#Totals]`,
4. first section-qualified multi-column selectors such as `Table1[[#All],[Amount]:[Tax]]` and `Table1[[#Data],[Amount]:[Tax]]`,
5. explicit bind rejection for illegal `#This Row` combinations and missing enclosing-table context.

Current structured-reference residuals stay explicit in `OXFML_STRUCTURED_REFERENCE_AND_TABLE_BOUNDARY.md` and `W036_structured_reference_and_table_formula_semantics.md`.

Working rule:
1. implementation and design bootstrap should start from the rule statements in Section 2 plus the architecture documents,
2. rule status promotion and evidence review should use the conformance matrix together with archived empirical baselines,
3. wave summaries and run-pack-specific observations must not be treated as bootstrap authority.

Canonical archive pointer:
1. `archive/EXCEL_FORMULA_LANGUAGE_EMPIRICAL_BASELINES.md`

## 5. Operator Precedence Baseline (Worksheet Formula Context)
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

## 6. Helper-Form Coverage Baseline (Draft)
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
| ISOMITTED | `=LAMBDA(a,ISOMITTED(a))(3)` | authoritative_behavioral + local_exercised | W040-HO-LOCAL | local_floor | Present lambda arguments are visible as non-omitted; direct under-application remains a distinct arity-mismatch lane in the current OxFml floor. |

Coverage note:
1. Public formal grammar anchors are incomplete for some modern helper-form details.
2. Helper-form completeness therefore depends on mixed formal + behavioral + empirical evidence.

### 6.1 Compile-Time Reducibility Boundary (Planning Note)
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

### 6.2 OxFunc Boundary Relevance
Several formula-language rules have direct OxFunc-boundary consequences:
1. `FML-R-003`, `FML-R-004`, and `FML-R-005` affect how `@`, `#`, and spill-linked results survive into prepared evaluation structures,
2. `FML-R-008` and `FML-R-009` affect bind outputs and reference identity,
3. `FML-R-012` affects argument preparation and admission-vs-runtime-error classification.
4. `FML-R-018` affects whether ordinary arithmetic operators preserve array structure honestly before result publication and replay capture.

These cross-lane effects must remain explicit during future tightening passes.

### 6.3 Function-Call Admission vs Runtime Error Boundary (Planning Note)
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

### 6.4 Name/Call Shadowing Oracle Boundary (Planning Note)
`FML-R-019` is intentionally not frozen yet.

Before OxFml promotes a generic host namespace rule for OxCalc W051, the `W074` oracle matrix must identify Excel behavior for:
1. built-in function names in call position and non-call bare-name position,
2. registered UDF names in call position and non-call bare-name position,
3. workbook-defined name and sheet-defined name collisions with built-ins,
4. workbook-defined name and sheet-defined name collisions with registered UDFs,
5. defined-name `LAMBDA` invocation by bare call and behavior when referenced in non-call position,
6. value-like, reference-like, and lambda-valued defined names with the same identifier across workbook and sheet scopes,
7. lexical `LET` / `LAMBDA` bindings colliding with built-ins, UDFs, and defined names,
8. late UDF registration, UDF unregister, capability-denial, defined-name mutation, and host namespace mutation as cache-invalidation triggers.

Matrix rows must keep these dimensions explicit:
1. source position: `call_callee`, `non_call_bare_name`, `let_lambda_lexical`, or `explicit_host_reference`,
2. visible candidates: built-in function, registered UDF, workbook-defined name, sheet-defined name, defined-name `LAMBDA`, lexical local, and host namespace name,
3. observed outcome: callable, ordinary value, reference-like value, worksheet error, admission rejection, or unresolved diagnostic,
4. invalidation inputs: registry snapshot, structure context, defined-name scope/kind, host namespace version, caller context, table context, and resolution-rule version,
5. replay-visible resolution layer and diagnostic class.

Current host mapping rule:
1. TreeCalc host names and lambda-valued nodes map to the closest Excel defined-name lane only as a planning default,
2. explicit host-reference syntax can intentionally select host objects that collide with function names,
3. any TreeCalc-specific divergence must be documented as an extension with replay-visible diagnostics and invalidation effects.
4. `LET` / `LAMBDA` lexical variables, callable locals, captures, and returned lambdas remain OxFml-internal and must not be projected as host namespace entries.

Structured-reference interaction:
1. `W074` host-name work does not replace the structured-reference lane,
2. table syntax continues to bind through `table_catalog`, `enclosing_table_ref`, and `caller_table_region`,
3. table-name-versus-defined-name disambiguation is an OxFml bind result over host-owned table context,
4. table-context changes are prepared-identity/cache invalidation inputs where they can change structured-reference resolution.

## 7. Open Items For Next Tightening Pass
1. Replicate scoped-name and precedence lanes across target channels/builds to verify current provisional policy wording.
2. Expand external/workbook reference lane to cover additional link-update policy variants and workbook-open/closed permutations across builds/channels (same-build baseline captured in `EMP-0011`).
3. Establish a true linked-data fixture path for `P2-FML-002` so dot-field semantics can be split by linked vs non-linked contexts.
4. Expand `P2-FML-008` spill-blocking/update scenarios to support `FML-R-005` promotion from `draft`.
5. Replicate argument-gap and normalization lanes across additional target builds/channels for status promotion to validated.
6. Execute `P2-FML-011` function-admission/coercion edge matrix (`SIN`/`ASIN` seeds), then split stable sub-rules from remaining provisional rows.
7. Execute `P2-FML-012` operator array-lift matrix, then confirm ordinary arithmetic elementwise semantics across targeted channels/builds.
8. Execute the `W074-CALC005` name/call shadowing oracle matrix before freezing generic host namespace precedence for W051.

## 8. Conformance Matrix And Archive Evidence
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
7. `W036` and `W038` for the remaining `MS-OE376`-reviewed carrier families not yet at the same local floor as `FML-R-014` and `FML-R-016`.
8. `W074-CALC005` built-in/UDF/defined-name/defined-name-LAMBDA shadowing and invalidation oracle matrix (`FML-R-019`).
