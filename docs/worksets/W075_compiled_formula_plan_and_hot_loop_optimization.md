# W075: Compiled Formula Plan And Hot-Loop Optimization

## Purpose

Process OxFunc `HO-FN-016`, DNA OneCalc performance pressure, and the first Mandelbrot timing observations into an OxFml-owned execution-frame workset.

The long-term target is not a set of one-off fast paths. The target is a generic compiled formula execution substrate:

1. OxFml owns formula structure, lexical scope, LET/LAMBDA binding, reference binding, evaluation order, trace policy, and graph planning.
2. OxFunc owns every function and operator semantic through resolved `SurfaceCallSite` invocation.
3. OxFml must not hard-code special behavior for functions such as `INDEX`, `HSTACK`, arithmetic operators, helper functions, or future catalog entries.
4. Repeated evaluation should execute through a compiled `EvaluationFrame` with lexical slots, reusable call buffers, reusable call-site scratch, and trace-mode-aware emission.

## Position and Dependencies

- **Depends on**: `W065`, `W068`, `W070`, OxFunc `W095`, OxFunc `W096`
- **Responds to**: `../OxFunc/docs/handoffs/HO-FN-016_compiled_surface_call_site_and_index_dispatch.md`
- **Acknowledgement**: `docs/handoffs/HO-FN-016_COMPILED_SURFACE_CALL_SITE_AND_INDEX_DISPATCH_ACK.md`
- **Blocks**: downstream large-model and lambda-heavy performance replay once deterministic metrics are available
- **Cross-repo**: OxFunc owns `SurfaceCallSite`, `SurfaceCallRuntime`, `SurfaceCallScratch`, callable batching, function metadata, dispatch-table behavior, and function/operator semantics; OxFml owns compiled formula-plan consumption, lexical execution, trace-mode behavior, and formula graph planning.

## Baseline Observations

Exploratory release-mode timing on the `100 x 60 x 30` Mandelbrot formula showed:

1. formula compile/bind/semantic-plan construction is not the bottleneck,
2. repeated `stacker::maybe_grow(...)` probes around every helper lambda invocation were a major fixed cost,
3. coalescing stack guards around helper batches reduced trivial helper loops from seconds to milliseconds and reduced the full Mandelbrot formula from roughly ten seconds to roughly four seconds on the local machine,
4. the remaining cost is ordinary interpreted formula-body work: helper-local lookup, LET frame mutation, invocation setup, per-call argument buffers, and generic recursive expression dispatch.

These observations are not closure evidence. They define the first measurement targets for this workset.

## Architecture Target

### EvaluationFrame

Introduce a compiled execution frame that carries per-evaluation mutable state:

1. lexical slot storage for LET and LAMBDA locals,
2. reusable argument buffers for compiled call nodes,
3. reusable `SurfaceCallScratch` where a call node is invoked repeatedly,
4. reusable `SurfaceCallRuntime` provider wiring where ownership and borrow rules allow it,
5. trace-mode state and prepared-call emitters,
6. callable recursion budget state and coalesced stack-guard depth,
7. optional node-local scratch addresses allocated by the compiled plan.

The frame is an execution substrate, not a semantic registry. It must not encode function-specific Excel behavior.

### Compiled Plan Nodes

Lower OxFml-owned language structure into plan nodes:

1. literals and array literals,
2. lexical slot load/store,
3. reference materialization and reference-preserving argument preparation,
4. OxFml-owned special forms: `LET`, `LAMBDA`, lambda invocation, `IF` laziness, `IFERROR` laziness where applicable,
5. ordinary `SurfaceCallSite` calls for functions and operators,
6. helper-family callable slots and callable invocation,
7. trace emitter nodes or trace templates for prepared-call mode.

Every function/operator call remains a call through OxFunc `SurfaceCallSite`; OxFml plan nodes only describe formula-language control flow and data movement.

### Trace Policy

`EvaluationTraceMode::ValueOnly` remains the default. Value-only hot loops must not allocate prepared-call scaffolding. Prepared-call trace mode must preserve current trace evidence and semantic equivalence, even if it uses templates or emitters rather than ad hoc allocation.

### Hoisting Policy

Hoisting is intentionally delayed until the frame and node model are stable. When added, hoisting must use `SurfaceCallSite` metadata plus explicit runtime-context policy. Pure/invariant facts must not be inferred from OxFml-maintained function-name tables.

## Scope

### In Scope

1. Persist deterministic performance fixtures for helper loops and a Mandelbrot-shaped workload.
2. Coalesce callable stack guarding so helper-family batches do not pay `stacker::maybe_grow(...)` per iteration while recursive lambda chains remain guarded.
3. Introduce `EvaluationFrame` as the single substrate for lexical slots, reusable scratch, reusable argument buffers, and trace-mode state.
4. Replace repeated lambda/LET local name lookup with compiled lexical slots.
5. Lower OxFml-owned control-flow constructs to generic compiled nodes.
6. Reuse call argument buffers and `SurfaceCallScratch` for repeated compiled call-site invocation.
7. Preserve OxFunc ownership by invoking functions/operators through `SurfaceCallSite`.
8. Keep trace-off and trace-on behavior explicit and tested.
9. Use OxFunc metadata for later purity/hoisting gates.
10. File narrow OxFunc follow-ups for missing metadata rather than adding a mirror registry in OxFml.

### Out of Scope

1. Moving `INDEX`, arithmetic, `HSTACK`, helper semantics, or any other function-specific behavior into OxFml.
2. Whole-workbook scheduler implementation beyond compiled formula-plan prerequisites.
3. JIT/native-code generation.
4. Changing Excel-visible semantics for performance.
5. Pack-grade performance claims without deterministic evidence and baseline comparison.
6. Function-specific optimization tables in OxFml.

## Deliverables

1. Ignored/manual or benchmark-gated perf fixture suite for:
   - `MAKEARRAY(..., LAMBDA(..., 1))`,
   - scalar `REDUCE`,
   - stateful `REDUCE` with `INDEX`/`HSTACK`,
   - the full `100 x 60 x 30` Mandelbrot formula.
2. Stack-guard coalescing with recursion non-regression evidence.
3. `EvaluationFrame` carrying lexical slots, scratch, argument buffers, trace mode, and stack-guard state.
4. Slot-frame execution for lambda parameters and LET locals.
5. Generic compiled nodes for OxFml-owned formula-language control flow.
6. `SurfaceCallSite` call nodes with reusable argument/scratch handling.
7. Trace-on and trace-off parity tests for optimized paths.
8. Metadata-driven hoisting design and first safe hoist/no-hoist evidence.
9. Workset closure packet with before/after timing table and remaining open lanes if any.

## Gate Model

### Entry Gate

- OxFunc `HO-FN-016` surface is locally consumable from OxFml.
- Current OxFml evaluation suite passes against the resolved call-site surface.
- Initial `SurfaceCallSite` and `SurfaceCallScratch` consumption exists in OxFml.
- Stack-guard fixed-cost issue has local exploratory evidence.

### Mid-Gate A: Measurement And Safety

- Perf fixture exists but is not ordinary CI noise.
- Stack-guard coalescing has deterministic semantic tests.
- Recursive lambda boundary tests still pass.
- Release-mode timing can be reproduced locally with documented command and formula set.

### Mid-Gate B: Frame Substrate

- `EvaluationFrame` exists and owns slot/scratch/trace mutable state.
- Existing evaluator paths can still run through the frame without semantic changes.
- No duplicate long-lived fast-path substrate is introduced.

### Mid-Gate C: Slot And Node Execution

- Lambda parameters and LET locals are slot-addressed.
- OxFml-owned control-flow nodes execute through the frame.
- Function/operator calls still route through `SurfaceCallSite`.
- Trace-on and trace-off tests pass.

### Exit Gate

- No hot path calls OxFunc by surface string after binding when call identity is known.
- Lambda helper hot loops use slot-frame execution for local parameters and LET-bound values.
- Reusable argument buffers and scratch are applied where repeated call-site invocation occurs and do not change argument preparation or call ordering.
- Value-only hot loops avoid prepared-call allocation.
- Prepared-call trace mode remains equivalent for optimized paths.
- Deterministic performance evidence exists with before/after figures.
- Any missing OxFunc metadata has a filed follow-up handoff.

## Bead Set

### B075-01: Deterministic Performance Fixture

- **Status**: planned
- **Owner**: OxFml
- **Purpose**: create a repeatable local perf fixture suite before further optimizer work proceeds.
- **Files likely touched**:
  - `crates/oxfml_core/tests/perf_*.rs` or a benchmark/manual test module,
  - `docs/worksets/W075_compiled_formula_plan_and_hot_loop_optimization.md`.
- **Formula set**:
  - `=MAKEARRAY(100,60,LAMBDA(r,c,1))`,
  - `=REDUCE(0,SEQUENCE(6000),LAMBDA(a,b,a+b))`,
  - `=REDUCE(HSTACK(0,0,0),SEQUENCE(6000),LAMBDA(state,k,HSTACK(INDEX(state,1,1),INDEX(state,1,2),INDEX(state,1,3))))`,
  - the full Mandelbrot formula from DNA OneCalc timing discussion.
- **Execution details**:
  - run release-mode timings,
  - compile once where measuring evaluation,
  - keep warmup separate from measured runs,
  - report min/avg/max and formula result summary,
  - mark fixture ignored/manual unless a dedicated benchmark harness exists.
- **Evidence target**:
  - timing table recorded in this workset or sidecar evidence doc,
  - ordinary test suite unaffected by timing noise.
- **Risks**:
  - wall-clock timings are machine-sensitive; use them for directional evidence, not pack-grade claims.

### B075-02: Callable Stack-Guard Coalescing

- **Status**: in_progress
- **Owner**: OxFml
- **Purpose**: avoid paying `stacker::maybe_grow(...)` on every helper-loop iteration while preserving recursive lambda safety.
- **Files likely touched**:
  - `crates/oxfml_core/src/eval/mod.rs`,
  - focused evaluator tests if a new regression is needed.
- **Execution details**:
  - keep callable recursion budget enforcement unchanged,
  - enter a callable stack guard once around `CallableInvoker::invoke_many(...)` batches,
  - reuse an active guard for nested non-recursive helper work,
  - periodically re-probe stack for deep recursive chains,
  - keep direct lambda invocation and returned-lambda invocation guarded.
- **Evidence target**:
  - `cargo test -p oxfml_core --test evaluator_tests -- --nocapture`,
  - `cargo test -p oxfml_core`,
  - release-mode timing for `MAKEARRAY` and Mandelbrot controls.
- **Risks**:
  - removing stack probes entirely causes recursive lambda stack faults; periodic re-probe is mandatory.

### B075-03: EvaluationFrame Substrate

- **Status**: planned
- **Owner**: OxFml
- **Purpose**: create the single mutable execution substrate that later beads build on.
- **Files likely touched**:
  - `crates/oxfml_core/src/eval/mod.rs`,
  - possible new `crates/oxfml_core/src/eval/frame.rs` if split is warranted.
- **Frame responsibilities**:
  - lexical slot storage,
  - helper/local binding compatibility bridge during migration,
  - node-local reusable argument buffers,
  - node-local `SurfaceCallScratch`,
  - trace-mode state,
  - callable recursion and stack guard state,
  - reference resolver access.
- **Execution details**:
  - introduce frame without changing semantics first,
  - route existing evaluation functions through the frame where practical,
  - avoid making a second evaluator beside the existing one.
- **Evidence target**:
  - no public API churn unless necessary,
  - full `oxfml_core` tests pass,
  - no performance regression in B075-01 fixtures.
- **Risks**:
  - partial frame adoption can create duplicate state; migration must keep one owner for mutable execution data.

### B075-04: Lexical Slot Frame Compiler

- **Status**: planned
- **Owner**: OxFml
- **Purpose**: replace helper-local `BTreeMap<String, HelperBinding>` lookup in hot paths with compiled slot indexes.
- **Files likely touched**:
  - evaluator compiled-plan code,
  - LET/LAMBDA helper-binding code,
  - callable invocation code,
  - tests for shadowing, optional parameters, recursion, captures.
- **Execution details**:
  - assign slot indexes during compiled-plan construction,
  - preserve case-insensitive Excel helper-name semantics at bind/compile time,
  - represent lexical capture as slot/frame capture data,
  - support lambda parameters, optional parameters, LET locals, and returned lambdas,
  - maintain bridge behavior for defined-name callable bindings until migrated.
- **Evidence target**:
  - existing lambda/LET/capture/optional-parameter tests pass,
  - new test proves parameter and LET reads avoid name-map fallback in the compiled path,
  - timing improvement on scalar `REDUCE` and `MAKEARRAY`.
- **Risks**:
  - shadowing and capture semantics are subtle; preserve current tests before broadening.

### B075-05: Compiled OxFml Language Nodes

- **Status**: planned
- **Owner**: OxFml
- **Purpose**: lower OxFml-owned formula-language constructs into direct node execution.
- **Files likely touched**:
  - compiled expression plan types,
  - `LET`, `LAMBDA`, invocation, `IF`, `IFERROR` evaluation paths,
  - trace preparation helpers.
- **Node families**:
  - `LoadSlot`,
  - `StoreSlot`,
  - `LetBlock`,
  - `LambdaLiteral`,
  - `InvokeLambda`,
  - `IfLazy`,
  - `IfErrorLazy`,
  - `ReferenceArg`,
  - `SurfaceCall`.
- **Execution details**:
  - keep OxFml-owned laziness in OxFml nodes,
  - keep OxFunc-owned function/operator semantics behind `SurfaceCallSite`,
  - preserve prepared-call trace evidence through node emitters/templates.
- **Evidence target**:
  - lazy `IF` / `IFERROR` tests,
  - direct and returned lambda tests,
  - helper-family callable tests,
  - trace-on parity tests.
- **Risks**:
  - lowering `IF` and `IFERROR` must preserve current Excel-visible laziness boundaries.

### B075-06: SurfaceCall Argument And Scratch Reuse

- **Status**: in_progress
- **Owner**: OxFml with OxFunc seam consumption
- **Purpose**: broaden current call-site/scratch reuse from built-in callable batching to general repeated compiled call nodes.
- **Files likely touched**:
  - compiled call-site node execution,
  - `SurfaceCallScratch` integration,
  - call argument preparation helpers.
- **Execution details**:
  - allocate argument buffers by compiled node id or frame scratch index,
  - clear and refill buffers without reallocating,
  - use `SurfaceCallScratch` where OxFunc exposes a scratch path,
  - do not cache semantic results,
  - preserve argument evaluation order.
- **Evidence target**:
  - operator, `INDEX`, `HSTACK`, reference-visible, host/query, and registered-external tests pass,
  - timing improvement on `REDUCE` state and Mandelbrot controls.
- **Risks**:
  - references and callable carrier encoding must retain current ownership and lifetime behavior.

### B075-07: Trace Template And Value-Only Discipline

- **Status**: planned
- **Owner**: OxFml
- **Purpose**: make trace behavior part of compiled execution rather than ad hoc allocation.
- **Files likely touched**:
  - trace structs,
  - prepared-call helpers,
  - evaluation frame,
  - replay fixture tests.
- **Execution details**:
  - value-only mode emits no prepared-call records,
  - trace mode can stamp from static node templates plus dynamic argument/result fields,
  - trace output remains equivalent for existing replay fixtures,
  - trace policy stays in `EvaluationContext` / frame, not OxFunc.
- **Evidence target**:
  - `evaluation_context_defaults_to_value_only_trace_mode`,
  - prepared-call replay fixtures,
  - runtime and adapter trace tests.
- **Risks**:
  - replay fixture churn must represent real contract behavior, not incidental ordering changes.

### B075-08: Metadata-Driven Hoisting

- **Status**: planned
- **Owner**: OxFml with OxFunc coordination
- **Purpose**: use `SurfaceCallSite` metadata and runtime-context policy to hoist safe invariant subexpressions after the frame and node model exist.
- **Files likely touched**:
  - compiled plan construction,
  - frame initialization,
  - tests for volatile/host/reference/provider boundaries.
- **Execution details**:
  - use OxFunc metadata for determinism, volatility, host interaction, locale, reference, and external-provider sensitivity,
  - hoist only under explicit context policy,
  - keep `NOW`, `RAND`, locale-sensitive, host-query, RTD, registered-external, and reference-sensitive lanes guarded unless metadata and context permit,
  - file OxFunc follow-up if metadata is insufficient.
- **Evidence target**:
  - hoist/no-hoist tests for pure arithmetic, locale-sensitive `TEXT`, volatile time/random, host query, RTD, registered external, and reference-visible calls.
- **Risks**:
  - premature hoisting can change Excel-visible volatility or host/provider behavior.

### B075-09: Integration, Evidence, And Closure Packet

- **Status**: planned
- **Owner**: OxFml
- **Purpose**: consolidate W075 evidence and prevent premature closure.
- **Files likely touched**:
  - this workset,
  - `docs/IN_PROGRESS_FEATURE_WORKLIST.md`,
  - optional handoff docs if OxFunc metadata gaps are discovered.
- **Execution details**:
  - update the performance timing table,
  - run full verification,
  - document residual open lanes,
  - file narrow OxFunc handoffs only for concrete metadata/seam gaps,
  - do not claim pack-grade evidence from local wall-clock timings.
- **Evidence target**:
  - `cargo fmt -p oxfml_core`,
  - `cargo check -p oxfml_core`,
  - `cargo test -p oxfml_core`,
  - `git diff --check`,
  - release-mode perf fixture output.
- **Risks**:
  - workset closure language must follow the Anti-Premature-Completion Doctrine.

## Execution Order

1. B075-01: persist timing fixture.
2. B075-02: land stack-guard coalescing and recursive safety evidence.
3. B075-03: introduce `EvaluationFrame`.
4. B075-04: slot-frame compiler and execution.
5. B075-05: compiled OxFml language nodes.
6. B075-06: reusable call argument/scratch buffers.
7. B075-07: trace templates.
8. B075-08: metadata-driven hoisting.
9. B075-09: integration and evidence packet.

The ordering is intentional: hoisting comes after the execution frame and node model, so optimization does not outrun semantic structure.

## Current Local Measurement Notes

Exploratory timings from the first stack-guard investigation on this machine:

| Formula family | Pre-coalescing observation | Coalesced guard observation |
|---|---:|---:|
| `MAKEARRAY(100,60,LAMBDA(r,c,1))` | seconds | single-digit milliseconds |
| scalar `REDUCE` over 6000 items | seconds | single-digit milliseconds |
| full `100 x 60 x 30` Mandelbrot text formula | roughly 10 seconds | roughly 3.5-4.0 seconds |

These numbers are directional and must be replaced by B075-01 fixture output before any closure claim.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | no |
| 2 | Conformance matrix rows updated? | no |
| 3 | At least one deterministic replay or timing artifact exists per in-scope behavior? | no |
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
  - persisted deterministic perf fixture,
  - stack-guard coalescing closure evidence,
  - `EvaluationFrame` substrate,
  - lexical slot-frame execution,
  - compiled OxFml language nodes,
  - reusable call argument and scratch buffers,
  - reusable trace templates,
  - metadata-driven hoisting,
  - OxFunc metadata follow-ups if discovered.
- claim_confidence: draft
