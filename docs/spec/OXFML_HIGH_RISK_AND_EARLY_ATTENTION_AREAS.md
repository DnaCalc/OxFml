# OxFml High-Risk and Early-Attention Areas

## 1. Purpose
This document identifies the OxFml areas that carry the highest semantic or architectural risk and therefore need attention early in implementation.

It exists to keep the repo from drifting into low-risk surface work while the real seam and concurrency hazards remain implicit.

Read together with:
1. `OXFML_IMPLEMENTATION_BASELINE.md`
2. `OXFML_FORMAL_ARTIFACT_REGISTER.md`
3. `formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
4. `fec-f3e/FEC_F3E_DESIGN_SPEC.md`

## 2. Working Rule
High-risk items are not necessarily first by volume.
They are first by semantic irreversibility, cross-lane coupling, or concurrency impact.

Implementation should prefer resolving these early enough that later code does not harden around the wrong shape.

## 3. Current High-Risk Areas
### 3.1 Explicit `@`, implicit intersection, and `_xlfn.SINGLE`
Risk:
1. stored-form normalization can erase visible `@`,
2. semantic provenance can still matter even when stored text changes,
3. caller-context-sensitive scalarization is easy to collapse too early,
4. `_xlfn.SINGLE` compatibility and round-trip behavior may require version/profile honesty.

Why this is high risk:
1. parser, binder, semantic planning, and OxFunc dispatch all touch it,
2. once collapsed to a plain value path, the needed distinction is difficult to reconstruct,
3. concurrency and replay need the scalarization route to remain explicit.

Early attention rule:
1. treat `@` and `SINGLE` as a design-test lane in `W003`,
2. ensure prepared-call semantics can distinguish scalar, reference, array payload, and spill-linked inputs before scalarization.

### 3.2 `LET`, `LAMBDA`, and helper-family evaluation environments
Risk:
1. these functions are not only grammar cases, they also create scoped evaluation environments,
2. closure capture, parameter binding, and evaluation order can leak across parser/binder/semantic-plan boundaries,
3. lazy and selective evaluation semantics can interact with function-local environments.

Why this is high risk:
1. parser acceptance alone is misleading,
2. implementation shortcuts here can break future helper families such as `MAP`, `SCAN`, `REDUCE`, `BYROW`, and `BYCOL`,
3. environment representation affects replay and formalization shape early.

Early attention rule:
1. treat `LET` and `LAMBDA` as semantic-plan design-test lanes, not just formula-language acceptance lanes,
2. require explicit environment and evaluation-order modeling before helper-family implementation expands.

### 3.3 Reference-preserving versus value-materialized execution
Risk:
1. functions like `OFFSET`, `INDEX`, `INDIRECT`, and `XLOOKUP` may produce or preserve references rather than plain values,
2. aggregates and scalarizing operators observe different behavior depending on whether reference identity survived,
3. dynamic arrays and spill-linked references complicate the distinction.

Why this is high risk:
1. it cuts across binder output, semantic planning, OxFunc dispatch, and FEC/F3E publication,
2. an overly value-centric implementation would distort multiple function families at once.

Early attention rule:
1. normalize reference ADTs and reference-preserving execution paths early in `W002` and `W003`,
2. do not treat dereference as the default architecture.

### 3.4 Execution and scheduling profiles for concurrency
Risk:
1. not all functions will be equally safe for concurrent or async evaluation,
2. some functions may be thread-safe, some main-thread-only, some host-query dependent, some async/oracle dependent, and some may impose ordering or single-flight constraints,
3. the core engine may need formula- or cell-level execution profiles exposed from OxFml/OxFunc surfaces to schedule safely.

Why this is high risk:
1. concurrency retrofits are expensive,
2. if semantic plans do not expose execution-safety metadata early, OxCalc would be forced to infer scheduler-critical constraints too late,
3. TLA+ and replay work depend on this distinction being modeled rather than hidden.

Early attention rule:
1. semantic plans must carry execution/scheduling profile requirements early,
2. OxFml must leave room to expose formula- or call-level execution profiles to the core engine,
3. `W003` and `W004` should treat this as an early seam requirement even before Stage 2 concurrency implementation begins.

### 3.5 Host-query and host-coupled semantic lanes
Risk:
1. `CELL`, `INFO`, and similar lanes are not pure local kernels,
2. host facts can be capability-gated, profile-gated, or non-thread-safe,
3. host-query behavior can affect scheduler admissibility as well as function semantics.

Why this is high risk:
1. the clean-room boundary can blur if raw host objects leak,
2. concurrency and replay need typed host-fact and denial surfaces,
3. host-query paths may be exactly the functions that require scheduler restrictions.

Early attention rule:
1. keep host-query capability views typed and capability-scoped,
2. allow execution profiles to mark host-coupled lanes explicitly.

### 3.6 Semantic formatting and locale/date-system services
Risk:
1. formatting-sensitive and locale-sensitive semantics can cross parse, bind, OxFunc dispatch, and invalidation behavior,
2. `TEXT`, `VALUE`, `NOW`, `TODAY`, and related lanes may introduce publication hints or dependency tokens.

Why this is high risk:
1. it is easy to demote this to UI formatting too early,
2. format and locale dependence can affect both correctness and recalculation scheduling.

Early attention rule:
1. keep semantic formatting in the evaluator/seam world,
2. treat format-dependency tokens as runtime-significant facts rather than presentation-only details.

## 4. First Attention Order
The current recommended early-attention order is:
1. `@` / `SINGLE` and reference-preserving scalarization
2. `LET` / `LAMBDA` environment semantics
3. execution and scheduling profiles for concurrency and async safety
4. host-query and host-coupled semantic lanes
5. semantic formatting and locale/date-system dependency lanes

## 5. Workset Implications
Current expected primary owners:
1. `W002`: normalized reference ADTs and parser/binder handling needed by `@`, `#`, and helper forms
2. `W003`: `@` / `SINGLE`, `LET` / `LAMBDA`, provenance minimums, host-query boundary, and execution-profile metadata
3. `W004`: FEC/F3E runtime consequences, candidate/commit/reject handling, surfaced scheduler-relevant facts
4. `W005`: replay and formal witnesses for these high-risk lanes
5. `W006`: formatting and host-query follow-through
6. `W007`: execution-profile narrowing and concurrency-contract follow-through
7. `W008`: single-formula host and empirical-oracle bootstrap
8. `W013`: broader parser/binder surface and incremental reuse for high-risk reference lanes
9. `W014`: wider semantic breadth and OxFunc catalog expansion over helper, scalarization, and host-query lanes
10. `W015`: Stage 2 runtime/contention hardening for concurrency-sensitive reject and effect paths
11. `W016`: checked local formal artifacts and model runs over the runtime and replay floor
12. `W017`: replay promotion toward `cap.C4.distill_valid`
13. `W018`: broader proving-host and empirical-oracle follow-through
14. `W019`: remaining high-value reference breadth and formula-language closure
15. `W020`: broader OxFunc semantic breadth and callable-value follow-through
16. `W021`: async-facing runtime and scheduler-surface follow-through
17. `W022`: broader checked formal families over the widened runtime and replay floor
18. `W023`: retained witness-set and replay-promotion follow-through
19. `W024`: DNA OneCalc host-policy and empirical-pack planning follow-through
20. `W031`: `MS-OE376` rule review and gap classification for structured refs, R1C1, external names, and related formatting-rule surfaces
21. `W032`: OxFunc catalog, callable transport, and provider-stage closure
22. `W033`: broader replay promotion toward `cap.C4.distill_valid`
23. `W034`: distributed runtime and coordinator consequence boundary
24. `W035`: broader checked formal families over replay-promotion and distributed-runtime surfaces

## 6. Working Rule
If a design or implementation decision touches one of these lanes:
1. make the risk explicit,
2. avoid simplifying it by erasing semantic distinctions,
3. add replay/formal hooks early rather than promising to recover them later.
