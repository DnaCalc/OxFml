# BUG-FML-011: TAKE Omitted Row Count In Helper Lambda Path

## Summary
- **Bug id**: `BUG-FML-011`
- **Opened**: 2026-04-10
- **Status**: handed_off
- **Owner workset**: `none yet`

## Source Refs
- **Reported against ref**: `7e2a5c0687aeec9d6635c5b3934394f531209e14`
- **Reproduced on ref**: `7e2a5c0687aeec9d6635c5b3934394f531209e14`
- **Introduced in ref**: `unknown`
- **Fixed in ref**: `not yet fixed`

## Ownership And Root Cause
- **Ownership class**: OxFunc-owned bug
- **Root cause class**: initial_impl_gap
- **Root cause summary**: after the OxFml helper-invocation environment fix, the remaining failure reduces to `TAKE(w,,...)` / `TAKE(Z,,...)` returning `Error(Value)`. Current OxFunc `eval_take_prepared(...)` always parses `args[1]` as an integer count, so an omitted second argument does not currently behave as the expected “all rows” lane.

## Reproduction
1. Evaluate:
   `=LET(cMul,LAMBDA(w,Z,LET(u,TAKE(w,,COLUMNS(w)/2),v,TAKE(w,,-COLUMNS(w)/2),X,TAKE(Z,,COLUMNS(Z)/2),Y,TAKE(Z,,-COLUMNS(Z)/2),HSTACK(u*X-v*Y,u*Y+v*X))),a,HSTACK(3,4),b,HSTACK(1,2),result,cMul(a,b),INDEX(result,1,1)+INDEX(result,1,2)*100)`
2. Current local OxFml trace reaches:
   - `SPECIAL.LAMBDA`
   - `FUNC.HSTACK`
   - `FUNC.HSTACK`
   - `SPECIAL.LAMBDA_INVOKE`
   - `FUNC.COLUMNS`
   - `FUNC.OP_DIVIDE`
   - `FUNC.TAKE`
   - ...
3. `cMul(a,b)` reduces to `Array(1x2)` with `[Error(Value), Error(Value)]`.

## Similar-Risk Scan
### Adjacent families to check
1. `TAKE(array,,n)` outside lambda
2. `DROP(array,,n)` and similar reshape functions with omitted leading count
3. omitted-count semantics in direct worksheet calls versus helper-lambda-carried calls

### Results
1. no final family closure yet
2. local OxFml helper invocation is no longer the blocker

## Linked Reports
1. `BUGREP-FML-008`

## Evidence
1. `crates/oxfml_core/src/eval/mod.rs`
2. `../OxFunc/crates/oxfunc_core/src/functions/dynamic_array_reshape_family.rs`
3. `docs/handoffs/HANDOFF-OXFUNC-004_TAKE_OMITTED_ROW_COUNT_IN_COMPLEX_LAMBDA.md`
