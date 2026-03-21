# Notes for OxFunc

Status: `active`
Owner lane: `OxFml`
Relationship: outbound observation and seam-status note from OxFml for the next integration round with OxFunc

## 1. Purpose
Record the current OxFml-side semantic, runtime, and replay floor that OxFunc should use for the next upstream/downstream integration round.

This note is not a generic status dump.
It only records the distinctions and exercised behaviors that matter at the OxFml/OxFunc boundary.

## 2. Core Message
OxFml has materially widened the local semantic and proving-host floor since the earlier seam acknowledgment.

For the next OxFunc coordination round, the main points are:
1. helper-form lanes now have an exercised local baseline, not only preserved syntax,
2. callable helper values now have lexical-capture-sensitive behavior in the OxFml local floor,
3. caller-context, scalarization, host-query, formatting, and capability-sensitive lanes are all now represented in replayable local artifacts,
4. OxFml still preserves semantic distinctions and avoids freezing prematurely on a final transport shape.

## 3. Current Evidence In OxFml
The following OxFml canonical docs and exercised local artifacts now carry the active seam floor:

### 3.1 Canonical seam docs
1. `docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
2. `docs/spec/OXFML_CANONICAL_ARTIFACT_SHAPES.md`
3. `docs/spec/OXFML_MINIMUM_SEAM_SCHEMAS.md`
4. `docs/spec/OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`
5. `docs/spec/OXFML_DNA_ONECALC_HOST_POLICY_BASELINE.md`
6. `docs/spec/OXFML_TEST_LADDER_AND_PROVING_HOSTS.md`

### 3.2 Exercised local evidence
1. `crates/oxfml_core/tests/evaluator_tests.rs`
2. `crates/oxfml_core/tests/semantic_plan_tests.rs`
3. `crates/oxfml_core/tests/replay_fixture_tests.rs`
4. `crates/oxfml_core/tests/host_tests.rs`
5. `crates/oxfml_core/tests/fixtures/semantic_plan_replay_cases.json`
6. `crates/oxfml_core/tests/fixtures/prepared_call_replay_cases.json`
7. `crates/oxfml_core/tests/fixtures/single_formula_host_replay_cases.json`
8. `crates/oxfml_core/tests/fixtures/empirical_oracle_scenarios.json`

## 4. Observations That Matter To OxFunc
The following observations are now mature enough to surface explicitly.

### 4.1 Helper forms are no longer only a preservation concern
OxFml now has an exercised local floor for:
1. `LET` sequential helper binding,
2. helper-name shadowing,
3. `LAMBDA` literal formation,
4. immediate invocation,
5. helper-bound invocation.

This remains a local baseline rather than a finished cross-repo callable-value contract.

### 4.2 Lexical capture matters
OxFml local execution showed that helper lambdas must preserve lexical capture rather than re-reading helper names dynamically once shadowing is possible.

Working implication:
1. helper-profile and callable-value lanes should assume lexical, not dynamic, meaning,
2. any later OxFunc transport for callable helper values must not silently erase capture-sensitive meaning.
3. OxFml now also uses exact free-helper capture for callable summaries/carriers rather than reporting every helper merely visible in scope.

### 4.3 Callable-value carriers are still provisional
OxFml currently exposes helper-produced callable values through a replayable summary surface rather than a richer downstream-shared carrier.

That means:
1. the existence of callable helper values is now explicit,
2. the current local carrier is enough for replay, planning, and diagnostics,
3. the local carrier now distinguishes exact lexical capture from no-capture even under unused helper bindings and parameter shadowing,
4. the final OxFml/OxFunc callable-value carrier remains intentionally open.
5. OxFml now also has a local registry-backed typed invocation bridge, so callable meaning no longer stops at formation time for the first higher-order helper lanes.

### 4.4 Scalarization and caller-context lanes are exercised locally
The local floor now includes exercised coverage for:
1. explicit `@`,
2. `_xlfn.SINGLE` / `SINGLE`,
3. caller-context-sensitive evaluation lanes,
4. direct-cell-binding proving-host cases where defined names are insufficient.

OxFunc should assume these distinctions are now part of the exercised OxFml seam floor, not only draft spec text.

### 4.5 Host-query and formatting lanes are no longer only planning lanes
The local floor now includes exercised host-query and formatting-sensitive cases including:
1. `TEXT`,
2. `INFO`,
3. `CELL("filename", ...)`,
4. reference-sensitive host/query proving-host cases.

Active-selection-sensitive omitted-reference `CELL(...)` pressure from `HO-FN-002` remains acknowledged in principle, but the broader host-query carrier shape is still open.

## 5. Interface Implications
For the next integration round, OxFunc can rely on the following OxFml-side direction:
1. prepared arguments preserve source, structure, reference, blankness, and caller-context distinctions explicitly,
2. prepared results preserve result class, reference identity, and publication/format-oriented metadata explicitly,
3. semantic plans now carry helper-environment profile information, not only function-trait and execution-profile information,
4. semantic plans and compile surfaces now explicitly leave room for a versioned external library-context snapshot rather than hidden global registry state,
5. host-query lanes remain capability-scoped and typed rather than object-handle based,
6. proving-host and replay artifacts now preserve direct cell bindings where semantic truth depends on concrete cell resolution,
7. execution-profile and host/query sensitivity are visible in formula-level artifacts so downstream scheduler or host policy does not need to invent them.

## 6. Minimum Invariants
The following invariants remain mandatory on the OxFml side:
1. direct scalar input is not interchangeable with array-like input,
2. omitted argument, blank cell, empty string, and error remain distinct,
3. reference-returning meaning is not collapsed into unconditional eager dereference,
4. caller-context-dependent scalarization remains explicit and replayable,
5. typed host-query views remain capability-scoped,
6. helper-form shape facts remain modelable in semantic plans,
7. helper-name shadowing must not change the meaning of an already-created helper lambda,
8. direct cell bindings must be preserved in proving-host or retained-witness artifacts whenever semantic truth depends on them.

## 7. Open OxFml-Side Gaps Still Relevant To OxFunc
The following lanes remain intentionally open:
1. the smallest final provenance vocabulary for `PreparedArgument` and `PreparedResult`,
2. the final placement of explicit `@` semantics in the execution pipeline,
3. the final compatibility and round-trip treatment of `_xlfn.SINGLE(...)`,
4. the first locked execution-profile vocabulary for downstream scheduler consumption,
5. the exact typed carrier shape for broader host-query return families,
6. the final shared carrier for callable helper values beyond the current replayable summary surface,
7. the smallest honest shared library-context snapshot shape,
8. the split between library-context availability truth and runtime capability/provider-failure truth,
9. broader OxFunc catalog breadth beyond the current exercised local floor,
10. broader higher-order helper coverage beyond the now-exercised `MAP` / `REDUCE` / `SCAN` / `BYROW` / `BYCOL` / `MAKEARRAY` lanes.

## 8. Requests For The Next OxFunc Round
The next useful OxFunc-side feedback would be:
1. which callable-value facts OxFunc would need beyond the current helper summary carrier,
2. whether any currently expected function traits are still missing from the OxFml semantic-plan profile,
3. whether the current host-query capability split is sufficient for the next `CELL` / `INFO` tightening pass,
4. whether OxFunc wants to converge first on provenance vocabulary or callable-value carrier shape.

The next useful OxFunc-side integration artifact was more valuable than another broad note pass, and that artifact now exists in downstream `W044`.

The next useful OxFunc-side integration move is now:
1. keep the current pinned machine-readable catalog snapshot export stable enough for bounded OxFml-side consumption,
2. refine the export only where concrete OxFml consumption mismatches show that the first pass is not self-describing enough.

Minimum suitability for OxFml-side continued development:
1. stable snapshot id/version,
2. source commit/tag identity,
3. canonical function/operator surface ids,
4. alias/localized-name mapping truth or stable refs to it,
5. semantic trait/profile refs,
6. gating profile refs,
7. registration source kind,
8. stage-aware availability fields where already known,
9. a shape OxFml can consume in tests and semantic-plan compilation without manual note transcription.

Preferred use on the OxFml side:
1. pin OxFml semantic-plan tests to that snapshot,
2. synthesize broader catalog-consumption fixtures from it,
3. reserve later `NOTES_FOR...` cycles for concrete mismatches found through that integration path.

## 9. New OxFunc Intake Processed On The OxFml Side
The current upstream note at `../OxFunc/docs/upstream/NOTES_FOR_OXFML.md` materially aligns with the current OxFml direction, but it sharpens three areas that were previously too implicit on the OxFml side.

The most important intake points now processed are:
1. the shared seam should preserve semantic requirements first and keep transport/mechanism open until later narrowing,
2. OxFunc wants a versioned external library-context snapshot rather than hidden global registry ownership,
3. callable helper values should be treated as first-class semantic values even if publication restrictions remain separate,
4. availability and provider states need a cleaner split between library-context truth and runtime capability/runtime-failure truth,
5. OxFunc remains aligned with the current OxFml position that `#` normally resolves upstream and does not require a default spill-provenance flag once fully resolved,
6. the operator/literal/value-universe tension should stay explicit rather than being hidden behind a falsely clean ownership split,
7. the new narrowed OxFunc stage focus is now external library-context snapshot, callable-value minimum carrier, and availability/provider taxonomy,
8. the latest OxFunc narrowing asks for a more concrete minimum library-context field set and a stage-aware mapping of availability states before callable transport is narrowed further.

The resulting OxFml-side refinements are now reflected in the canonical boundary docs as:
1. explicit library-context snapshot wording,
2. explicit callable-value boundary wording,
3. explicit availability/profile/provider gating wording,
4. explicit operator/literal/value-universe ownership wording,
5. explicit stage-aware reading for availability and provider-failure states.

Current OxFml reading of what still remains open after that intake:
1. the final shared callable-value carrier still remains open,
2. the smallest honest shared library-context snapshot still remains open,
3. the split between early formula rejection, runtime `#NAME?`, typed capability denial, and provider-failure outcomes is now narrower locally but still not final,
4. the exact catalog-backed boundary for operator admission versus pure grammar ownership still needs narrower exercised closure.

## 9A. Current W044 Snapshot Export Intake
OxFml has now processed the current downstream `W044` snapshot-export attempt as the right next-step artifact for this seam.

Current downstream artifact:
1. `../OxFunc/docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
2. `../OxFunc/docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`

Current OxFml reading:
1. this is the right kind of artifact to move the exchange forward,
2. it is useful immediately as a first-pass downstream stabilization artifact,
3. it is not yet a final shared snapshot ABI or final cross-repo field lock,
4. OxFml now has direct local consumption tests for selected seam-heavy and ordinary rows from this export rather than relying only on synthetic local snapshots.

Current pinned first-pass downstream snapshot for the next bounded integration round:
1. `snapshot_id = oxfunc-libctx-v1`
2. `snapshot_generation = 2026-03-21`
3. `source_commit_short = 717831e`
4. `source_commit_full = 717831ed354bcf713c0defe718c5910016b07d3a`
5. `source_tree_state = dirty`

Fields OxFml can already use as-is in the current first pass:
1. `snapshot_id`
2. `snapshot_generation`
3. `source_commit_short`
4. `source_commit_full`
5. `source_tree_state`
6. canonical function/operator surface ids
7. `registration_source_kind`
8. `special_interface_kind`
9. `preparation_owner`
10. `runtime_boundary_kind`
11. `interface_contract_ref`

Additional first-pass rows OxFml now reads as directly useful:
1. seam-heavy rows:
   - `FUNC.LET`
   - `FUNC.LAMBDA`
   - `FUNC.RTD`
   - `FUNC.OP_IMPLICIT_INTERSECTION`
2. ordinary-but-useful extracted rows:
   - `FUNC.CHOOSECOLS`
   - `FUNC.FILTER`
   - `FUNC.UNIQUE`
   - `FUNC.VSTACK`

Current OxFml reading of those ordinary extracted rows:
1. they are already useful planning and test-synthesis inputs,
2. they reduce the need for an immediate special-case side channel for dynamic-array reshaping families,
3. they are a good first check for whether broader ordinary catalog consumption can replace narrow local metadata sooner than expected,
4. OxFml now consumes those rows directly in local semantic-plan tests rather than only acknowledging them in note form.

Fields OxFml reads as useful but still candidate rather than locked:
1. `admission_interface_kind`
2. `arity_shape_note`
3. OxFunc-local semantic/gating reference fields where the current export points to them indirectly rather than via stable normalized bundles

Current OxFml-side alternatives or refinements:
1. `interface_contract_ref` is a useful first-pass bridge only if it denotes stable semantic contract material rather than ephemeral implementation notes,
2. the provenance triple `source_commit_short` + `source_commit_full` + `source_tree_state` is now usable for immediate test pinning and mismatch reports,
3. a stable ref/tag field would still be welcome later, but it is no longer a near-term blocker for bounded OxFml-side consumption,
4. `arity_shape_note` is useful as explanatory metadata, but it does not yet read as a stable hot-path seam field,
5. `admission_interface_kind`, `special_interface_kind`, `preparation_owner`, and `runtime_boundary_kind` are useful first-pass split fields, but OxFml does not yet treat those exact names as locked shared vocabulary.

## 10. OxFml Topic Split For The Next Round
The current OxFml-side working split is:

### 10.1 Library-context snapshot
This is where OxFml currently expects to carry:
1. canonical function/operator ids,
2. aliases and localized names,
3. semantic trait/profile references,
4. feature and compatibility gates,
5. add-in/provider-presence and related registration truth that affects early formula admission or later execution planning,
6. registration source kind where it materially affects admission, diagnostics, or replay,
7. snapshot identity/versioning strong enough for parse, bind, semantic planning, and replay correlation.

Current narrower local floor:
1. `surface_stable_id`,
2. `name_resolution_table_ref`,
3. `semantic_trait_profile_ref`,
4. `gating_profile_ref`.

Current OxFml-side next ask:
1. the first-pass export now exists and should be treated as the current integration artifact,
2. continue providing stable reading guidance where exported fields are not yet self-describing,
3. keep the provenance triple (`source_commit_short`, `source_commit_full`, `source_tree_state`) preserved in downstream mismatch reports so a bounded dirty-tree export is not confused with a clean release snapshot,
4. a stable ref/tag field would still be useful when convenient, but it is now optional improvement rather than a standing export-shape gap,
5. keep pushing toward dereferenceable semantic/gating profile bundles without blocking first-pass consumption now.

### 10.2 Prepared arguments and prepared results
This is where OxFml currently expects to carry:
1. source and structure class,
2. reference identity,
3. blankness class,
4. evaluation mode,
5. caller-context-sensitive scalarization facts,
6. a typed minimum callable carrier for origin kind, invocation model, capture mode, and arity,
7. helper-result callable summary/detail facts until a richer carrier is agreed,
8. the same minimum callable carrier and summary/detail floor when a callable value is preserved through adopted defined-name context in the current local proving floor,
9. a typed invocation path over opaque callable identity that is now exercised locally for `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY`, plus a first defined-name callable higher-order lane through `MAP(...,NamedLambda)`.

### 10.3 Host capability view
This is where OxFml currently expects to carry:
1. active-selection-sensitive host-query inputs,
2. workbook, environment, and referenced-cell fact access,
3. runtime availability/denial states that are genuinely host- or session-dependent,
4. provider/service availability when it is a runtime capability issue rather than a catalog truth issue.

### 10.3A Stage-oriented availability reading
Current OxFml-side stage split is:
1. parse/bind: catalog-known, alias/localization, feature-gated, and compatibility-gated states where early admission depends on them,
2. semantic-plan: preserved availability/gating summary for later execution and replay classification,
3. runtime capability: genuinely host- or session-dependent unavailable states,
4. post-dispatch/runtime: provider-failure outcomes that remain distinct from both early unknown-name classification and capability denial.

Current exercised local reading:
1. semantic-plan fixtures now preserve post-dispatch provider-unavailable separately from runtime add-in-absent and host-profile-unavailable states,
2. managed-session fixtures still keep runtime capability denial distinct from later external-provider consequence surfacing.
3. OxFml canonical docs now also say explicitly that edit rejection before artifact adoption is a different lane from accepted formula text that later produces unresolved-name classification and OxFunc-owned `#NAME?` value results.
4. OxFml now also has a dedicated deterministic fixture family covering accepted-unresolved-name, semantic-plan gated, runtime capability denied, and post-dispatch provider-unavailable classification as distinct lanes.

### 10.4 Transport intentionally left open
OxFml still wants to keep these transport details open for now:
1. the final callable-value carrier,
2. the smallest honest shared library-context snapshot shape,
3. the final split between runtime capability denial and provider-failure reporting in replay-facing surfaces.

## 11. OxFml Current Stabilization Order
If the next round needs a narrowed working set, OxFml currently agrees with this stabilization order:
1. external library-context snapshot,
2. availability / feature-gate / provider-failure taxonomy,
3. callable-value minimum carrier.

Reason for this order:
1. library-context truth affects parse, bind, semantic planning, and early rejection broadly,
2. availability/provider taxonomy affects both admission and replay classification,
3. callable-value carrier narrowing is important, but it is safer once the surrounding catalog and availability surfaces are less ambiguous.

Current alignment note:
1. OxFml and OxFunc now appear aligned on the same three next-round topics,
2. the remaining mismatch is mainly preferred ordering, not topic selection,
3. OxFml is still prioritizing availability/provider taxonomy slightly earlier than OxFunc, but that is now a sequencing preference rather than a scope conflict,
4. the latest OxFunc narrowing also suggests callable transport should remain intentionally looser for one more round while library-context and stage-aware availability surfaces become more concrete.

## 12. Current OxFml Reply For The Next Seam Sync
The main new OxFml-side facts for the next sync are:
1. typed invocation over opaque callable identity is now locally exercised, not hypothetical,
2. inline and helper-bound local lambdas now run end-to-end through `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY`,
3. adopted defined-name callable values now preserve a distinct `DefinedNameCallable` runtime origin instead of being flattened into the helper lane, and `MAP(...,NamedLambda)` is now exercised locally,
4. the remaining callable seam question is now primarily the minimum carrier/provenance field split, not whether the invocation boundary works.

The bounded next-round OxFunc ask is:
1. react to the exercised typed-invocation floor rather than the older boundary-only reading,
2. narrow the minimum shared callable carrier field set,
3. say which callable fields may stay structured provenance only,
4. keep `ISOMITTED` and broader higher-order family expansion as deferred evidence-driven lanes unless the current export already forces a smaller carrier decision.

## 11A. Current Round Closure Reading
OxFml now reads the latest OxFunc note as a round-closure signal rather than as a request for indefinite further note-only narrowing.

Current OxFml reading:
1. the current OxFml canonical seam docs are strong enough to act as the active upstream baseline for ongoing OxFunc work,
2. the current three-topic stabilization order remains useful, but it should now be reopened only when a concrete trigger appears,
3. this is enough alignment to proceed with function work without pretending the final carrier or transport details are already locked,
4. this does not mean the seam is finalized,
5. OxFml now has a narrower local callable floor where callable values can survive helper scope adoption into defined-name context without losing typed invocation or lexical-capture meaning.

Current trigger examples for the next narrower round are:
1. OxFunc needs a concrete minimum library-context field set locked,
2. OxFunc needs a smaller callable-value carrier because a proving-host or implementation slice can no longer stay transport-open,
3. availability/provider-failure handling starts forcing narrower exercised closure in replay, diagnostics, or runtime outcome typing,
4. any of the above begins changing coordinator-visible consequences and therefore risks an OxCalc-facing seam packet.

## 11B. Focused Next Round For `LET` / `LAMBDA`
OxFml is now prepared to pin down the `LET` / `LAMBDA` seam more directly.

Current OxFml prep posture:
1. lexical, not dynamic, helper meaning should now be treated as fixed,
2. exact free-helper capture should now be treated as fixed where OxFml can know it,
3. callable values are semantically first-class even when publication policy remains narrower,
4. the current local floor now also preserves callable meaning through adopted defined-name context,
5. the next round should narrow carrier and invocation shape, not reopen helper-scope meaning.

The exact next-round questions OxFml now wants to settle are:
1. the smallest honest shared callable carrier,
2. the split between callable carrier fields and provenance/replay detail,
3. the callable invocation boundary,
4. callable-specific interaction with stage-aware availability/provider states.

OxFml has written a focused local prep note for this at:
1. `docs/spec/formula-language/OXFML_OXFUNC_LET_LAMBDA_PIN_DOWN_PREP.md`

Current OxFml preferred order for this narrower round is:
1. fix lexical/capture truths,
2. fix minimum callable carrier,
3. fix carrier vs provenance split,
4. fix invocation boundary,
5. only then narrow callable-specific stage interaction where still needed.

Current processed OxFunc reply:
1. lexical meaning, exact capture truth, and callable-first-class status are now aligned enough to treat as fixed for the next round,
2. OxFunc prefers a smaller minimum callable carrier centered on opaque callable identity plus semantic minimums,
3. OxFunc currently wants parameter-name, capture-name, and body-kind detail to remain provenance/replay detail rather than minimum carrier fields,
4. OxFunc prefers typed invocation over a narrower callable carrier rather than richer direct inspection.

Current OxFml response:
1. incorporated: richer callable detail may move out of the minimum carrier and remain structured provenance/replay detail,
2. incorporated: typed invocation over a narrower callable carrier is the right direction,
3. proposed alternative: opaque callable identity is acceptable only if origin kind, capture mode, arity shape, and invocation-contract meaning remain recoverable as typed semantic fields,
4. proposed alternative: `invocation_contract_ref` should denote stable semantic invocation meaning, not an implementation-specific callback/ABI handle,
5. still intentionally open: full defined-name/UDF callable transport, final callable publication policy, and broader callable/provider interaction.

Current additional intake from the latest OxFunc note:
1. OxFunc now explicitly prefers to keep callable-specific availability/provider typing inside the generic staged availability model unless concrete callable cases prove that insufficient,
2. OxFunc now explicitly prefers not to promote parameter-name, capture-name, and body-kind detail into the shared hot-path carrier by default,
3. OxFunc now frames `callable_token`, `arity_shape`, and `invocation_contract_ref` as the likely next narrower callable carrier vocabulary,
4. OxFunc is using broader local helper-function evidence, including higher-order lanes, to justify that narrower callable direction.

Current OxFml response to that additional intake:
1. incorporated: do not invent a special callable-only availability taxonomy before evidence requires it; the generic staged availability model remains the right default,
2. incorporated: keep richer callable detail out of the minimum hot-path carrier by default as long as structured provenance/replay detail preserves it,
3. proposed alternative: OxFml does not yet adopt `callable_token`, `arity_shape`, or `invocation_contract_ref` as canonical names; those are acceptable candidate labels, not yet locked OxFml-local vocabulary,
4. incorporated with narrower scope: `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY` are now exercised locally and are valid seam-pressure evidence for the invocation boundary and minimum carrier discussion; `ISOMITTED` remains the main deferred higher-order evidence lane,
5. still intentionally open: whether the eventual minimum callable carrier needs an explicit invocation-model field in addition to any future `callable_token`/`invocation_contract_ref` pair.

## 12. What This Note Does Not Authorize
This note should not be read as authorizing:
1. a final shared callable-value carrier,
2. a final locked provenance vocabulary,
3. full OxFunc catalog closure on the OxFml side,
4. replacement of typed host-query capability views with raw workbook or host objects,
5. approximation of lexical helper capture by dynamic helper-name lookup.

## 13. Current OxFml Position On Follow-Up
No OxCalc-facing handoff is being filed from this intake alone.

Current OxFml reading:
1. the new OxFunc note tightens semantic-boundary planning and canonical wording,
2. it does not by itself change coordinator-facing FEC/F3E clauses,
3. if later callable-value publication, availability gating, or provider-failure handling starts changing coordinator-visible consequences, that would be the right point to open an OxCalc-facing seam packet.

## 14. Working Rule
Until the open lanes narrow further:
1. treat OxFml canonical seam docs as the active semantic baseline,
2. treat the current local helper/callable floor as exercised but still provisional,
3. treat missing exact transport type names as open design detail, not as missing semantic acknowledgment,
4. do not keep reopening the note exchange without a concrete trigger such as a catalog-export mismatch, field-set lock, proving-host pressure, implementation-facing handoff need, or coordinator-visible consequence,
5. a few bounded back-and-forth rounds are acceptable when they are tied to concrete artifacts or concrete mismatches rather than abstract naming debate.

## 15. Current OxFml Message For The Next Integration Rounds
OxFml now treats this note as the current outbound baseline for the next bounded integration rounds rather than as a reason to reopen the whole seam.

What OxFml is incorporating as the settled reading for this round:
1. semantic requirements stay primary and transport remains provisional,
2. library-context truth stays above runtime capability/provider truth,
3. callable values remain first-class semantic values,
4. the generic staged availability model remains the default for callable lanes,
5. richer callable detail stays out of the minimum hot-path carrier by default if structured provenance/replay detail preserves it,
6. typed invocation over a narrower callable carrier is the current preferred direction,
7. workbook Defined Name callable preservation should now be treated as first-pass seam pressure rather than as a late extension,
8. `RTD`-like host/provider seams should be modeled as prepared request plus typed host/provider outcome surface, not as ordinary provider-fetch kernels.

What OxFml is explicitly not locking in this round:
1. final canonical field names such as `callable_token`, `arity_shape`, or `invocation_contract_ref`,
2. final minimum callable carrier field set,
3. final callable carrier versus provenance split,
4. final placement of callable/provider interaction beyond the generic staged availability model,
5. final direct `LET` / `LAMBDA` formation artifact shape,
6. exact `RTD` edge-case matrix or broader generalized provider/subscription contract,
7. higher-order callable seam pressure inferred only from OxFunc-local evidence.

Deferred until further OxFml-local evidence and future worksets:
1. minimum callable carrier closure beyond the current narrowed candidate remains deferred within `W032`,
2. broader higher-order callable lanes beyond the now-exercised `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY` floor remain deferred to `W040`,
3. fuller defined-name/UDF/interoperable callable transport remains deferred until later evidence after `W032` and `W038`,
4. any callable/provider-stage refinement beyond the generic staged availability model remains deferred until local evidence proves the generic model insufficient.

Working rule for the next bounded integration rounds:
1. do not use these rounds merely to debate names or speculative transport shapes,
2. use the next round first to consume the current pinned machine-readable OxFunc catalog snapshot export and identify concrete field or interpretation mismatches,
3. then use follow-on rounds only for narrower locks exposed by snapshot consumption, proving-host/runtime artifacts, Defined Name callable transport pressure, or typed host/provider seams such as `RTD`,
4. if those triggers do not appear, keep the remaining issues deferred to future worksets rather than continuing note churn.

## 16. Current Convergence Check
OxFml has reprocessed the latest `../OxFunc/docs/upstream/NOTES_FOR_OXFML.md` and reads it as convergent with the current OxFml status for this stage of the exchange.

Converged reading:
1. semantic requirements remain primary while transport stays provisional,
2. external library-context truth remains above runtime capability/provider truth,
3. callable values remain first-class semantic values,
4. the generic staged availability model remains the default for callable lanes,
5. richer callable detail remains outside the minimum hot-path carrier by default if structured provenance/replay preserves it,
6. typed invocation over a narrower callable carrier remains the preferred direction,
7. candidate field labels such as `callable_token`, `arity_shape`, and `invocation_contract_ref` are still only candidate labels rather than locked shared OxFml vocabulary,
8. OxFunc-local higher-order callable evidence is useful pressure but not yet upstream seam-lock evidence on the OxFml side,
9. workbook Defined Name callable preservation is now strong first-pass seam pressure even though fuller interoperable callable transport remains deferred,
10. `RTD` currently reads as a prepared request plus typed host/provider outcome seam rather than a reason to reopen the callable boundary broadly.

Deferred-work mapping:
1. OxFunc now records its deferred callable follow-up under `W042`,
2. OxFml records the matching upstream-side deferred callable evidence and seam-reopen lane under `W040`,
3. these should be read as corresponding future-work owners rather than as a fresh disagreement.

Current integration reading:
1. there is no active semantic disagreement left in the current note exchange,
2. the remaining callable/library-context/provider questions are now concrete artifact, field-set, transport-shape, and evidence-maturity questions,
3. OxFml is prepared for a few bounded integration rounds driven first by concrete snapshot-consumption mismatches in `W044`, then by narrower proving-host/runtime pressure where needed,
4. the refreshed ordinary dynamic-array rows in `W044` are now a useful early check for whether broader ordinary catalog consumption can proceed without a special-case side channel,
5. OxFml now also consumes the higher-order helper rows (`MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, `MAKEARRAY`) from `W044` directly in local semantic-plan tests and now executes them locally through a typed callable invoker for inline and helper-bound lambdas, which narrows the remaining mismatch toward minimum carrier/provenance fields rather than invocation viability,
6. OxFml now also has a first higher-order defined-name callable lane (`MAP(...,NamedLambda)`), so adopted defined-name callable origin is no longer only a note-level concern,
7. absent those triggers, the remaining questions remain intentionally deferred until later OxFml-local evidence or a narrower implementation-facing trigger appears.

## 17. Immediate Next Sync Agenda
OxFml wants the next OxFunc sync to stay bounded and artifact-driven.

### 17.1 What OxFml Will Treat As Settled Going In
1. lexical rather than dynamic helper meaning,
2. exact free-helper capture where OxFml can know it,
3. callable values as semantically first-class,
4. typed invocation over opaque callable identity as viable for the currently exercised higher-order lanes,
5. the generic staged availability model as the default callable/provider model unless evidence forces something narrower.

### 17.2 What OxFml Wants To Review In The Next Sync
1. concrete `W044` field usefulness and mismatch reports for callable-relevant rows:
   - `FUNC.LET`
   - `FUNC.LAMBDA`
   - `FUNC.MAP`
   - `FUNC.REDUCE`
   - `FUNC.SCAN`
   - `FUNC.BYROW`
   - `FUNC.BYCOL`
   - `FUNC.MAKEARRAY`
   - `FUNC.ISOMITTED`
2. whether the current first-pass split
   - `admission_interface_kind`
   - `preparation_owner`
   - `runtime_boundary_kind`
   - `interface_contract_ref`
   is already sufficient for OxFml semantic planning on those rows or needs a narrower replacement,
3. the smallest honest callable carrier field set, specifically whether the shared minimum needs typed fields for:
   - origin kind
   - capture mode
   - arity shape
   - invocation-contract meaning
4. whether OxFunc still sees any need for an explicit invocation-model field beyond a future `callable_token` plus `invocation_contract_ref`-style pair.

### 17.3 What OxFml Is Explicitly Not Asking To Reopen
1. lexical versus dynamic helper semantics,
2. whether typed invocation can work at all,
3. full defined-name/UDF/interoperable callable transport,
4. final worksheet publication policy for callable values,
5. generalized provider/subscription contracts inferred only from `RTD`.

### 17.4 Preferred Output Of The Next Sync
1. a short field-level acknowledgment of which `W044` callable-row fields OxFunc expects OxFml to rely on now,
2. a short list of callable-minimum fields OxFunc now thinks are semantically required versus provenance-only,
3. any concrete row or interpretation mismatches OxFunc wants OxFml to treat as blockers for the next `W032` slice,
4. explicit confirmation if `ISOMITTED` should remain deferred rather than being used as a seam-lock driver yet.
