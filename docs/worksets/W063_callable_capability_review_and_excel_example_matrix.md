# W063: Callable Capability Review And Excel Example Matrix

## 1. Summary

This workset owns a bounded review of advanced Excel callable behavior around `LET`, `LAMBDA`, higher-order helpers, recursion, returned lambdas / currying, optional arguments, and array-oriented callable examples.

Its purpose is not to assume support from general Excel commentary, but to:
1. pin what Excel actually does,
2. pin what OxFml actually does,
3. author a concrete example matrix,
4. split supported, partial, and unsupported callable families honestly,
5. open implementation follow-on only where the behavior is both desired and bounded.

## 2. Why This Exists

Recent callable discussion surfaced a gap between:
1. broad public descriptions of Excel `LAMBDA` power,
2. the narrower current OxFml callable floor,
3. the specific advanced callable families we have actually exercised locally.

The risk is accidental drift in both directions:
1. assuming OxFml supports advanced callable behavior just because Excel does,
2. under-tracking advanced callable lanes that should become explicit work rather than folklore.

There is also a second classification risk:
1. a complex formula may fail inside a lambda even when callable formation, capture, and invocation are correct,
2. the actual defect may live in an underlying helper/function lane such as `TAKE`, `DROP`, `HSTACK`, `INDEX`, or array-shaping semantics,
3. this workset must therefore distinguish callable-layer gaps from downstream helper/function-semantic gaps rather than classifying every lambda-shaped failure as a callable bug.

## 3. Scope

In scope:
1. review current OxFml callable support against documented Excel behavior,
2. assemble a callable example matrix with exact formula rows,
3. classify each row as:
   - supported locally,
   - partial,
   - unsupported,
   - unpinned,
4. gather authoritative or official-looking Excel source notes where available,
5. add empirical-review placeholders where the docs are insufficient,
6. identify which families should become bounded implementation worksets,
7. add deterministic local evidence for any already-supported examples that are currently undocumented.

Primary callable families to review:
1. direct `LAMBDA` formation and invocation,
2. helper-bound invocation through `LET`,
3. lexical capture and closure behavior,
4. returned lambdas / currying,
5. recursion via self-reference / named lambdas,
6. higher-order helpers:
   - `MAP`
   - `REDUCE`
   - `SCAN`
   - `BYROW`
   - `BYCOL`
   - `MAKEARRAY`
7. optional parameters and omitted-argument semantics,
8. array/matrix callable examples using `HSTACK`, `VSTACK`, `TAKE`, `DROP`, `CHOOSECOLS`, and similar helpers,
9. named reusable workbook callables.

Required review distinctions:
1. callable-layer support versus underlying helper/function support,
2. locally exercised adopted defined-name callable lanes versus the broader workbook/name-manager story,
3. documented syntax/behavior versus empirical-only syntax/behavior,
4. supported callable omission versus plain under-application failure.

Out of scope unless explicitly widened later:
1. implementation of every discovered missing callable feature,
2. full cross-build/channel parity beyond the local review lane,
3. final callable seam redesign by itself.

## 4. Deliverables

1. A review note inside this workset describing:
   - current OxFml exercised callable floor,
   - Excel-documented callable behavior,
   - gaps between them.
2. A concrete example matrix with:
   - formula,
   - expected Excel behavior or status,
   - current OxFml status,
   - lane classification:
     - callable semantics,
     - helper/function semantics,
     - mixed,
   - evidence or source note,
   - follow-on owner.
3. Deterministic OxFml local examples for supported callable families that are currently under-documented.
4. New bounded worksets or bug streams where missing behavior is substantial enough to deserve its own owner.

## 4.1 Current Review Baseline

The currently exercised OxFml callable floor now includes:
1. direct `LAMBDA` value formation,
2. immediate lambda invocation,
3. helper-bound invocation through `LET`,
4. lexical capture rather than dynamic helper lookup,
5. capture exclusion under parameter shadowing,
6. higher-order helper execution through:
   - `MAP`
   - `REDUCE`
   - `SCAN`
   - `BYROW`
   - `BYCOL`
   - `MAKEARRAY`
7. adopted defined-name callable transport and invocation,
8. present-argument `ISOMITTED`,
9. explicit omitted-placeholder `ISOMITTED`,
10. bounded optional-parameter invocation through the local `W062` slice.

The currently pinned non-support / partial lanes now include:
1. returned-lambda invocation such as `adder(5)` producing a lambda that is then called through a helper binding,
2. recursion via self-reference,
3. broad workbook/name-manager callable parity beyond the adopted defined-name lane,
4. empirical Excel syntax truth for bracketed optional parameter declaration.

Current additional local finding:
1. a bounded defined-name recursion probe is not safe to keep in the normal test floor yet, because the current lane falls into unguarded recursion and overflows the process stack rather than producing a typed workbook-visible outcome.

## 5. Initial Question Set

1. Do we already support returned-lambda invocation end to end?
2. Do we support any bounded recursion lane, especially through adopted defined-name callables?
3. What exact optional-argument behaviors are Excel-documented versus empirical-only?
4. Which array-oriented lambda examples fail because of callable semantics versus underlying function/helper gaps?
5. Which public descriptions of Excel callable power should be treated as marketing/general guidance rather than implementation-grade syntax/semantics evidence?
6. Where does current defined-name callable support stop short of the broader workbook/name-manager callable story?
7. Which failures that appear "lambda-related" are actually helper-function defects exposed through lambda orchestration?

## 6. Example Families To Author

At minimum, author or review examples for:
1. `=LAMBDA(x,x+1)(2)`
2. `=LET(f,LAMBDA(x,x+1),f(2))`
3. `=LET(x,10,f,LAMBDA(y,x+y),LET(x,20,f(2)))`
4. `=LET(adder,LAMBDA(n,LAMBDA(x,x+n)),add5,adder(5),add5(10))`
5. recursion candidate such as factorial or countdown through a named callable
6. `=MAP(SEQUENCE(3),LAMBDA(x,x^2))`
7. `=REDUCE(0,SEQUENCE(3),LAMBDA(a,b,a+b))`
8. `=SCAN(0,SEQUENCE(3),LAMBDA(a,b,a+b))`
9. `=MAKEARRAY(2,3,LAMBDA(r,c,r*c))`
10. optional-argument candidate using `ISOMITTED`
11. array/matrix callable candidate using stacked or reshaped arrays
12. a complex helper-lambda row whose failure can be classified cleanly as callable-layer or helper-layer

## 6.1 Current Example Matrix

| Formula | Excel / Source Status | Current OxFml Status | Lane Classification | Evidence / Note | Follow-on Owner |
|---|---|---|---|---|---|
| `=LAMBDA(x,x+1)(2)` | ordinary lambda invocation expected | supported locally | callable semantics | `evaluator_runs_immediate_lambda_invocation` | none |
| `=LET(f,LAMBDA(x,x+1),f(2))` | ordinary helper-bound invocation expected | supported locally | callable semantics | `evaluator_runs_helper_bound_lambda_invocation` | none |
| `=LET(x,10,f,LAMBDA(y,x+y),LET(x,20,f(2)))` | lexical closure behavior expected | supported locally | callable semantics | `evaluator_uses_lexical_not_dynamic_scope_for_helper_bound_lambda` | none |
| `=MAP(SEQUENCE(3),LAMBDA(x,x^2))` | documented helper lambda lane | supported locally | callable semantics | `evaluator_executes_map_with_local_lambda_callable` | none |
| `=REDUCE(0,SEQUENCE(3),LAMBDA(a,b,a+b))` | documented helper lambda lane | supported locally | callable semantics | `evaluator_executes_reduce_with_local_lambda_callable` | none |
| `=SCAN(0,SEQUENCE(3),LAMBDA(a,b,a+b))` | documented helper lambda lane | supported locally | callable semantics | `evaluator_executes_scan_with_local_lambda_callable` | none |
| `=MAKEARRAY(2,3,LAMBDA(r,c,r+c))` | documented helper lambda lane | supported locally | callable semantics | `evaluator_executes_makearray_with_local_lambda_callable` | none |
| `=LAMBDA(a,b,ISOMITTED(b))(1,)` | omitted invocation documented by Microsoft | supported locally | callable semantics | `evaluator_preserves_explicit_omitted_placeholder_for_plain_lambda_params` | none |
| `=LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y))(5)` | declaration syntax empirically unpinned; omission behavior desired | supported locally, parity unpinned | callable semantics | `evaluator_executes_direct_lambda_with_optional_bracket_parameter` | `W062` |
| `=MAP(SEQUENCE(2),LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y)))` | declaration syntax empirically unpinned; helper arity rules documented | supported locally, parity unpinned | callable semantics | `evaluator_executes_map_with_optional_lambda_parameter_omitted_by_helper` | `W062` |
| `=LET(adder,LAMBDA(n,LAMBDA(x,x+n)),add5,adder(5),add5(10))` | public Excel commentary suggests this should work; no local Excel pin yet | rejected locally | callable semantics | `evaluator_currently_rejects_returned_lambda_invocation` | new follow-on if promoted |
| complex helper-lambda row using `TAKE(...,,...)` | helper-function semantics dependent | callable plumbing fixed earlier; formula still exposed helper gap until OxFunc lane landed | helper/function semantics | prior `HANDOFF-OXFUNC-004` note | downstream helper owner |
| recursion candidate via self-reference / named lambda | public Excel commentary suggests support; local Excel pin still absent | local probe currently unsafe: unguarded recursion overflows stack rather than producing a typed outcome | callable semantics | local attempted factorial-style probe showed stack-overflow behavior and was removed from the normal floor | new bounded bug/workset if promoted |
| workbook named callable through Name Manager semantics | broader than current adopted defined-name lane | partial | mixed | current local floor proves adopted defined-name callable only | potential new review follow-on |

## 7. Acceptance Gates

Gate 1: Review Baseline
1. current OxFml callable floor is summarized with references to existing evidence.

Gate 2: Example Matrix
1. a concrete callable example matrix exists,
2. each row is classified honestly,
3. each row distinguishes callable-layer versus helper-layer ownership where relevant.

Gate 3: Source Pinning
1. documented Excel callable guidance is linked where available,
2. empirical-only lanes are marked as empirical-only.

Gate 4: Follow-On Ownership
1. substantial missing callable families are assigned to explicit worksets or bug streams,
2. helper/function-semantic failures exposed through lambda examples are handed to the correct owner rather than kept as vague callable debt,
3. no large unsupported family remains only as a conversational note.

## 7.1 Current Gate Read

Gate 1: Review Baseline
1. satisfied locally.

Gate 2: Example Matrix
1. initial matrix now exists,
2. callable-layer versus helper-layer distinctions are explicit,
3. matrix still needs more Excel-pinned rows for recursion and workbook-level named callables.

Gate 3: Source Pinning
1. optional omission is source-backed,
2. bracketed parameter declaration remains empirical-only,
3. returned-lambda and recursion examples remain only partially source-backed and need Excel-specific confirmation if we want parity claims.

Gate 4: Follow-On Ownership
1. `W062` owns optional omission implementation,
2. returned-lambda invocation is now explicitly tracked as a callable-layer unsupported row rather than a vague conversational note,
3. recursion and workbook-level named-callable parity still need a bounded owner if promoted beyond review,
4. recursion specifically now has evidence of unsafe current behavior, not just absence of review.

## 8. Status

- `execution_state`: `in_progress`
- `scope_completeness`: `scope_partial`
- `target_completeness`: `target_partial`
- `integration_completeness`: `partial`
- `open_lanes`:
  - pin recursion behavior against Excel with a concrete exercised row
  - decide whether recursion should open a dedicated bug/workset now that the current local probe indicates stack-overflow behavior rather than a typed failure
  - decide whether returned-lambda invocation should become a bounded implementation lane or remain an explicit non-support note
  - refine the workbook/name-manager callable parity boundary beyond the currently adopted defined-name callable lane
