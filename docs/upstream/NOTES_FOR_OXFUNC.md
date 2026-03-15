# Notes for OxFunc

Status: `active`
Owner lane: `OxFml`
Relationship: outbound acknowledgment and seam-status note from OxFml after rereading `../OxFunc/docs/upstream/NOTES_FOR_OXFML.md`

## 1. Purpose
Record what OxFml has now explicitly incorporated from the OxFunc upstream note, what remains intentionally open, and which parts of the current seam are ready for OxFunc to rely on semantically.

## 2. Core Message
OxFml has reprocessed the current OxFunc note and accepts its main direction:
1. preserve semantic distinctions,
2. do not freeze on OxFunc's provisional transport sketches,
3. keep the mapping from preserved upstream distinctions to OxFunc semantic needs explicit.

The current OxFml position is:
1. direct scalar versus array-like remains a required distinction,
2. value-only versus reference-visible behavior remains a required distinction,
3. value-only result versus may-return-reference result remains a required distinction,
4. caller-context, `@`, host-query, and execution-restriction lanes are now all treated as first-class seam concerns rather than later glue.

## 3. Current Evidence In OxFml
The following OxFml canonical docs now directly carry the upstream pressures:
1. `docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
   - prepared argument/result minimums
   - caller-context and `@` handling
   - typed host-query capability boundary
   - execution-profile boundary
2. `docs/spec/formula-language/OXFML_NORMALIZED_REFERENCE_ADTS.md`
   - reference atoms
   - runtime-reference expressions
   - spill and dynamic-reference separation
3. `docs/spec/OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`
   - code-facing `parse -> bind -> compile_semantic_plan -> evaluate -> commit` transform chain
   - proving-host helper surface
4. `docs/spec/OXFML_CANONICAL_ARTIFACT_SHAPES.md`
   - `PreparedArgument`
   - `PreparedCall`
   - `PreparedResult`
   - `SemanticPlan`
5. `docs/spec/OXFML_MINIMUM_SEAM_SCHEMAS.md`
   - typed host-query capability view baseline

Additional acknowledgment from the latest OxFunc note:
1. omitted-reference `CELL(info_type)` forms depending on active selection are now explicitly acknowledged in the OxFml host-query baseline,
2. OxFml now treats selection-context support as a real host-query capability concern rather than assuming caller context is always sufficient.
3. OxFml explicitly acknowledges the upstream host-query handoff lane `HO-FN-002` as incorporated in principle, with the active-selection requirement now promoted into the local host-query seam baseline.
4. OxFml now carries a first semantic-plan helper-environment profile for `LET`, `LAMBDA`, helper invocation, and lexical helper capture requirement, plus replayable helper-result summaries for current local callable-value lanes.
5. OxFml execution in the helper-form lane showed that lexical capture must be preserved semantically and cannot be approximated by dynamic helper-name lookup once shadowing is possible.

## 4. Interface Implications
OxFunc can now assume the following OxFml-side direction:
1. prepared arguments will carry structure/source/reference/evaluation/blankness distinctions as canonical fields, not only payload,
2. prepared results will preserve result class, reference identity, and result metadata such as `format_hint` and `publication_hint`,
3. reference-returning expressions are preserved in bind/semantic planning rather than normalized eagerly to value-only paths,
4. host-query lanes such as `CELL` and `INFO` are modeled through typed capability views rather than raw workbook objects or ad hoc callbacks,
5. execution-profile metadata is expected to flow from OxFunc traits through OxFml semantic plans so later hosts/core engines can schedule safely without inventing function-safety rules themselves.
6. helper-form lanes now have a first plan-level profile and replayable callable-value summary surface on the OxFml side, so later OxFunc seam narrowing can refer to explicit helper-environment facts rather than only ad hoc evaluator behavior.
7. helper-bound callable values currently flow through a replayable lambda-summary carrier, and OxFml now treats that as a provisional seam surface rather than merely an internal debugging detail.

## 5. Minimum Invariants
The following are the current OxFml-side minimum invariants aligned to the OxFunc note:
1. direct scalar input is not interchangeable with array-like input,
2. omitted argument, blank cell, empty string, and error are not collapsed into one generic empty bucket,
3. reference-returning expressions are not forced through unconditional eager dereference,
4. caller-context-dependent scalarization remains explicit and replayable,
5. typed host-query views remain capability-scoped and may include active-selection support where `CELL` semantics require it,
6. result metadata needed for later publication remains modelable above the pure function kernel,
7. execution restrictions such as host-query dependence, thread affinity, async coupling, or serial-only lanes remain expressible in the semantic-plan world.
8. helper-form shape facts such as presence of `LET`, `LAMBDA`, helper invocation, and lexical helper capture requirement remain expressible in the semantic-plan world.
9. helper-name shadowing must not change the meaning of an already-created helper lambda; lexical capture is the current OxFml baseline.

## 6. What Remains Open
The following are still intentionally open on the OxFml side:
1. the smallest final provenance vocabulary for `PreparedArgument` and `PreparedResult`,
2. the exact placement of explicit `@` semantics in the execution pipeline,
3. the exact compatibility and round-trip treatment of `_xlfn.SINGLE(...)`,
4. the first locked execution-profile vocabulary that later schedulers will consume,
5. the exact typed carrier shape needed, if any, for multi-item host-query return lanes such as the `CELL(\"width\", ref)` dual-item case,
6. whether callable helper values should remain summary-carried in OxFunc `EvalValue::Lambda(String)` form or move to a richer downstream-shared carrier later.

## 7. Working Rule For OxFunc Coordination
Until these open lanes narrow further:
1. treat the OxFml canonical docs above as the current semantic baseline,
2. treat missing exact type names as open transport details rather than missing acknowledgment,
3. file further upstream observations when a function-semantic requirement depends on a distinction not yet represented in the OxFml canonical docs.
