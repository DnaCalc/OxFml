# W062: Optional Lambda Parameters And Omitted Argument Support

## 1. Summary

This workset owns OxFml support for Excel-style optional `LAMBDA` parameters and omitted-argument invocation semantics, including bracketed parameter syntax such as `LAMBDA(x,[y],...)` and callable omission-sensitive lanes such as `ISOMITTED(y)`.

Current local behavior at workset open was narrower:
1. plain identifier lambda parameters were supported,
2. present-argument `ISOMITTED` behavior was exercised,
3. direct under-application remained an arity-mismatch failure,
4. explicit omitted-placeholder callable invocation remained unsupported.

This workset exists to move that topic from "outside the current floor" into a bounded implementation lane.

## 2. Why This Exists

The current callable floor intentionally stops before optional parameter syntax and omitted-placeholder preservation.

Current documented limitation at workset open:
1. `LAMBDA(a,b,ISOMITTED(b))(1,)` remained outside the current worksheet-helper floor.

That limitation is already reflected in:
1. `docs/spec/formula-language/OXFML_OXFUNC_LET_LAMBDA_PIN_DOWN_PREP.md`
2. `docs/spec/formula-language/EXCEL_FORMULA_LANGUAGE_CONFORMANCE_MATRIX.csv` row `FML-R-017`

User-facing example now requiring bounded support:
1. `=LET(f,LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y)),f(5,3))`

## 2.1 Source Notes

Current Microsoft-facing documentation relevant to this lane:
1. `LAMBDA function`
   - `https://support.microsoft.com/en-gb/office/lambda-function-bd212d27-1cd1-4321-a34a-ccbf254b8b67`
2. `ISOMITTED function`
   - `https://support.microsoft.com/en-us/office/isomitted-function-831d6fbc-0f07-40c4-9c5b-9c73fd1d60c1`
3. higher-order callable helpers with explicit lambda arity requirements:
   - `MAP`
   - `SCAN`
   - `MAKEARRAY`

Current documented facts from those sources:
1. `ISOMITTED` is an official callable-omission lane.
2. Microsoft documents omitted invocation syntax such as `...(1,)` in the `ISOMITTED` examples.
3. Microsoft documents helper functions as requiring exact lambda parameter counts for their callable slots.
4. Microsoft documents `LAMBDA([parameter1, parameter2, ...], calculation)` using square brackets in the prose signature.

Important caution:
1. the `LAMBDA([parameter1, parameter2, ...], calculation)` signature may be documentation notation for optional/repeated arguments rather than proof that literal bracketed parameter names like `[y]` are accepted worksheet syntax,
2. omitted invocation syntax is documented more clearly than bracketed optional-parameter declaration syntax,
3. this workset therefore treats bracketed parameter declaration as an implementation target that still needs empirical confirmation before parity claims are widened.

## 3. Scope

In scope:
1. parser and binder admission for bracketed optional lambda parameters,
2. bound callable shape updates needed to distinguish required vs optional parameters,
3. invocation semantics for omitted trailing callable arguments,
4. omitted-placeholder preservation into local callable evaluation,
5. `ISOMITTED` behavior over explicitly omitted optional lambda parameters,
6. direct invocation evidence and higher-order callable evidence where optional parameters are admitted,
7. spec and matrix updates once replay-backed local evidence exists.

Out of scope for this workset unless explicitly widened later:
1. non-trailing optional parameter reorder policies,
2. speculative UI/editor affordances for optional-parameter completion text,
3. cross-build/channel parity claims beyond the local OxFml floor,
4. unrelated callable/lambda transport redesign.

## 4. Current Drift / Problem Statement

Current OxFml drift:
1. `bind_lambda_args(...)` only accepts plain identifier helper parameter names.
2. `evaluate_invocation(...)` currently enforces exact callable arity.
3. present-argument `ISOMITTED` is covered, but explicit omission is not.

That means OxFml can currently prove:
1. `=LAMBDA(a,ISOMITTED(a))(3) -> FALSE`
2. `=MAP(SEQUENCE(2),LAMBDA(a,ISOMITTED(a))) -> {FALSE;FALSE}`

But it does not currently support:
1. `LAMBDA(x,[y],...)`
2. `f(5,)`
3. `ISOMITTED(y)` over an explicitly omitted optional callable parameter

## 4.1 Current Local Landed Slice

Current local implementation in this worktree now supports a bounded optional-callable slice:
1. bracketed lambda parameters such as `LAMBDA(x,[y],...)` bind as optional helper parameters,
2. callable metadata now preserves:
   - total arity,
   - required arity,
   - optional parameter names,
3. direct lambda invocation may omit trailing optional parameters,
4. helper-bound invocation through `LET` may omit trailing optional parameters,
5. omitted placeholders flow as `MissingArg`,
6. `ISOMITTED` now observes those placeholders through both direct and helper-bound invocation,
7. higher-order helper invocation currently carries the same omission contract through `MAP`.

Exercised local evaluator rows:
1. `=LAMBDA(a,b,ISOMITTED(b))(1,) -> TRUE`
2. `=LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y))(5) -> 10`
3. `=LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y))(5,3) -> 8`
4. `=LET(f,LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y)),f(5)) -> 10`
5. `=LET(f,LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y)),f(5,3)) -> 8`
6. `=MAP(SEQUENCE(2),LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y))) -> {2;4}`

Still intentionally true after this slice:
1. plain under-application of a required lambda parameter still fails as an arity mismatch,
2. this workset does not yet widen any Excel-parity claim for bracketed declaration syntax itself,
3. this workset does not claim replay-backed callable transport widening beyond the local evaluator and existing callable snapshot surfaces.

## 5. Desired End State

OxFml should support a bounded optional-callable lane where:
1. bracketed optional lambda parameters parse and bind explicitly,
2. direct invocation may omit trailing optional parameters without becoming an arity-mismatch failure,
3. omitted parameters are preserved as omitted placeholders, not silently converted to blank/value defaults,
4. `ISOMITTED` can observe those placeholders,
5. helper-bound and higher-order callable lanes treat that omission contract consistently.

What this source set suggests is required beyond mere bracket-parameter admission:
1. support for omitted trailing invocation arguments such as `(5,)`,
2. preservation of omitted placeholders into callable execution,
3. `ISOMITTED` visibility over those placeholders,
4. correct arity/error behavior for higher-order helper functions that receive lambdas,
5. clear separation between:
   - documentation notation for optional parameters,
   - actual worksheet syntax that is empirically confirmed.

## 6. Planned Execution Sequence

1. Define the admitted syntax and bind shape for optional parameters.
2. Extend callable metadata to distinguish required and optional parameters.
3. Extend invocation preparation so omitted trailing arguments survive as explicit omitted placeholders.
4. Update direct helper-call evaluation and higher-order callable invocation paths.
5. Add deterministic evaluator evidence for:
   - direct optional-parameter invocation,
   - omitted-placeholder `ISOMITTED`,
   - present-value override of an optional parameter,
   - helper-bound `LET` usage,
   - higher-order callable behavior if admitted in this round.
6. Update callable seam/spec text and `FML-R-017` if the evidence floor is truly widened.

## 7. Acceptance Gates

Gate 1: Syntax / Binding
1. `LAMBDA(x,[y],...)` is admitted and bound without degrading into malformed parameter diagnostics.
2. if empirical review shows bracketed declaration is only documentation notation, revise this gate to the actually admitted worksheet syntax rather than forcing a false syntax claim.

Gate 2: Direct Invocation
1. `LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y))(5)` succeeds.
2. `LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y))(5,3)` succeeds.

Gate 3: Helper-Bound Invocation
1. `LET(f,LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y)),f(5))` succeeds.
2. `LET(f,LAMBDA(x,[y],IF(ISOMITTED(y),x*2,x+y)),f(5,3))` succeeds.

Gate 4: Evidence / Doctrine
1. deterministic evaluator evidence exists,
2. callable-spec docs are updated,
3. `FML-R-017` is widened only if the new admitted slice is actually exercised.

## 7.1 Current Gate Read

Gate 1: Syntax / Binding
1. locally satisfied for the bounded bracketed-parameter slice.

Gate 2: Direct Invocation
1. locally satisfied.

Gate 3: Helper-Bound Invocation
1. locally satisfied.

Gate 4: Evidence / Doctrine
1. deterministic evaluator evidence exists locally,
2. callable transport and replay snapshot surfaces were updated for the widened callable profile metadata,
3. formula-language spec and conformance-matrix promotion remain open until the bracketed declaration syntax is empirically pinned against Excel rather than only implemented locally.

## 8. Risks And Adjacent Families

Adjacent families to inspect during implementation:
1. higher-order functions carrying optional lambdas (`MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, `MAKEARRAY`),
2. parameter shadowing and capture metadata,
3. omitted vs blank vs empty-text distinctions,
4. under-application vs optional-parameter omission boundaries,
5. callable transport snapshots and replay artifacts.

## 9. Status

- `execution_state`: `in_progress`
- `scope_completeness`: `scope_partial`
- `target_completeness`: `target_partial`
- `integration_completeness`: `partial`
- `open_lanes`:
  - empirical confirmation of actual Excel worksheet syntax for optional parameter declaration, especially whether literal bracketed parameter names are accepted formula text or documentation notation only
  - formula-language spec and `FML-R-017` promotion after that syntax question is pinned
  - broader higher-order coverage beyond the currently exercised `MAP` lane
