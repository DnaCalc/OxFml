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

### 4.3 Callable-value carriers are still provisional
OxFml currently exposes helper-produced callable values through a replayable summary surface rather than a richer downstream-shared carrier.

That means:
1. the existence of callable helper values is now explicit,
2. the current local carrier is enough for replay, planning, and diagnostics,
3. the final OxFml/OxFunc callable-value carrier remains intentionally open.

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
9. broader OxFunc catalog breadth beyond the current exercised local floor.

## 8. Requests For The Next OxFunc Round
The next useful OxFunc-side feedback would be:
1. which callable-value facts OxFunc would need beyond the current helper summary carrier,
2. whether any currently expected function traits are still missing from the OxFml semantic-plan profile,
3. whether the current host-query capability split is sufficient for the next `CELL` / `INFO` tightening pass,
4. whether OxFunc wants to converge first on provenance vocabulary or callable-value carrier shape.

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
3. the split between early formula rejection, runtime `#NAME?`, typed capability denial, and provider-failure outcomes still needs narrower exercised closure,
4. the exact catalog-backed boundary for operator admission versus pure grammar ownership still needs narrower exercised closure.

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

### 10.2 Prepared arguments and prepared results
This is where OxFml currently expects to carry:
1. source and structure class,
2. reference identity,
3. blankness class,
4. evaluation mode,
5. caller-context-sensitive scalarization facts,
6. helper-result callable summary facts until a richer carrier is agreed.

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

## 11A. Current Round Closure Reading
OxFml now reads the latest OxFunc note as a round-closure signal rather than as a request for indefinite further note-only narrowing.

Current OxFml reading:
1. the current OxFml canonical seam docs are strong enough to act as the active upstream baseline for ongoing OxFunc work,
2. the current three-topic stabilization order remains useful, but it should now be reopened only when a concrete trigger appears,
3. this is enough alignment to proceed with function work without pretending the final carrier or transport details are already locked,
4. this does not mean the seam is finalized.

Current trigger examples for the next narrower round are:
1. OxFunc needs a concrete minimum library-context field set locked,
2. OxFunc needs a smaller callable-value carrier because a proving-host or implementation slice can no longer stay transport-open,
3. availability/provider-failure handling starts forcing narrower exercised closure in replay, diagnostics, or runtime outcome typing,
4. any of the above begins changing coordinator-visible consequences and therefore risks an OxCalc-facing seam packet.

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
4. do not keep reopening the note exchange without a concrete trigger such as a field-set lock, proving-host pressure, implementation-facing handoff need, or coordinator-visible consequence.
