# W065: Recursive Callable Safety And Workbook-Visible Behavior

## 1. Summary

This workset owns the recursion callable lane for OxFml, with two separate goals:
1. prevent unsafe local recursion behavior such as process stack overflow,
2. pin and implement the workbook-visible behavior OxFml should surface when recursive callable formulas are exercised.

Representative target examples:
1. factorial or countdown through a named recursive lambda,
2. bounded self-reference patterns where Excel documentation suggests recursion is possible,
3. failure behavior when recursion exceeds Excel's callable boundary.

## 2. Purpose

`W063` isolated recursion as more than an unreviewed advanced feature:
1. Microsoft `LAMBDA` guidance says self-call recursion can return `#NUM!`,
2. the current local OxFml probe falls into unguarded recursion and overflows the process stack instead of producing a typed workbook-visible outcome.

This workset exists to turn that from a vague review note into a bounded owner for:
1. recursion behavior pinning,
2. safe local handling,
3. explicit workbook-visible failure semantics.

Current local finding after the current `W065` slice:
1. runaway recursive defined-name callables now surface worksheet-visible `#NUM!` locally instead of process stack overflow,
2. bounded recursive success is now admitted locally for the exercised defined-name lane when the stop condition uses the locally lazy `IF` path,
3. local callable execution now uses an explicit grown-stack lane plus an empirical recursion-budget model for the exercised callable families,
4. the explicit Excel-probed named-recursion and `LET` self-application boundaries are now matched locally for the exercised rows,
5. broader recursion parity is still not claimed beyond those exercised lanes.

## 3. Position And Dependencies

- **Depends on**:
  - `W063_callable_capability_review_and_excel_example_matrix.md`
- **Blocks**: none
- **Cross-repo**:
  - may require OxCalc / host review if recursion behavior depends on workbook-level named-callable admission beyond the adopted local lane

## 4. Scope

In scope:
1. bounded recursive callable examples through the currently admitted local callable lanes,
2. prevention of unsafe stack-overflow behavior in local evaluation,
3. classification of workbook-visible recursive failure behavior,
4. deterministic local evidence for any admitted recursion slice,
5. explicit non-support notes if some recursion families remain outside the admitted floor.

Out of scope:
1. full workbook Name Manager parity by itself,
2. every advanced recursive workbook pattern,
3. speculative recursion-cap optimization beyond what is needed for safe workbook-visible behavior.

## 5. Deliverables

1. A pinned local recursion behavior statement grounded in Microsoft guidance and local evidence.
2. A safe local outcome for the exercised recursion lane:
   - supported result, or
   - typed workbook-visible error, but not process stack overflow.
3. Deterministic tests for at least one bounded recursive example and one failure-boundary example if admitted.
4. If exact empirical boundaries are adopted locally, deterministic tests for those exact boundary rows.

Current retained local evidence:
1. runaway recursive defined-name callable:
   - `=Loop()`
   - current local outcome: `#NUM!`
   - proof row: `evaluator_projects_runaway_recursive_defined_name_callable_as_num_error`
2. bounded recursive defined-name callable:
   - `=Fact(5)`
   - current local outcome: `120`
   - proof row: `evaluator_executes_bounded_recursive_defined_name_callable`
3. local recursion stop-condition laziness:
   - `=IF(TRUE,1,1/0)` -> `1`
   - `=IFERROR(1,1/0)` -> `1`
   - proof rows:
     - `evaluator_preserves_if_branch_laziness_locally`
     - `evaluator_preserves_iferror_fallback_laziness_locally`
4. retained prepared-call evidence for the local lazy stop-condition lane:
   - `prepared_021_if_branch_laziness_local`
   - `prepared_022_iferror_fallback_laziness_local`
5. exact local recursion-boundary evidence:
   - workbook named recursion:
     - `=CountDown(5460)` -> `5460`
     - `=CountDown(5461)` -> `#NUM!`
   - `LET` self-application recursion:
     - `=LET(F,LAMBDA(self,n,IF(n<=0,0,1+self(self,n-1))),F(F,4094))` -> `4094`
     - `=LET(F,LAMBDA(self,n,IF(n<=0,0,1+self(self,n-1))),F(F,4095))` -> `#NUM!`
   - proof rows:
     - `evaluator_matches_excel_named_recursion_success_boundary`
     - `evaluator_matches_excel_named_recursion_failure_boundary`
     - `evaluator_matches_excel_let_self_application_recursion_success_boundary`
     - `evaluator_matches_excel_let_self_application_recursion_failure_boundary`

## 6.2 Explicit Excel Comparison Notes

Explicit Excel COM probes on 2026-04-11 established:
1. workbook named recursion is admitted well beyond the current OxFml local cap:
   - `CountDown(5000)` returned `5000`
   - `CountDown(5500)` returned `#NUM!`
2. `LET` self-application recursion is also admitted well beyond the current OxFml local cap, but fails earlier than workbook named recursion:
   - `=LET(F,LAMBDA(self,n,IF(n<=0,0,1+self(self,n-1))),F(F,4000))` returned `4000`
   - `=LET(F,LAMBDA(self,n,IF(n<=0,0,1+self(self,n-1))),F(F,4200))` returned `#NUM!`
3. direct helper-local self-recursion by name inside `LET` is not a success lane in Excel:
   - `=LET(F,LAMBDA(n,IF(n<=0,0,1+F(n-1))),F(5))` returned `#NAME?`

Current OxFml comparison:
1. local callable recursion no longer uses the earlier provisional depth cap,
2. OxFml now uses:
   - explicit stack growth in local callable execution,
   - an empirical recursion-budget model tuned to the currently exercised Excel lanes,
   - a red-zone reserve sized for the current OxFunc value representation so deep recursion reaches the guard instead of the process stack,
3. direct helper-local self-recursion now surfaces worksheet-visible `#NAME?` locally instead of a fatal execution failure,
4. the exercised named-recursion and `LET` self-application rows above now match the explicit Excel probes.

## 6. Gate Model

Gate 1: Source And Behavior Pin
1. Microsoft recursion guidance is cited,
2. the exercised local recursion lane is stated explicitly.

Gate 2: Safety
1. the current unsafe stack-overflow behavior is removed from the admitted local lane,
2. recursive callable failure becomes workbook-visible rather than process-fatal.

Gate 3: Evidence
1. deterministic local evidence exists for the admitted slice,
2. excluded recursion families remain explicitly documented.

## 6.1 Current Admitted Slice

The currently admitted local recursion slice is:
1. adopted defined-name callable recursion,
2. bounded success through an `IF` stop condition,
3. runaway recursion capped to worksheet-visible `#NUM!`,
4. exact exercised boundary matching for:
   - `CountDown(5460)` success / `CountDown(5461)` `#NUM!`,
   - `LET` self-application `4094` success / `4095` `#NUM!`,
5. direct helper-local self-recursion by name surfaces worksheet-visible `#NAME?`,
6. no claim yet for broader workbook Name Manager parity or every branch-lazy function family.

## 7. Status

- `execution_state`: `in_progress`
- `scope_completeness`: `scope_partial`
- `target_completeness`: `target_partial`
- `integration_completeness`: `partial`
- `open_lanes`:
  - refine recursion behavior beyond the currently exercised named-recursion and `LET` self-application rows if broader workbook Name Manager parity is promoted
  - decide whether the current empirical recursion-budget model should remain local implementation detail or be documented as an admitted OxFml callable boundary
  - decide whether broader branch-lazy function families beyond local `IF` / `IFERROR` need their own owner before more recursive examples are admitted
- `claim_confidence`: `provisional`
