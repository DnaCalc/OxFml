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
8. post-evaluation format hints when the semantic lane requires them,
9. catalog, feature/profile, and provider-availability distinctions where function admission or runtime behavior depends on them.

## 3. Design Tests That Drive The Boundary
The current boundary is primarily shaped by these function families:
1. aggregate semantics such as `SUM`,
2. reference-returning semantics such as `OFFSET`, `INDEX`, `INDIRECT`, and `XLOOKUP`,
3. caller-context-sensitive semantics such as `ROW`, `COLUMN`, and explicit `@`,
4. text and formatting-sensitive semantics such as `TEXT`, `VALUE`, `DOLLAR`, `FIXED`, `NOW`, and `TODAY`,
5. host-query semantics such as `CELL` and `INFO`.

These are not random examples. They are the current proof points for whether OxFml is preserving enough structure.

## 3A. Library Context And Catalog Snapshot Boundary
OxFunc should own the canonical function/operator catalog semantics.
OxFml should consume that world through an externally supplied library-context snapshot rather than through hidden global registry state.

The current intended split is:
1. OxFunc owns canonical function and operator ids, aliases, localized names, semantic traits, function profiles, and capability declarations,
2. OxFml owns parse, bind, semantic-plan, and evaluation behavior that consumes a versioned library-context snapshot,
3. the library-context snapshot should remain externally allocated and versioned rather than globally owned by OxFunc.
4. dynamic registrations from add-in, VBA, user-defined, or later provider-backed sources should be representable as snapshot truth without requiring OxFunc-owned hidden global state.

Minimum library-context concerns that must remain representable are:
1. canonical function/operator identity,
2. alias and localized name tables,
3. semantic trait/profile references,
4. feature, compatibility, or host-profile gates,
5. provider/add-in presence or absence where those states materially change formula admission or execution behavior,
6. registration source kind where add-in, built-in, provider-backed, or other sources materially affect admission or diagnostics,
7. snapshot identity/versioning strong enough for early rejection, bind, semantic planning, and replay correlation.

Current local floor:
1. OxFml now preserves `library_context_snapshot_ref` on the semantic plan when an external snapshot is supplied,
2. availability summaries are stage-aware across parse/bind, semantic-plan, runtime-capability, and post-dispatch/provider lanes,
3. per-surface availability summaries now preserve a smaller concrete field floor for:
   - `surface_stable_id`
   - `name_resolution_table_ref`
   - `semantic_trait_profile_ref`
   - `gating_profile_ref`
4. transport remains intentionally open beyond those preserved semantic distinctions.

Current integration preference:
1. OxFunc has now exposed a first-pass pinned machine-readable catalog snapshot export and reading guide,
2. OxFml should treat that export as the current best downstream stabilization artifact rather than continuing note-only catalog narrowing,
3. the snapshot should be suitable for OxFml-side semantic-plan consumption, replay correlation, and generated test synthesis rather than only for human note exchange,
4. the minimum useful export should include:
   - snapshot identity/version,
   - source commit/tag identity,
   - canonical surface ids,
   - aliases/localized-name table refs or embedded tables,
   - semantic trait/profile refs,
   - gating profile refs,
   - registration source kind,
   - stage-aware availability fields where already known,
5. richer profile/detail registries may remain separate as long as the exported snapshot points to them stably.

Current first-pass downstream artifact:
1. `../OxFunc/docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
2. `../OxFunc/docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`

Current OxFml reading of that artifact:
1. the export is useful now for initial consumption and mismatch discovery,
2. `snapshot_id`, `snapshot_generation`, `source_commit_short`, `source_commit_full`, `source_tree_state`, canonical surface ids, `registration_source_kind`, `special_interface_kind`, `preparation_owner`, `runtime_boundary_kind`, and `interface_contract_ref` are already useful first-pass fields,
3. refreshed ordinary extracted rows such as `FUNC.CHOOSECOLS`, `FUNC.FILTER`, `FUNC.UNIQUE`, and `FUNC.VSTACK` are now useful first-pass planning and test-synthesis inputs rather than mere catalog placeholders,
4. OxFml now consumes selected seam-heavy and ordinary rows from this export directly in local semantic-plan tests,
5. exact shared field names and fuller dereferenceable profile bundles remain open.

Working rule:
1. preserve the semantic distinction first,
2. keep the exact transport or runtime ownership shape open until later narrowing,
3. do not require OxFunc to own hidden mutable registry state just to express catalog truth.

## 3B. Operator, Literal, And Value-Universe Boundary
The OxFml/OxFunc seam should keep lexical/grammar ownership distinct from semantic value/operator ownership rather than smoothing the boundary into one generic language layer.

Current intended split:
1. OxFml owns lexical grammar, parse structure, localized separators, literal tokenization, and precedence/associativity handling,
2. OxFunc owns semantic value-universe meaning, canonical operator identities, operator/function semantics, coercion policy, and result-class behavior,
3. catalog/profile availability may influence which operators or functions are admitted semantically, but that does not move raw lexical parsing ownership out of OxFml.

Current examples:
1. decimal, group, currency, and localized literal spelling are lexical and locale-sensitive on the OxFml side,
2. operator meaning and result-class behavior are semantic and catalog-owned on the OxFunc side,
3. compatibility/profile gates may influence admission, but they should remain explicit rather than hidden inside parse normalization.

Working rule:
1. keep the tension explicit where needed,
2. do not force OxFml to own semantic operator truth just because it owns grammar,
3. do not force OxFunc to own localized literal parsing just because it owns semantic value meaning.

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
8. a typed callable carrier when the result is semantically callable
   - at minimum origin kind, invocation model, capture mode, and arity.

Current local floor:
1. callable helper values now additionally carry a typed callable carrier for helper-lambda origin, typed-invocation-only admission, lexical-vs-none capture mode, and arity,
2. callable helper values also carry structured callable detail for arity, parameter names, capture names, and body kind,
3. helper capture reporting is now based on exact free-helper capture rather than every helper name merely in scope,
4. parameter shadowing and unused helper bindings must not appear as false captures in callable carriers,
5. those fields are still a baseline carrier and not yet the final shared callable transport contract.

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

## 6A. Callable-Value Boundary
Callable helper and lambda values should be treated as first-class semantic values even when publication or worksheet-surface display restrictions still apply.

The current intended split is:
1. OxFml owns helper syntax, sequential binding, shadowing, lambda formation, lexical capture, and invocation planning,
2. OxFunc should not need raw helper AST ownership to apply callable semantics,
3. OxFunc should be able to consume callable values through a typed carrier or typed invocation facility without losing lexical meaning.

Current OxFml baseline:
1. helper forms are exercised locally,
2. lexical helper capture is preserved semantically,
3. callable results now carry a typed minimum carrier for origin, invocation model, capture mode, and arity,
4. that callable floor now also covers helper-bound and defined-name callable bindings, so callable meaning is no longer confined to immediate helper-local scope in the local proving floor,
5. typed invocation over an opaque callable identity is now exercised locally for `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY` rather than remaining a note-only boundary,
6. the current callable-value carrier remains provisional and replay-summary-oriented rather than finalized as a shared downstream transport.

Current narrowing direction:
1. the next smaller shared carrier may legitimately converge toward an opaque callable identity plus minimum semantic fields,
2. parameter names, capture names, and body-kind detail do not necessarily belong in that minimum carrier if provenance/replay surfaces preserve them explicitly,
3. any opaque callable identity is still too weak if origin, capture mode, arity shape, or invocation contract become unrecoverable,
4. workbook Defined Name callable preservation should be treated as first-pass seam pressure rather than deferred callable transport cleanup,
5. higher-order helper evidence from OxFunc is now strong downstream pressure, and OxFml now has matching local evidence for `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY`: their `W044` catalog rows are consumed directly in local semantic-plan tests and their runtime lanes now execute through a local typed callable invoker,
6. that newer runtime evidence narrows the open question toward minimum callable carrier and provenance fields rather than whether typed invocation itself is viable.

Working rule:
1. publication restrictions on callable values remain separate from the question of whether callable values are semantically admissible,
2. lexical capture must not be approximated away by dynamic helper-name lookup,
3. callable capture reporting must not be approximated by "all visible helpers" when exact free-helper capture is knowable,
4. named callable bindings adopted into OxFml-defined name context may preserve callable meaning and typed invocation without forcing immediate publication-policy closure,
5. transport details remain open, but callable-value meaning must stay recoverable.
6. parameter-name, capture-name, and body-kind detail may remain provenance/replay detail rather than minimum transport fields if the smaller shared callable carrier still preserves lexical meaning honestly.

## 6AA. Host-Provider And Subscription Seam Pattern
Some function families should be treated as prepared request plus typed host/provider outcome seams rather than as ordinary pure-function kernels or generic provider fetches.

Current pressure example:
1. `RTD`

Current intended split:
1. OxFml owns parse, bind, semantic planning, prepared request formation, and higher-level host/runtime orchestration concerns such as topic lifecycle, subscription tables, callback threading, and recalc scheduling,
2. OxFunc may own admitted call shape, typed request/result projection, and worksheet-semantic interpretation of typed provider outcomes once a prepared request is supplied,
3. broader lifecycle/runtime orchestration should remain above OxFunc rather than being smuggled into the function-semantic carrier.

Working rule:
1. prefer a prepared request plus typed host/provider outcome surface,
2. keep host/provider capability denial, connection failure, and provider error outcomes typed and distinct,
3. do not use this pattern to reopen the callable boundary broadly unless a coordinator-visible consequence or stronger OxFml-local evidence requires it.

## 6B. Availability, Feature, And Provider Gating Boundary
OxFml and OxFunc need a shared way to distinguish function availability states without collapsing them into one generic unknown-function bucket.

The current minimum state families are:
1. known in catalog,
2. feature-gated,
3. compatibility-gated,
4. host-profile unavailable,
5. add-in absent,
6. provider unavailable.

Working split:
1. catalog- and profile-defined availability belongs primarily in library-context truth,
2. runtime host or provider presence belongs primarily in capability view or runtime execution truth,
3. early formula rejection, runtime `#NAME?`, typed capability denial, and provider-failure outcomes must remain distinguishable.

Current edit-path reading:
1. OxFml may reject an edit attempt before formula adoption when the submitted string cannot honestly enter the canonical parse/bind artifact ladder,
2. OxFml may also accept the formula text as syntactically and artifact-valid while preserving unresolved-name or unresolved-bind facts in `BoundFormula` or `SemanticPlan`,
3. in that accepted-but-unresolved lane, OxFunc remains authoritative for the eventual `#NAME?` value payload and broader value-universe behavior,
4. OxFml remains authoritative for the unresolved-name classification, bind diagnostics, and stage at which the failure first became knowable.

Current stage-oriented reading:
1. parse and bind should be able to observe catalog-known, alias/localization, feature-gated, and compatibility-gated states where early admission depends on them,
2. semantic planning should preserve the relevant availability/gating summary rather than collapsing it into one generic unsupported marker,
3. runtime capability view should carry genuinely host- or session-dependent unavailable states,
4. post-dispatch or runtime execution may still surface provider-failure outcomes that are distinct from both early unknown-name classification and capability denial,
5. the stage split is only honest if the semantic-plan layer preserves enough identity to explain which catalog surface and which gate/profile produced the observed state.

Current local evidence now includes dedicated deterministic fixtures for:
1. accepted formula text with unresolved-name classification,
2. semantic-plan feature-gated classification,
3. runtime capability denial,
4. post-dispatch provider-unavailable classification.

Working rule:
1. do not collapse accepted-but-unresolved formula state into edit rejection,
2. do not collapse unresolved-name classification into provider-failure or runtime capability denial,
3. do not collapse OxFml bind-time knowledge into OxFunc-owned value payload ownership.

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
6. which function-semantic traits must be mandatory in the first execution-profile surface exposed to the core engine,
7. what is the smallest honest shared library-context snapshot shape,
8. which callable-value facts must cross the boundary beyond opaque identity plus typed invocation,
9. which availability and provider states belong in library context versus runtime capability view.

Current next useful narrowing step:
1. stop relying on note-only catalog narrowing,
2. consume the current pinned OxFunc catalog export,
3. use that export to replace narrow local catalog assumptions with broader OxFml-side snapshot-backed tests.

## 15. Current Round Stabilization Posture
The current OxFml reading is that this seam round has reached the point of diminishing returns for note-only narrowing.

Working rule:
1. treat the current canonical OxFml seam docs as the active semantic baseline,
2. keep preserving semantic distinctions even where the exact transport or carrier remains open,
3. do not read provisional candidate type sketches from note exchange as locked API commitments,
4. reopen this seam for narrower stabilization only when a concrete trigger appears.

Current trigger examples are:
1. a canonical minimum field set needs to be locked,
2. a proving-host or replay artifact forces a narrower carrier,
3. an implementation-facing handoff needs a more explicit transport decision,
4. a coordinator-visible consequence emerges from availability, provider-failure, callable publication, or a related lane.
