# W075: Compiled Formula Plan And Hot-Loop Optimization

## Purpose

Process OxFunc `HO-FN-016` and the DNA OneCalc performance observations into an OxFml-owned optimization workset for generic compiled formula plans, lambda hot-loop execution, and later whole-model recalculation planning.

The intent is to reduce repeated interpreter overhead while preserving ownership boundaries:

1. OxFml owns formula structure, lexical scope, LET/LAMBDA binding, reference binding, evaluation order, trace policy, and graph planning.
2. OxFunc owns every function and operator semantic through resolved `SurfaceCallSite` invocation.
3. OxFml must not hard-code special behavior for functions such as `INDEX`, `HSTACK`, arithmetic operators, or helper functions.

## Position and Dependencies

- **Depends on**: `W065`, `W068`, `W070`, OxFunc `W095`, OxFunc `W096`
- **Responds to**: `../OxFunc/docs/handoffs/HO-FN-016_compiled_surface_call_site_and_index_dispatch.md`
- **Blocks**: downstream large-model and lambda-heavy performance replay once deterministic metrics are available
- **Cross-repo**: OxFunc owns `SurfaceCallSite`, `SurfaceCallRuntime`, `SurfaceCallScratch`, callable batching, function metadata, and dispatch-table behavior; OxFml owns compiled formula-plan consumption and trace-mode behavior.

## Scope

### In Scope

1. Expand the current compiled evaluation plan so ordinary function/operator calls retain resolved `SurfaceCallSite` handles and can reuse `SurfaceCallScratch` where repeated invocation makes reuse meaningful.
2. Introduce a generic lexical-slot execution model for LET/LAMBDA frames so lambda hot loops do not repeatedly resolve local names through map lookup.
3. Preserve all function/operator semantics inside OxFunc by dispatching through resolved call sites.
4. Use `CallableInvoker::invoke_many(...)` for helper-family batch lanes and hoist batch-stable setup outside per-row/per-step execution.
5. Define generic expression-plan nodes for constants, lexical slots, references, call sites, special forms, and invocations without function-specific AST rewrites.
6. Use `SurfaceCallSite` metadata for purity and hoistability gates under explicit runtime-context policy.
7. Keep `EvaluationTraceMode::ValueOnly` as the default and define reusable trace-template behavior for prepared-call trace mode rather than allocating full trace scaffolding in value-only hot loops.
8. Add deterministic microbenchmarks or replay-style perf fixtures for lambda-heavy formulas, including `REDUCE`/`SCAN`/`MAP` and a Mandelbrot-shaped workload if available.
9. Document any missing OxFunc metadata as narrow follow-up handoffs rather than adding a mirror function registry in OxFml.

### Out of Scope

1. Moving `INDEX`, arithmetic, `HSTACK`, helper semantics, or any other function-specific behavior into OxFml.
2. Whole-workbook scheduler implementation beyond the compiled formula-plan prerequisites.
3. JIT/native-code generation.
4. Changing Excel-visible semantics for performance.
5. Pack-grade performance claims without deterministic evidence and baseline comparison.

## Deliverables

1. Compiled plan consumes `SurfaceCallSite`, `SurfaceCallRuntime`, and meaningful `SurfaceCallScratch` reuse in repeated call paths.
2. Lambda helper invocation has a slot-frame execution path that avoids repeated local-name map lookup for lambda parameters and LET locals.
3. Value-only mode avoids prepared-call trace allocation on hot paths; trace mode remains opt-in and semantically equivalent.
4. Constant/hoist planning uses OxFunc metadata and explicit runtime-context policy.
5. Performance fixtures record baseline and optimized timings for at least one lambda-heavy helper family.
6. Tests prove semantic parity for ordinary functions, operators, reference-visible functions, helper callables, trace-on mode, and trace-off mode.

## Gate Model

### Entry Gate

- OxFunc `HO-FN-016` surface is locally consumable from OxFml.
- Current OxFml evaluation suite passes against the resolved call-site surface.
- Initial scratch reuse is present in at least one repeated invocation path or explicitly documented as blocked.

### Exit Gate

- No runtime hot path calls OxFunc by surface string after binding when the call identity is known.
- Lambda helper hot loops use slot-frame execution for local parameters and LET-bound values.
- Scratch reuse is applied where repeated call-site invocation occurs and does not change call ordering or argument preparation.
- Trace-off and trace-on tests both pass for optimized paths.
- Deterministic performance evidence exists with before/after figures and semantic parity tests.
- Any missing OxFunc metadata has a filed follow-up handoff.

## Bead Set

### B075-01: Call-Site Scratch Reuse

- **Status**: in_progress
- **Owner**: OxFml
- **Effect**: use `SurfaceCallScratch` in repeated invocation paths where one call site is invoked many times.
- **Evidence target**: helper-family built-in callable batching and non-regression tests.

### B075-02: Lexical Slot Frame Plan

- **Status**: planned
- **Owner**: OxFml
- **Effect**: replace repeated lambda/LET local name lookup with compiled slot indexes.
- **Evidence target**: lambda helper tests plus recursive/optional-argument non-regressions.

### B075-03: Generic Compiled Expression Nodes

- **Status**: planned
- **Owner**: OxFml
- **Effect**: define reusable node families for literals, slots, references, function call sites, special forms, and invocations.
- **Evidence target**: no function-specific optimizer tables in OxFml.

### B075-04: Trace Template Mode

- **Status**: planned
- **Owner**: OxFml
- **Effect**: keep value-only hot loops free of prepared-call allocation while allowing prepared-call trace mode to stamp reusable templates.
- **Evidence target**: trace-off empty trace tests and trace-on prepared-call parity tests.

### B075-05: Metadata-Driven Hoisting

- **Status**: planned
- **Owner**: OxFml with OxFunc coordination
- **Effect**: use `SurfaceCallSite` metadata and runtime-context policy for constant and pure-subtree hoisting.
- **Evidence target**: deterministic hoist/no-hoist tests for pure, locale, time, random, host, reference, and external-provider lanes.

### B075-06: Performance Evidence

- **Status**: planned
- **Owner**: OxFml with downstream replay consumers
- **Effect**: produce repeatable perf fixtures for lambda-helper and large-formula workloads.
- **Evidence target**: benchmark or replay artifact with baseline, optimized result, and semantic parity.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | no |
| 2 | Conformance matrix rows updated? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Cross-repo impact assessed and handoff filed if needed? | no |
| 5 | All required tests pass? | no |
| 6 | No known semantic gaps remain in declared scope? | no |
| 7 | Completion language audit passed? | no |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | no |

## Status

- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - lexical slot-frame execution,
  - reusable trace templates,
  - metadata-driven hoisting,
  - deterministic performance fixtures,
  - OxFunc metadata follow-ups if discovered.
- claim_confidence: draft
