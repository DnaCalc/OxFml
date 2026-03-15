# OxFml OxFunc Semantic Boundary

## 1. Purpose
This document defines the canonical semantic boundary between OxFml and OxFunc.

It is the OxFml-local promotion of the active upstream requirements recorded in `../OxFunc/docs/upstream/NOTES_FOR_OXFML.md`.

The goal is not to freeze an implementation-specific API prematurely.
The goal is to freeze the semantic distinctions that OxFml must preserve so OxFunc can implement Excel-compatible function behavior honestly.

The canonical field surfaces for prepared-call and prepared-result artifacts are defined in:
1. `../OXFML_CANONICAL_ARTIFACT_SHAPES.md`

The canonical minimum schema objects for publication deltas, spill events, typed reject contexts, and trace payloads are defined in:
1. `../OXFML_MINIMUM_SEAM_SCHEMAS.md`

## 2. Boundary Rule
OxFml must preserve the distinctions that Excel semantics actually depend on through parse, bind, semantic planning, and evaluation.

At minimum, the boundary must preserve:
1. direct scalar argument versus array-like argument,
2. value-only function versus reference-observable function,
3. value-only result versus may-return-reference result,
4. omitted argument, blank cell, empty string, and error as distinct states,
5. caller-context-sensitive scalarization and reference formation,
6. locale and format-service dependencies,
7. typed host-query capabilities for host-observing functions,
8. post-evaluation format hints when the semantic lane requires them.

## 3. Design Tests That Drive The Boundary
The current boundary is primarily shaped by these function families:
1. aggregate semantics such as `SUM`,
2. reference-returning semantics such as `OFFSET`, `INDEX`, `INDIRECT`, and `XLOOKUP`,
3. caller-context-sensitive semantics such as `ROW`, `COLUMN`, and explicit `@`,
4. text and formatting-sensitive semantics such as `TEXT`, `VALUE`, `DOLLAR`, `FIXED`, `NOW`, and `TODAY`,
5. host-query semantics such as `CELL` and `INFO`.

These are not random examples. They are the current proof points for whether OxFml is preserving enough structure.

## 4. Prepared Argument Contract
Prepared arguments must preserve more than payload alone.

The canonical prepared-argument surface must carry at least:
1. `value_view`
   - the scalar, array, blank, or error payload visible at the current semantic stage,
2. `structure_class`
   - whether the argument is direct scalar, array-like, omitted, or another semantically distinct prepared form,
3. `source_class`
   - whether the argument originated as direct syntax, reference syntax, reference-returning evaluation, spill result, or another explicit category,
4. `reference_identity`
   - present when the argument is reference-visible or reference-preserved,
5. `evaluation_mode`
   - eager, branch-lazy, fallback-lazy, reference-preserved, or other explicit policy,
6. `blankness_class`
   - omitted, blank cell, empty string, or other explicit empty-like state,
7. `caller_context`
   - row/column anchor and any other context needed for caller-sensitive semantics.

The exact type names may evolve. These semantic fields may not silently disappear.

## 5. Prepared Result Contract
Prepared results must preserve result shape and semantic class explicitly.

The canonical prepared-result surface must carry at least:
1. `payload`
2. `result_class`
   - scalar value, array payload, reference result, error result, or omitted-like result,
3. `structure_class`
4. `reference_identity`
5. `format_hint`
   - when a function semantically produces a worksheet-surface format recommendation,
6. `publication_hint`
   - when later seam or host layers need an explicit non-value publication decision.
7. helper-result provenance when the result is a helper-produced callable value
   - at minimum a replayable summary of arity, helper-capture presence, and body-shape class in the current baseline.

## 5A. Execution and Scheduling Profile Boundary
Some function traits materially affect whether a formula or call path is safely schedulable under concurrent or async calculation.

OxFunc should expose enough semantic trait information for OxFml to derive execution-profile metadata such as:
1. broadly concurrent-safe evaluation,
2. thread-affine or host-thread-required evaluation,
3. async or externally-coupled evaluation classes,
4. host-query or host-service dependence,
5. volatility or invalidation-sensitive execution classes,
6. branch-lazy, fallback-lazy, or other ordering-sensitive evaluation requirements.

Boundary rule:
1. OxFunc owns the function-semantic trait source,
2. OxFml owns the formula- and call-level semantic-plan profile derived from those traits plus formula structure,
3. OxCalc or other hosts may consume the exposed execution-profile result for scheduling, but that scheduler policy remains outside OxFunc and outside the evaluator seam contract itself.

## 6. Host-Query Capability Boundary
Some function families observe workbook, cell, or host facts rather than only local value payloads.

The current canonical examples are:
1. `CELL`
2. `INFO`

Working split:
1. OxFunc owns query-text normalization, query classification, and result-shaping policy.
2. OxFml/FEC/F3E owns the actual host-facing fact surface.
3. OxFunc must consume typed host-query facilities rather than raw workbook objects, parser internals, or arbitrary callbacks.

For helper-form lanes, OxFml also owns the formula-level helper environment profile derived from formula structure.
That profile may carry facts such as:
1. presence of `LET`,
2. presence of `LAMBDA`,
3. presence of helper invocation,
4. whether lexical helper capture is required,
5. summary counts or arity ceilings needed for replay, planning, or later OxFunc seam narrowing.

The minimum host-query capability families are:
1. selected cell metadata by preserved reference identity,
2. selected workbook facts,
3. selected application or environment facts,
4. active-selection context for omitted-reference query lanes where Excel semantics depend on the selected cell rather than only the caller cell,
5. typed capability-denial outcomes when a query family is unavailable in the active profile or host mode.

Boundary rule:
1. host-query functions must not force OxFunc to depend on workbook object handles,
2. host-query answers must be replayable as typed facts or typed denials,
3. omitted-reference `CELL(info_type)` forms must be able to observe active-selection context when the host/profile admits it,
4. the query vocabulary may grow, but the transport must stay typed and capability-scoped.

## 7. Source and Structure Classes
The current minimum semantic vocabulary should distinguish at least:

### 7.1 Source classes
1. `DirectScalar`
2. `ArrayLikeValue`
3. `ReferenceNode`
4. `ReferenceReturningExpr`
5. `Omitted`

### 7.2 Structure classes
1. `DirectScalar`
2. `ArrayLike`
3. `Omitted`
4. `AdapterSynthesized`

These are minimums, not maximums. The vocabulary may grow where evidence requires it.

## 8. Reference Identity Requirements
OxFunc needs more than a rendered A1 string when reference identity matters.

Reference identity must be able to preserve:
1. workbook and sheet scope,
2. anchor information,
3. address mode,
4. area kind and area shape,
5. row/column displacement semantics where relevant,
6. enough identity to distinguish scalar cells, rectangular areas, rows, columns, and future richer shapes.

Reference identity must survive through reference-returning functions until a consuming semantic rule explicitly dereferences or flattens it.

## 9. Evaluation Mode Requirements
OxFml must make evaluation strategy explicit where Excel semantics depend on it.

The current minimum evaluation modes are:
1. `EagerValue`
2. `BranchLazy`
3. `FallbackLazy`
4. `ReferencePreserved`
5. `Selective`

These are required because:
1. `IF` needs branch laziness,
2. `IFERROR` needs fallback laziness,
3. `INDEX`, `OFFSET`, `INDIRECT`, and similar functions may require reference-preserved handling,
4. future families may require selective evaluation rather than universal eager normalization.

## 10. Caller Context and `@`
Caller context is not optional glue.
It is a semantic requirement for several function families and for explicit implicit intersection.

The boundary must preserve:
1. caller row/column context,
2. explicit `@` provenance even when stored-form normalization removes a visible `@`,
3. distinction between scalar, reference, array payload, and spill-linked result before scalarization,
4. scalarization route metadata for replay and diagnosis.

`@` must not be approximated by unconditional top-left selection.

## 11. Spill and `#`
Spill behavior creates a second important distinction:
1. the spill-anchor syntax itself,
2. the resolved reference/array/error outcome after spill resolution.

Current OxFunc pressure does not require OxFml to preserve a default spill-origin flag for every prepared value once `#` has been fully resolved.
It does require OxFml to preserve enough structure so that spill-linked semantics are not confused with generic array payloads before the right stage.

## 12. Text, Locale, and Formatting Services
OxFunc should not be forced to embed ad hoc locale or format-language logic.

OxFml and the surrounding seam context must provide explicit access to:
1. locale/profile identity,
2. workbook date system,
3. format-code language services,
4. locale-sensitive parse/render services,
5. post-evaluation format-hint propagation.

This is necessary for `TEXT`, `VALUE`, `DOLLAR`, `FIXED`, and time/date-producing functions such as `NOW` and `TODAY`.

Host-query functions such as `CELL` and `INFO` must consume typed host-query facilities from the same evaluator context world rather than ad hoc side channels.

## 13. Boundary Invariants
The following invariants are currently mandatory:
1. direct scalar input is not interchangeable with array-like input,
2. reference-returning expression is not interchangeable with already-dereferenced payload when reference identity matters,
3. omitted argument, blank cell, empty string, and error are not collapsed into one generic empty bucket,
4. reference identity survives until a consuming semantic rule explicitly discards it,
5. evaluation strategy remains visible where Excel semantics depend on non-eager behavior,
6. caller-context-dependent scalarization remains explicit and replayable,
7. locale and formatting services are explicit facilities, not hidden host-language fallbacks,
8. host-query functions consume typed capability-scoped fact views rather than raw host objects.

## 14. Open Questions
The most important open questions at this boundary are:
1. what is the smallest provenance vocabulary that keeps aggregate and reference semantics correct without overcommitting,
2. where explicit `@` semantics should live in the execution pipeline,
3. how to distinguish spill-derived values from generic array-like payloads at the minimum necessary level,
4. which prepared-result publication hints belong in OxFml versus FEC/F3E,
5. which host-query families belong in the minimum cross-profile baseline versus profile-gated extensions,
6. which function-semantic traits must be mandatory in the first execution-profile surface exposed to the core engine.
