# Excel Formula Language Pass-2 Probe Plan

## 1. Purpose
Define the deferred empirical pass-2 execution plan for formula-language unresolved lanes.

This plan is intended to:
1. close known ambiguities in `FML-R-*`,
2. provide clear status-upgrade criteria (`draft` -> `provisional` -> `validated`),
3. be executable later without relying on agent session memory.

## 2. Inputs and Dependencies
Primary references:
1. `EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md`
2. `EXCEL_FORMULA_LANGUAGE_CONFORMANCE_MATRIX.csv`
3. `EXCEL_FORMULA_LANGUAGE_PASS2_SCENARIO_SEED.csv`
4. `EXCEL_CELL_CONCRETE_MODEL_OPEN_QUESTIONS.md`
5. Prior wave artifacts under:
   - `research/runs/20260228-180047-excel-compat-empirical-pass-01/outputs/formula_parse_wave1/`

Execution prerequisites:
1. Excel runner available and version-pinned with executable hash capture.
2. Scenario fixtures and manifests versioned in run outputs.
3. Output records include requirement and rule IDs (`XLS-CF-FL-*`, `FML-R-*`).

## 3. Scenario Set

| probe_id | objective | target_rules | target_open_questions | scenario_scope | expected_output_artifacts |
|---|---|---|---|---|---|
| P2-FML-001 | Resolve argument-gap behavior around double commas and missing arguments in function calls. | FML-R-010 | ECM-Q-010 | `SUM`, `AVERAGE`, `IF`, `LET`, nested calls; gaps in first/middle/last arg positions | acceptance table, stored formula capture, result-class matrix by build |
| P2-FML-002 | Bound dot-field parse/eval semantics across linked-data and non-linked-data contexts. | FML-R-011 | ECM-Q-009 | `A1.Field` variants with linked data type, plain text/number/error, missing/invalid field names | parse/eval matrix, error-code matrix, context flags |
| P2-FML-003 | Expand intersection whitespace disambiguation and parser edge handling. | FML-R-001 | ECM-Q-001 | intersections with names, ranges, structured refs, extra whitespace forms | accept/reject matrix and normalized formula captures |
| P2-FML-004 | Build helper-form grammar acceptance/rejection matrix with formal-vs-behavioral tags. | FML-R-006 | ECM-Q-011 | `LET`, `LAMBDA`, `MAP`, `BYROW`, `BYCOL`, `SCAN`, `REDUCE`, malformed arity/parentheses cases | per-construct matrix and source-class annotations |
| P2-FML-005 | Validate workbook/sheet name resolution and shadowing precedence. | FML-R-008 | ECM-Q-001 | collisions between workbook names, sheet names, local names, structured references | name-resolution outcome matrix with bound target references |
| P2-FML-006 | Expand external/workbook reference grammar acceptance and failure behavior. | FML-R-006 | ECM-Q-001 | external workbook refs, missing workbook, quoted sheet names, malformed separators | parse acceptance + resulting error behavior matrix |
| P2-FML-007 | Expand structured-reference grammar malformed/edge corpus. | FML-R-009 | ECM-Q-001 | nested qualifiers, malformed bracket nests, escaped identifiers, this-row variations | accept/reject matrix, stored formula normalization outcomes |
| P2-FML-008 | Expand `@`/`#` interaction corpus with spill/non-spill contexts and blocked spill lanes. | FML-R-003;FML-R-004;FML-R-005 | ECM-Q-001 | combinations of `@`, `#`, dynamic arrays, blocked spill targets, table contexts | behavior matrix including parse, evaluation, and spill result class |
| P2-FML-009 | Broaden normalization corpus beyond current two rows. | FML-R-007 | ECM-Q-001 | function case, name case, structured refs, helper forms, external refs, whitespace variants | input-vs-stored formula normalization table |
| P2-FML-010 | Validate precedence checksum corpus for full operator tiers. | FML-R-001;FML-R-002 | ECM-Q-001 | formulas combining reference ops, arithmetic, `%`, exponentiation, concat, comparison | expected-result checksum table and precedence decision traces |
| P2-FML-011 | Resolve function-call admission vs runtime error boundaries for canonical non-interesting trig seeds. | FML-R-012 | ECM-Q-012 | `SIN`/`ASIN` required-arg omission, scalar coercion failure, mixed-type array-lift behavior, numeric-domain errors | parse-admission matrix + runtime error/result-shape matrix by build/channel |

Execution seed artifact:
1. `EXCEL_FORMULA_LANGUAGE_PASS2_SCENARIO_SEED.csv` contains initial scenario rows (`FMLP2-001`..`FMLP2-041`).
2. Seed rows marked `probe` require outcome capture before status decisions.

## 4. Status Upgrade Rules
1. `draft` -> `provisional`:
   - probe rows for target rule family are executed with reproducible artifacts,
   - no unexplained mismatches remain for required baseline rows.
2. `provisional` -> `validated`:
   - ambiguity/conflict lane is either resolved by convergent evidence or explicitly version-scoped with accepted compatibility policy,
   - requirement and trace bindings are updated to reflect final policy wording.

## 5. Output Contract (Pass 2)
For each `probe_id`:
1. Scenario manifest (`.csv` or `.jsonl`) with stable scenario IDs.
2. Seed-to-executed mapping report against `EXCEL_FORMULA_LANGUAGE_PASS2_SCENARIO_SEED.csv`.
3. Per-scenario evidence bundles:
   - run manifest,
   - raw capture,
   - normalized capture,
   - stdout/stderr,
   - step capture.
4. Aggregated result matrix with `expected`, `observed`, `result_class`.
5. Promotion notes for `FML-R-*` status updates and affected `ECM-Q-*` closures.

## 6. Execution Status (2026-03-02)
Execution status update:
1. Plan executed in run pack:
   - `research/runs/20260302-070309-excel-formula-language-pass2-pack-01/outputs/formula_parse_pass2/`
2. Scenario coverage:
   - seeded scenarios `FMLP2-001..FMLP2-037` executed (`37/37` evidence bundles).
3. Summary outcome:
   - observed accepted: `35`
   - observed rejected: `2`
   - mismatch rows: `0`
   - run-failed rows: `0`

Primary execution artifacts:
1. `FORMULA_PARSE_PASS2_RESULTS.csv`
2. `SEED_TO_EXECUTED_MAPPING_PASS2.csv`
3. `PASS2_EXECUTION_REPORT.md`
4. `MANUAL_PREP_PASS2B_REPORT.md`
5. `TARGETED_PASS2C_LANES_REPORT.md`

Remaining unresolved lanes after execution:
1. `P2-FML-002` linked-data semantic branch (true linked-data fixture still required; conversion attempts currently captured as allowed-error in this environment).
2. `P2-FML-005` policy wording has been drafted from pass-2c evidence; cross-build/channel replay is still required.
3. `P2-FML-006` now has workbook-present/open-state baseline evidence (`EMP-0011`); remaining work is link-update/open-state policy expansion and cross-build replay.
4. `P2-FML-008` spill-blocking/update lane still needs expansion before `FML-R-005` can leave `draft`.
5. `P2-FML-011` has been seeded but not executed; this lane is required to resolve parse-admission vs runtime-error boundaries for non-interesting functions.

## 7. Post Pass-2 Parallel Lanes (2026-03-02)
Pass-2 execution is complete. Follow-up is split into two independent lanes:
1. Pass-4 policy/trace sync (documentation tightening without new execution).
2. Pass-5 replay-pack preparation (cross-build/channel execution planning).

Run artifacts:
1. `research/runs/20260302-070309-excel-formula-language-pass2-pack-01/outputs/formula_parse_pass2/PASS4_POLICY_TRACE_SYNC.md`
2. `research/runs/20260302-070309-excel-formula-language-pass2-pack-01/outputs/formula_parse_pass2/PASS5_REPLAY_PACK.md`
3. `research/runs/20260302-070309-excel-formula-language-pass2-pack-01/outputs/formula_parse_pass2/PASS5_REPLAY_MANIFEST.csv`
