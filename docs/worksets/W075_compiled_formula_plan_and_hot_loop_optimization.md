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

- **Status**: complete
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
- **Current execution note**:
  - `crates/oxfml_core/tests/w075_perf_fixture_tests.rs` now contains an ignored manual release-mode timing fixture for the four target formulas.
  - Command: `cargo test -p oxfml_core --test w075_perf_fixture_tests --release -- --ignored --nocapture`.
  - The fixture compiles each formula once, runs one warmup evaluation, measures three runs per case, reports min/avg/max, and prints a bounded result summary.

### B075-02: Callable Stack-Guard Coalescing

- **Status**: complete
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
- **Current execution note**:
  - Coalesced callable stack guarding is present in `crates/oxfml_core/src/eval/mod.rs`.
  - Focused evaluator recursion and lambda tests pass after the current W075 changes.

### B075-03: EvaluationFrame Substrate

- **Status**: complete
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
- **Current execution note**:
  - `EvaluationFrame` now owns the formula evaluation trace, callable registry, root helper-binding environment, reusable ordinary-call scratch stack, callable recursion budget state, and callable stack-guard depth for the ordinary evaluation entrypoint.
  - `EvaluationContext` receives a private pointer to the active frame state during evaluation so the existing evaluator helpers can consume frame-owned mutable state without introducing a second evaluator path.
  - This is still a migration substrate rather than a full separate frame VM. Node-indexed scratch addresses and reference-resolver ownership remain open only if profiling or later planning justifies them.

### B075-04: Lexical Slot Frame Compiler

- **Status**: complete
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
- **Current execution note**:
  - A compatibility fast path now checks exact helper-binding keys before falling back to the existing case-insensitive scan.
  - `HelperBindingFrame` now carries helper bindings as persistent case-folded layers with stored display names, preserving case-insensitive shadowing while preparing the evaluator for real slot storage.
  - `LET` now evaluates in a child helper layer instead of cloning the whole helper frame, and helper-batch lambda parameter keys are precomputed for direct slot updates.
  - This is not full slot-vector execution, but it removes cloned helper frames from LET-heavy helper loops and establishes the storage boundary needed for slot vectors.
  - A first slot-id plumbing experiment was exercised after this slice, but the attempted runtime slot storage regressed the Mandelbrot fixture and was not promoted as the B075-04 path.
  - Interim gate result at that point: keep this bead partial; the next attempt needed true frame-owned slot arrays or another non-cloning design, not layered slot overlays.
  - A follow-up frame-owned slot-array slice now assigns helper slot ids during compiled-plan construction and lowers helper-local references to direct slot reads where a slot is known.
  - Lambda invocation now gets a per-call slot frame copied from captured slot values, while child `LET` frames share that call frame rather than cloning slots.
  - Helper-batch argument assignment updates slotted lambda parameters through the slot frame and leaves the compatibility name map as a fallback/capture index.
  - Interim gate result at that point: semantics and controls were acceptable for a first slot-frame floor, but Mandelbrot remained in a noisy local band rather than showing a decisive improvement; B075-04 still needed tighter capture/slot-frame evidence.
  - A follow-up slot-only `LET` evaluator path now skips helper-map child layer creation and binding insertion for compiled `LET` blocks whose binding names are all slotted and whose subtree contains no `LAMBDA` literal requiring closure capture through helper-map entries.
  - The slot-only `LET` path stores and reads through the existing helper slot frame only; all function/operator calls inside the block continue through OxFunc `SurfaceCallSite`.
  - Gate result: correctness held and the Mandelbrot timing fixture improved materially, so this slice is retained as the current B075-04 floor. Later slices added the remaining current-scope capture and direct-invocation evidence needed for W075 closure.
  - Direct `LAMBDA` invocation now binds slotted parameters through `insert_helper_slot_binding(...)` rather than name-only helper entries, so captured slotted parameters stay available to nested helper lambdas through slot reads instead of map fallback.
  - Gate result: evaluator, shadowing, replay, and W075-specific tests held; Mandelbrot improved modestly while simpler controls stayed in the local noise band, so this is retained as a structural slot-frame correction.
  - Stored helper/callable lambda bodies now use shared compiled-expression ownership, reducing deep body clones when lambda bindings are copied through helper slots, callable carriers, and registries.
  - Stored lambda parameter arrays now also use shared ownership; the isolated retest held semantics and remained timing-neutral on the Mandelbrot fixture.
  - Gate result: evaluator, replay, and W075-specific tests held; Mandelbrot improved materially, so the shared-body representation is retained as a generic callable transport improvement.
  - Closure construction now allocates an empty slot frame and populates only the named captured slots instead of cloning the whole helper slot vector into every closure.
  - Gate result: evaluator, shadowing, replay, and W075-specific tests held; Mandelbrot improved, and capture scope is now narrower and closer to the lexical free-name set.
  - A follow-up experiment that additionally stored precomputed helper keys on `LambdaParam` was tried and rejected at the gate: semantics held, but the release timing fixture regressed and variance spiked, so that helper-key metadata experiment was backed out.

### B075-05: Compiled OxFml Language Nodes

- **Status**: complete
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
- **Current execution note**:
  - Compiled function-call sites now cache callable-argument ordinals at compile time for the call's arity.
  - This removes per-dispatch callable-ordinal vector construction without changing function/operator ownership or call ordering.
  - Numeric and string literals now parse/decode during compiled-plan construction rather than on every node evaluation.
  - `LET`, `LAMBDA`, `IF`, and `IFERROR` now lower into explicit compiled OxFml nodes instead of re-entering the ordinary function-call dispatch path.
  - Known ordinary function/operator calls with resolved OxFunc call sites now lower into an explicit `SurfaceCall` node, while `FunctionCall` remains the compatibility path for special forms and unresolved or fallback calls.
  - The lowered nodes still reuse the existing evaluator behavior and trace publication helpers. Wider invocation/reference lowering is future optimizer scope rather than a W075 closure requirement.
  - Compiled `LET` nodes now carry conservative slot-only eligibility so lambda-free slotted `LET` blocks can execute through slot storage without helper-map mutation.

### B075-06: SurfaceCall Argument And Scratch Reuse

- **Status**: complete
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
- **Current execution note**:
  - Ordinary compiled `SurfaceCall` evaluation now takes a reusable frame-state `SurfaceCallScratch` from `EvaluationContext` instead of allocating a fresh surface-call scratch for every call.
  - Compiled function-call sites now reuse precomputed callable argument ordinals instead of recomputing them per dispatch.
  - Local callable `invoke_many(...)` now decodes prepared arguments once while assigning helper slots and deriving lambda-argument recursion cost.
  - Built-in callable batching continues to use OxFunc `SurfaceCallScratch` for `invoke_many(...)`.
  - The ordinary `SurfaceCallScratch` pool now retains a stack of reusable scratch buffers instead of a single buffer, so nested repeated calls learn their scratch depth once and avoid repeated inner scratch allocation.
  - The current pool is frame-wide rather than node-indexed; node-local scratch indexes remain an open lane if profiling shows a benefit beyond the stack pool.

### B075-07: Trace Template And Value-Only Discipline

- **Status**: complete
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
- **Current execution note**:
  - Ordinary function calls no longer construct `PreparedCall` trace records in value-only mode.
  - Trace-on paths still build prepared-call records and the existing replay-focused tests pass.
  - `w075_plan_optimization_tests.rs` now includes direct evidence that value-only helper hot loops emit no prepared-call records, prepared-call mode still records the optimized slot-only `LET` path, slot-only `LET` preserves lexical shadowing, and narrowed lambda closures preserve named captures.
  - The current W075 trace model uses trace emitters/helpers rather than static templates; static template extraction is not required for the current exit gate unless future profiling identifies trace-on overhead as material.

### B075-08: Metadata-Driven Hoisting

- **Status**: complete
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
- **Current execution note**:
  - A first strict context-free precompute slice now wraps only literal/helper-free `Binary`, `Unary`, and ordinary `SurfaceCall` subtrees whose OxFunc `SurfaceCallSite` reports `is_context_free_pure()`.
  - Precomputed nodes retain their original source subtree and evaluate that source when `EvaluationTraceMode::PreparedCalls` is active, preserving replay-facing prepared-call evidence.
  - Runtime-context-sensitive calls remain dynamic; focused W075 tests cover prepared trace preservation for nested pure calls and `NOW()` with changing runtime seeds.
  - A captured-helper invariant cache experiment was tried for lambda bodies and rejected at the gate: focused semantics passed, but the W075 timing fixture regressed and variance increased, so it was backed out and not promoted.
  - Broader hoisting over captured helper constants, references under fixed context, locale/time/random policies, and host/external providers remains future optimizer scope outside W075.

### B075-09: Integration, Evidence, And Closure Packet

- **Status**: complete
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
- **Current execution note**:
  - Current promoted floor has been verified with `cargo fmt -p oxfml_core`, `cargo check -p oxfml_core`, `cargo test -p oxfml_core`, `git diff --check`, and the ignored release-mode W075 timing fixture.
  - `git diff --check` reports only LF-to-CRLF normalization warnings on touched files.
  - The latest current-floor slices also have focused verification from `cargo test -p oxfml_core --test w075_plan_optimization_tests -- --nocapture` with six W075 optimization/trace tests, `cargo test -p oxfml_core --test ftc_1013_case_insensitive_shadowing_tests -- --nocapture`, `cargo test -p oxfml_core --test evaluator_tests -- --nocapture`, `cargo test -p oxfml_core --test replay_fixture_tests`, and the ignored release-mode W075 timing fixture.
  - This evidence supports the W075 closure claim together with the verification checklist and completion self-audit below.

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

Exploratory timings from the first stack-guard investigation on this machine remain directional. B075-01 now provides a repeatable local fixture, but the numbers are still wall-clock local evidence rather than pack-grade claims.

Latest local command:

`cargo test -p oxfml_core --test w075_perf_fixture_tests --release -- --ignored --nocapture`

| Formula family | Latest local fixture observation |
|---|---:|
| `MAKEARRAY(100,60,LAMBDA(r,c,1))` | min 1.964 ms / avg 1.993 ms / max 2.012 ms |
| scalar `REDUCE` over 6000 items | min 4.832 ms / avg 4.837 ms / max 4.845 ms |
| stateful `REDUCE` with `INDEX`/`HSTACK` over 6000 items | min 12.346 ms / avg 12.462 ms / max 12.548 ms |
| full `100 x 60 x 30` Mandelbrot text formula | min 1427.692 ms / avg 1453.733 ms / max 1483.027 ms |

The latest numbers include stack-guard coalescing, frame-owned mutable execution state, pooled ordinary `SurfaceCallScratch` with a frame-level scratch stack for nested calls, compiled callable-argument ordinal reuse, single-pass prepared-argument decoding inside local callable batching, the persistent layered `HelperBindingFrame` boundary, the first frame-owned helper slot array, narrowed captured-slot closure construction, slotted direct `LAMBDA` invocation parameters, shared ownership for stored lambda bodies and parameter arrays, parsed numeric/string literal payloads, explicit compiled `LET` / `LAMBDA` / `IF` / `IFERROR` nodes, explicit ordinary `SurfaceCall` nodes for resolved OxFunc call sites, the value-only trace allocation fix for ordinary calls, the first strict context-free precompute slice, and conservative slot-only execution for lambda-free slotted `LET` blocks.

Gate assessment after the persistent `HelperBindingFrame` slice:

- progress_as_expected: yes for this compatibility slice; at this interim gate, full lexical slot vectors still needed later evidence
- implementation_doc_sync: yes
- path_forward_clear: yes - continue by replacing layered helper bindings with true lexical slot vectors for lambda parameters and LET locals
- decision: proceed only with a dedicated slot-vector rewrite as the next structural bead; avoid further map-layer micro-optimization unless profiling identifies a specific residual cost

Gate assessment after the first slot-storage experiment:

- progress_as_expected: no for B075-04; evaluator semantics held, but Mandelbrot timing regressed when helper-local slot reads used layered runtime slot overlays
- implementation_doc_sync: yes - the experiment is documented here as not promoted, and the next B075-04 slice needs a different storage shape
- path_forward_clear: yes for the next slice - direct slot execution still looks architecturally correct, but it needs frame-owned slot arrays/call frames rather than child-scope overlay structures
- decision: do not continue by tuning layered slot overlays; either design the real frame-owned slot array model or move to the next non-slot bead with this gate recorded

Gate assessment after the first frame-owned slot-array and explicit `LET` / `LAMBDA` node slice:

- progress_as_expected: yes for structural migration and helper-loop controls; at this interim gate, Mandelbrot-level performance evidence still needed more slices
- implementation_doc_sync: yes - B075-04 and B075-05 notes now distinguish the promoted slot-array/node floor from the non-promoted overlay experiment
- path_forward_clear: yes - continue by tightening capture slot snapshots and lowering the remaining OxFml-owned control-flow nodes, with no function-specific handling in OxFml
- decision: continue within W075, but do not claim B075-04 or B075-05 target closure from this local timing evidence

Gate assessment after adding explicit `IF` / `IFERROR` nodes:

- progress_as_expected: yes for the current B075-05 slice; evaluator semantics held and the local Mandelbrot fixture returned to the earlier best timing band
- implementation_doc_sync: yes - current B075-05 notes and timing table include the explicit lazy-control-flow nodes
- path_forward_clear: yes - continue later with invocation/reference node lowering and trace templates; do not add function-specific fast paths
- decision: retain this slice as the current W075 floor; later slices provide the current-scope closure evidence for B075-04 and B075-05

Gate assessment after ordinary `SurfaceCall` node, pooled scratch, and value-only trace allocation slices:

- progress_as_expected: yes for structural routing and correctness; perf remains in the same local Mandelbrot band, with stateful `REDUCE` slightly better and scalar `REDUCE` noisier on this run
- implementation_doc_sync: yes - B075-05, B075-06, B075-07, and the latest timing table now describe the promoted floor
- path_forward_clear: yes - continue with remaining W075 lanes through generic plan execution, trace-template work, and metadata-driven hoisting; keep all function/operator semantics in OxFunc
- decision: retain this as the current W075 floor and continue because the gate is healthy and the next lanes are explicit

Gate assessment after literal preprocessing and strict context-free precompute:

- progress_as_expected: yes - focused semantics, replay fixtures, and W075-specific hoist/no-hoist tests pass; local Mandelbrot timing improved into the current sub-two-second average band
- implementation_doc_sync: yes - B075-05 and B075-08 notes now distinguish literal preprocessing, strict context-free precompute, trace preservation, and remaining hoisting policy lanes
- path_forward_clear: yes - next work is integration evidence plus later broader hoisting only where OxFunc metadata and OxFml dependency proofs agree
- decision: retain this slice as the current W075 floor; broader captured-helper/reference/context-policy hoisting is future optimizer scope outside W075

Gate assessment after captured-helper invariant cache experiment:

- progress_as_expected: no - the plan-shape and evaluator semantics were acceptable, but release-mode W075 timing regressed against the retained floor and showed higher variance
- implementation_doc_sync: yes - the experiment is recorded here as not promoted, and the code path was backed out
- path_forward_clear: yes - do not continue by adding per-expression lambda cache wrappers; future captured-helper hoisting needs a lower-overhead dependency plan or OxFunc-side batch cooperation before promotion
- decision: keep the strict helper-free context-free precompute as the B075-08 floor and move to another W075 lane

Gate assessment after conservative slot-only `LET` execution:

- progress_as_expected: yes - focused semantics, case-insensitive shadowing, replay fixtures, and W075-specific tests passed; local Mandelbrot timing improved from the prior retained `2018.738 ms` average to `1729.372 ms` average on the same three-run release fixture
- implementation_doc_sync: yes - B075-04, B075-05, and the timing table now describe the promoted slot-only `LET` floor
- path_forward_clear: yes - continue with broader generic plan execution and trace/template work; do not add function-specific handling in OxFml
- decision: retain this slice as the current W075 floor; later slices address the remaining current-scope capture and node-execution evidence

Gate assessment after frame-level `SurfaceCallScratch` stack pooling:

- progress_as_expected: yes - evaluator, replay, and W075-specific tests passed; local Mandelbrot timing improved from the slot-only `LET` `1729.372 ms` average to `1678.701 ms`, and the stateful `REDUCE` control improved from `13.031 ms` average to `12.406 ms`
- implementation_doc_sync: yes - B075-06 and the timing table now describe the retained scratch-stack pool
- path_forward_clear: yes - continue with trace/template and broader plan-execution lanes; node-indexed buffers should remain profiling-justified rather than assumed
- decision: retain this B075-06 slice as the current W075 floor and continue because the gate is healthy

Gate assessment after slotted direct `LAMBDA` invocation parameters:

- progress_as_expected: yes for structure and targeted Mandelbrot behavior; evaluator, case-insensitive shadowing, replay, and W075-specific tests passed, Mandelbrot improved from `1678.701 ms` average to `1664.063 ms`, and the simpler controls moved within local timing noise
- implementation_doc_sync: yes - B075-04 and the timing table now describe slotted direct invocation parameter binding
- path_forward_clear: yes - continue with remaining generic slot/capture and trace/template lanes; keep function semantics in OxFunc
- decision: retain this slot-frame correction as the current W075 floor

Gate assessment after shared stored lambda bodies and parameters:

- progress_as_expected: yes - evaluator, replay, and W075-specific tests passed; Mandelbrot improved from `1664.063 ms` average to `1451.915 ms`, and the `MAKEARRAY` control also improved
- implementation_doc_sync: yes - B075-04 and the timing table now describe shared compiled lambda-body and parameter-array ownership
- path_forward_clear: yes - continue with remaining generic callable/slot transport and trace/template lanes; no function-specific behavior was introduced
- decision: retain this representation change as the current W075 floor; the later isolated shared-parameter retest stayed effectively timing-neutral at `1450.754 ms` average

Gate assessment after narrowed captured-slot closure construction:

- progress_as_expected: yes - evaluator, case-insensitive shadowing, replay, and W075-specific tests passed; Mandelbrot improved from `1450.754 ms` average to `1414.288 ms`, and closure slot contents now track the named capture set rather than cloning the full helper slot vector
- implementation_doc_sync: yes - B075-04 and the timing table now describe narrowed captured-slot closures
- path_forward_clear: yes - continue with trace/template and broader plan-execution lanes; the capture model is healthier and no function-specific behavior was added
- decision: retain this capture-floor slice as the current W075 floor

Gate assessment after frame-owned mutable execution state:

- progress_as_expected: yes - focused evaluator, shadowing, replay, and W075 tests passed; release fixture stayed flat for Mandelbrot at `1415.335 ms` average and improved the smaller helper controls
- implementation_doc_sync: yes - B075-03 and the timing notes now describe frame-owned scratch, recursion budget, and stack-guard state
- path_forward_clear: yes - continue with trace/template and evidence lanes; node-indexed buffers remain optional and profiling-driven
- decision: retain this B075-03 substrate slice as the current W075 floor

Gate assessment after W075 trace-policy tests:

- progress_as_expected: yes - W075-focused tests now prove that value-only helper hot loops emit no prepared records and prepared-call mode still records the optimized slot-only `LET` path
- implementation_doc_sync: yes - B075-07 notes now describe trace emitters as the current W075 mechanism and keep static template extraction as profiling-driven follow-on only
- path_forward_clear: yes - continue with closure/evidence consolidation; no additional trace refactor is needed for the current W075 exit gate
- decision: retain trace emitters plus direct value-only/prepared-call evidence as the current B075-07 floor

Gate assessment after helper-key parameter metadata experiment:

- progress_as_expected: no - semantics held, but release-mode W075 timing regressed sharply and variance spiked when `LambdaParam` carried precomputed helper keys
- implementation_doc_sync: yes - the rejected experiment is recorded here and the code path was backed out
- path_forward_clear: yes - keep shared lambda bodies, but do not continue by enlarging parameter metadata without stronger profiling evidence
- decision: do not promote the helper-key metadata experiment; retain the shared-body/shared-parameter floor

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | yes - this W075 workset carries the optimization scope, gate evidence, retained slices, rejected experiments, and final timing table |
| 2 | Conformance matrix rows updated? | yes - no separate conformance matrix rows are affected by this implementation/performance workset; W075 evidence is captured in this workset and focused tests |
| 3 | At least one deterministic replay or timing artifact exists per in-scope behavior? | yes - W075 has focused semantic/trace tests plus the ignored release-mode timing fixture for all declared formula families |
| 4 | Cross-repo impact assessed and handoff filed if needed? | yes - W075 consumes OxFunc W095/W096 surfaces without changing coordinator-facing clauses; no new OxFunc or OxCalc handoff is required |
| 5 | All required tests pass? | yes - full `cargo test -p oxfml_core`, focused W075/evaluator/shadowing/replay tests, `cargo check`, `cargo fmt`, `git diff --check`, and the ignored release fixture pass for the closure floor |
| 6 | No known semantic gaps remain in declared scope? | yes - remaining optimizer ideas are documented as profiling-driven follow-ons, not W075 declared-scope semantic gaps |
| 7 | Completion language audit passed? | yes - rejected experiments are described as not promoted, local timings are not reported as pack-grade claims, and scope-limited closure is explicit |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | yes - no new blocker was encountered and no blocker update is required |

## Completion Claim Self-Audit

### Step 1: Scope Re-Read

Pass. W075 scope items are represented by exercised implementation and evidence:

- deterministic helper/Mandelbrot timing fixture exists and is ignored/manual,
- callable stack guards are coalesced while recursive safety tests pass,
- `EvaluationFrame` owns the current mutable execution substrate for trace, callable registry, root helper state, scratch stack, recursion budget, and stack-guard state,
- lambda parameters, direct invocation parameters, helper-batch parameters, helper-local reads, captured slots, and lambda-free `LET` locals use compiled slots where the current lexical model can do so safely,
- OxFml-owned control flow lowers to compiled nodes and ordinary function/operator calls stay behind OxFunc `SurfaceCallSite`,
- value-only and prepared-call trace behavior are explicitly tested,
- strict context-free precompute uses OxFunc metadata and preserves runtime-sensitive calls.

### Step 2: Gate Criteria Re-Read

Pass. Entry, Mid-Gate A, Mid-Gate B, Mid-Gate C, and Exit Gate criteria have local evidence in this workset, focused tests, full test run, and timing fixture output. Local wall-clock timings remain directional and are labelled as such.

### Step 3: Silent Scope Reduction Check

Pass. Static trace-template extraction, broader captured-helper hoisting, node-indexed buffers, and deeper slot/capture specialization are explicitly treated as future profiling-driven optimizer follow-ons rather than hidden W075 closure requirements. The current W075 exit gate is satisfied by trace emitters, strict metadata-backed precompute, frame-owned scratch stack, and documented before/after timing evidence.

### Step 4: "Looks Done But Is Not" Pattern Check

Pass. No scaffolding-only path is counted as implementation. The captured-helper invariant cache and helper-key parameter metadata experiments are documented as rejected and backed out. No function-specific semantic fast path was added in OxFml.

## Status

- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- claim_confidence: validated

Future optimizer follow-ons, outside W075 closure scope unless reopened:

- node-indexed scratch/buffer addresses if profiling shows frame-level scratch-stack reuse is insufficient,
- broader captured-helper or reference-sensitive hoisting only with OxFunc metadata and OxFml dependency proof support,
- static trace-template extraction only if trace-on profiling identifies material overhead,
- wider formula-graph planning across workbook scheduling boundaries.
