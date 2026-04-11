# HANDOFF-OXFUNC-004: TAKE Omitted Row Count In Complex Helper Lambda

## 1. Direction
- **From**: `OxFml`
- **To**: `OxFunc`
- **Filed date**: `2026-04-10`
- **Source workset**: `local direct investigation`

## 2. Purpose
Report the remaining failure in a complex helper-bound lambda formula after the
local OxFml invocation-environment bug was fixed.

## 3. Repro Formula
`=LET(cMul,LAMBDA(w,Z,LET(u,TAKE(w,,COLUMNS(w)/2),v,TAKE(w,,-COLUMNS(w)/2),X,TAKE(Z,,COLUMNS(Z)/2),Y,TAKE(Z,,-COLUMNS(Z)/2),HSTACK(u*X-v*Y,u*Y+v*X))),a,HSTACK(3,4),b,HSTACK(1,2),result,cMul(a,b),INDEX(result,1,1)+INDEX(result,1,2)*100)`

Expected result:
- `995`

Current local outcome after the OxFml-side fix:
- no helper-binding failure
- `cMul(a,b)` reduces to `Array(1x2)` with both cells `Error(Value)`
- final formula becomes `Error(Value)`

## 4. Current OxFml Read
The OxFml-local helper invocation bug was:
- invocation arg expressions were being evaluated against the callee closure
  instead of the caller helper environment

That local issue is now fixed in `crates/oxfml_core/src/eval/mod.rs`.

After that fix, the remaining failure appears to be in the `TAKE` lane:
- `TAKE(w,,COLUMNS(w)/2)`
- `TAKE(w,,-COLUMNS(w)/2)`
- `TAKE(Z,,COLUMNS(Z)/2)`
- `TAKE(Z,,-COLUMNS(Z)/2)`

Current OxFunc implementation read:
- `eval_take_prepared(...)` in
  `../OxFunc/crates/oxfunc_core/src/functions/dynamic_array_reshape_family.rs`
  always parses `args[1]` as a required integer row count
- for this formula, the second argument is intentionally omitted
- OxFml expects the omitted row-count lane to mean “all rows” while the third
  argument supplies the column slice

## 5. Trace Evidence
Current local prepared-call trace for the lambda body reaches:
1. `FUNC.COLUMNS`
2. `FUNC.OP_DIVIDE`
3. `FUNC.TAKE`
4. `FUNC.COLUMNS`
5. `FUNC.OP_NEGATE`
6. `FUNC.OP_DIVIDE`
7. `FUNC.TAKE`
8. `FUNC.COLUMNS`
9. `FUNC.OP_DIVIDE`
10. `FUNC.TAKE`
11. `FUNC.COLUMNS`
12. `FUNC.OP_NEGATE`
13. `FUNC.OP_DIVIDE`
14. `FUNC.TAKE`
15. `FUNC.OP_MULTIPLY`
16. `FUNC.OP_MULTIPLY`
17. `FUNC.OP_SUBTRACT`
18. `FUNC.OP_MULTIPLY`
19. `FUNC.OP_MULTIPLY`
20. `FUNC.OP_ADD`
21. `FUNC.HSTACK`

So the remaining break is not helper invocation dispatch; it is inside the
reshape/value lane.

## 6. OxFunc Ask
1. pin the intended semantics for `TAKE(array,,n)` and `TAKE(array,,-n)`
2. if the intended Excel lane is “all rows, slice columns”, implement that in
   the prepared/surface path
3. check adjacent reshape families with omitted leading count arguments
4. reply with the exact landed ref once available
