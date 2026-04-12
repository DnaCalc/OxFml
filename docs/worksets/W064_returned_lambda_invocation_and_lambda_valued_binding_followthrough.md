# W064: Returned Lambda Invocation And Lambda-Valued Binding Follow-Through

## 1. Summary

This workset owns the missing callable lane where a `LAMBDA` returns another `LAMBDA`, and the returned callable is then invoked through a helper binding or equivalent local callable path.

Representative target examples:
1. `=LET(adder,LAMBDA(n,LAMBDA(x,x+n)),add5,adder(5),add5(10))`
2. direct or nested invocation patterns that require OxFml to treat a lambda-valued expression result as an invokable callable rather than rejecting it because it is not an immediate helper-bound or defined-name call.

## 2. Purpose

The current OxFml callable floor supports:
1. direct `LAMBDA` invocation,
2. helper-bound invocation through `LET`,
3. higher-order callable transport into helper functions such as `MAP`.

It does not currently support the next callable step:
1. a callable expression result being rebound and invoked later as a first-class returned lambda value.

This workset exists to:
1. pin the intended OxFml behavior for returned-lambda invocation,
2. add deterministic local evidence for the supported slice,
3. keep callable ownership in the right place rather than bypassing the invocation boundary with ad hoc local shortcuts.

Current local findings:
1. direct nested returned-lambda invocation is now admitted locally,
2. helper-bound returned-lambda rebinding required two bounded fixes:
   - invocation must accept lambda-valued callee expressions, not only immediate helper-bound or defined-name callables,
   - helper-local names that resemble A1 references, such as `add5`, must bind as helper-local names in callable scope instead of being misclassified as cell references.

## 3. Position And Dependencies

- **Depends on**:
  - `W062_optional_lambda_parameters_and_omitted_argument_support.md`
  - `W063_callable_capability_review_and_excel_example_matrix.md`
- **Blocks**: none
- **Cross-repo**: none currently expected unless a shared callable carrier or OxFunc callable bridge change becomes necessary

## 4. Scope

In scope:
1. helper-bound invocation of lambda-valued expression results,
2. invocation of returned lambdas produced by `LET` / direct lambda bodies,
3. closure/capture preservation across returned-lambda invocation,
4. deterministic evaluator and prepared-call evidence for the admitted slice,
5. explicit classification of any cases that remain outside the admitted local callable floor.

Out of scope:
1. general recursion support,
2. workbook-wide Name Manager callable parity beyond the adopted local lane,
3. optional-argument semantics except where already admitted by `W062`.

## 5. Deliverables

1. A bounded returned-lambda invocation implementation if the lane can be admitted locally.
2. Deterministic local tests covering:
   - simple returned lambda invocation,
   - lexical capture preservation through the returned lambda,
   - helper-bound rebinding of a returned lambda before invocation.
3. Honest documentation of any remaining excluded callable sub-lanes.

Current local evidence rows:
1. `=LAMBDA(n,LAMBDA(x,x+n))(5)(10)`
2. `=LET(adder,LAMBDA(n,LAMBDA(x,x+n)),add5,adder(5),add5(10))`
3. `=LET(base,100,adder,LAMBDA(n,LAMBDA(x,x+n+base)),add5,adder(5),add5(10))`
4. retained prepared-call evidence:
   - `prepared_019_nested_returned_lambda_invoke`
   - `prepared_020_helper_bound_returned_lambda_value`
5. coordinator-facing adapter evidence:
   - `adapter_preserves_internal_lambda_but_publishes_calc_for_helper_bound_returned_lambda`
   - `adapter_executes_helper_bound_returned_lambda_invocation`

## 6. Gate Model

Gate 1: Behavior Pin
1. the exact admitted returned-lambda invocation slice is stated,
2. any excluded callable forms are stated explicitly.

Gate 2: Local Evidence
1. at least one simple returned-lambda example passes locally,
2. at least one capture-preserving returned-lambda example passes locally.

Gate 3: Boundary Integrity
1. the change does not bypass callable ownership with ad hoc evaluator shortcuts,
2. any seam widening is documented before being claimed.

## 7. Status

- `execution_state`: `in_progress`
- `scope_completeness`: `scope_partial`
- `target_completeness`: `target_partial`
- `integration_completeness`: `partial`
- `open_lanes`:
  - decide whether broader workbook/name-manager callable rebinding belongs here or remains outside the admitted local lane
- `claim_confidence`: `provisional`
