# BUGREP-FML-008: Helper Lambda Invocation With TAKE Omitted Rows Exposes OxFunc Gap

## Intake
- **Report id**: `BUGREP-FML-008`
- **Filed**: 2026-04-10
- **Source channel**: local investigation
- **Reporter/source**: direct formula check
- **Reported against ref**: `7e2a5c0687aeec9d6635c5b3934394f531209e14`
- **Reported against kind**: commit
- **Canonical bug id**: `BUG-FML-011`
- **Status**: triaged

## Observed Symptom
The formula below still fails after the local OxFml helper-invocation fix:

`=LET(cMul,LAMBDA(w,Z,LET(u,TAKE(w,,COLUMNS(w)/2),v,TAKE(w,,-COLUMNS(w)/2),X,TAKE(Z,,COLUMNS(Z)/2),Y,TAKE(Z,,-COLUMNS(Z)/2),HSTACK(u*X-v*Y,u*Y+v*X))),a,HSTACK(3,4),b,HSTACK(1,2),result,cMul(a,b),INDEX(result,1,1)+INDEX(result,1,2)*100)`

Current local trace shows:
1. helper-bound invocation now works,
2. `cMul(a,b)` evaluates to `Array(1x2)` with both cells `Error(Value)`,
3. the remaining failure is inside `TAKE(w,,...)` and `TAKE(Z,,...)`.

## Initial Ownership Read
- **Initial classification**: OxFunc-owned bug
- **Reason**: current OxFunc `TAKE` parsing/coercion does not appear to accept an omitted row-count argument in the second position as “all rows”, even though OxFml now passes the lambda arguments correctly.

## Links
1. `docs/bugs/streams/BUG-FML-011_take_omitted_row_count_in_helper_lambda_path.md`
2. `docs/handoffs/HANDOFF-OXFUNC-004_TAKE_OMITTED_ROW_COUNT_IN_COMPLEX_LAMBDA.md`
