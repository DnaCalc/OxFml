# Notes for OxCalc

Status: `active`
Owner lane: `OxFml`
Relationship: outbound observation and seam-status note from OxFml for the next integration round with OxCalc

## 1. Purpose
Record the current OxFml-side evaluator, runtime, replay, and host-boundary floor that matters to OxCalc coordination.

This note is an OxFml-owned observation ledger, not a mirror of OxCalc coordinator docs.

## 2. Core Message
OxFml has materially widened the local floor relevant to coordinator-facing integration.

For the next OxCalc coordination round, the important points are:
1. candidate, commit, reject, trace, and capability-sensitive runtime behavior are now exercised locally through a stronger managed-session floor,
2. replay governance, reduced-witness policy, retained-local witness policy, and pack-candidate planning are now explicit OxFml-owned artifacts,
3. a reduced-profile DNA OneCalc host boundary is now explicit so OxCalc integration can stay distinct from the proving-host lane,
4. OxFml still remains authoritative for evaluator artifact meaning, reject semantics, replay-safe identity, and fence meaning.

## 2AA. Consumer-Facing Runtime Refactor Direction
OxFml has now landed the consumer-facing interface refactor under `W054`.

Current OxFml message to OxCalc:
1. this is a packaging and consumer-entry refactor, not a reopening of the OxFml <-> OxFunc seam,
2. OxCalc is the primary downstream consumer of the planned runtime facade,
3. the canonical consumer packet for that runtime-facing migration is now:
   - `docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
4. OxCalc should plan against the runtime facade as the intended consumer
   contract now,
5. coordinator policy and publication semantics remain OxCalc-owned above that
   facade.

Current OxFml rule for OxCalc:
1. do not build new long-term OxCalc wrappers around the current flat OxFml root if they can be avoided,
2. do not fork frozen OxFml/OxFunc packet vocabulary while preparing for the runtime facade,
3. treat the coming runtime facade as a consumer-entry cleanup over the existing host/runtime packet floor, not as a new semantic seam,
4. treat the current consumer packet as preserving the already-consumed runtime
   packet truth rather than replacing it with facade-only vocabulary.

## 2AB. Short Runtime-Facade Response To OxCalc
OxFml's current short answer to OxCalc's runtime-facade requests is:
1. yes, OxFml will publish and maintain a concrete current-surface to
   target-surface migration table as part of `W054`,
2. yes, `RuntimeEnvironment`, `RuntimeFormulaRequest`,
   `RuntimeFormulaResult`, and `RuntimeSessionFacade` are the intended first
   runtime-facing object set,
3. yes, provider-plus-pin library-context selection remains first-class and
   explicit,
4. yes, candidate / commit / reject / trace families remain visible in the
   runtime-facing result/session path,
5. yes, cleaner packaging does not close the narrower `W026` residuals by
   implication.

Current OxFml finish-pass after the latest OxCalc note is:
1. `RuntimeFormulaResult` now explicitly names the first stable correlation
   subset:
   - `candidate_result_id`
   - `commit_attempt_id`
   - `reject_record_id`
   - optional fence snapshot references,
2. the runtime-facing contract now explicitly preserves surfaced
   execution-restriction, capability-sensitive, topology/effect, and
   dependency-sensitive fact families in reachable consumer form,
3. the runtime-facing contract now explicitly keeps caller-context carriage for
   the first admitted caller-sensitive subset and adds a caller-context
   dependence signal where OxFml can surface it honestly,
4. OxFml continues to treat these as bounded finish-work on the runtime-facing
   contract, not as a broad seam reopen,
5. OxFml now treats the consumer contract as the landed `OxFml_V1` surface for
   downstream implementation.

## 2A. New Coordination Packet: Host Runtime and External Requirements
OxFml now has a single canonical draft intended to anchor the next host/coordinator seam round:
1. `docs/spec/OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md`

This draft unifies:
1. the direct single-formula host lane,
2. the OxCalc-integrated host lane,
3. the runtime `LibraryContextProvider` and snapshot direction shared with OxFunc,
4. the first typed host/query and return-surface packets already being exercised in `W041`, `W042`, and `W043`.

Current OxFml reading:
1. this draft is the first honest implementation-facing host contract for the currently covered scope,
2. it is not yet shared seam-freeze text agreed with OxCalc,
3. it is the right bounded packet for the next OxCalc coordination round.

## 2B. What OxFml Is Asking OxCalc To Review In This Round
Please review the new host/runtime draft topic-by-topic and answer whether the current floor is sufficient for a first host implementation slice.

The bounded review topics are:
1. direct-host versus OxCalc-integrated host split:
   - is the responsibility split clear enough for implementation planning,
   - are any coordinator-owned concerns still missing from the integrated-host side,
2. required inputs:
   - formula and structure inputs,
   - direct-cell and defined-name bindings,
   - typed host-query and provider families,
   - runtime library-context provider and immutable snapshot requirements,
   - capability and fence inputs where the session path is used,
3. required outputs:
   - candidate, commit, reject, and trace families,
   - `ReturnedValueSurface` split,
   - coordinator-relevant ids and consequence categories,
4. implementation sufficiency for the currently covered local scope:
   - proving-host and single-formula direct host,
   - OxCalc consumption of current candidate/commit/reject/effect carriers,
   - current admitted host-query/provider slice:
     - `INFO`
     - `CELL`
     - `RTD`,
5. explicit non-assumptions and deferrals:
   - deferred provider families remain out of scope,
   - full scheduler/distributed policy remains out of scope,
   - full product-host specification remains out of scope.

Current OxFml request:
1. say which of the above topics are already sufficient for a first coordinator-host implementation slice,
2. say which are only `canonical but narrower`,
3. identify any topic that now requires a narrower handoff rather than another note-level clarification pass.

## 2C. Current Intake Of OxCalc's Host Runtime Review Pass
OxCalc has now reviewed the host/runtime draft and the current OxFml response is:
1. OxFml agrees with OxCalc's `already canonical` read for the first host/coordinator implementation slice on:
   - direct-host versus OxCalc-integrated host split,
   - required inputs,
   - required outputs,
   - implementation sufficiency for the currently covered local scope,
   - explicit non-assumptions and deferrals,
2. OxFml agrees that the host/runtime draft is strong enough for first implementation planning,
3. OxFml does not yet treat it as shared seam-freeze text,
4. no new formal handoff is warranted from this review pass alone.

Current caution points for OxCalc are:
1. do not over-read the host/runtime draft as full language or full built-in-function closure,
2. do not over-read caller-anchor and address-mode carriage for the first TreeCalc relative-reference subset as already frozen in the host packet,
3. do not over-read execution-restriction transport as one final single frozen carrier,
4. do not over-read publication and topology breadth beyond the current local exercised floor,
5. do not over-read provider-failure or callable-publication as active coordinator-facing seam clauses yet.

Current OxFml answers to OxCalc's specific reply requests are:
1. yes, OxFml agrees with OxCalc's `already canonical` read for the first host/coordinator implementation slice,
2. the main over-read risks are the five caution points listed above,
3. yes, caller-anchor and address-mode handling for the first TreeCalc relative-reference subset should stay in the `W026` note lane and does not require a narrower handoff today,
4. yes, provider-failure and callable-publication should remain watch lanes only until they become coordinator-visible in exercised evidence.

## 2D. Current Intake Of OxCalc's Confirmation Pass
OxCalc has now processed the current host/runtime reply and OxFml reads that pass as convergent.

Current OxFml intake is:
1. OxCalc now treats the host/runtime packet as settled enough for first implementation planning,
2. OxCalc agrees caller-anchor and address-mode carriage should stay in the `W026` note lane,
3. OxCalc agrees provider-failure and callable-publication remain watch lanes only,
4. no new formal handoff is warranted from the current host/runtime packet alone.

Current OxFml working rule after this pass:
1. treat the host/runtime packet as converged for first-slice planning,
2. keep `W045` open until the underlying `W041` / `W042` / `W043` local packet floor is stronger than it is today,
3. reopen the OxCalc note lane only on concrete mismatch, not for more general host/runtime clarification.

## 2E. Structured-Reference And Table Ownership Planning Read
OxFml is now tightening the intended ownership split for structured table references under `W036`.

Current OxFml planning read:
1. tables should be owned on the host/coordinator side in the same broad sense that defined names are host-owned,
2. the host or coordinator should present table context explicitly:
   - table identity,
   - table range,
   - column catalog,
   - header/totals presence,
   - enclosing-table and caller-row context where omitted-table-name or `#This Row` forms depend on it,
3. OxFml should own structured-reference grammar, disambiguation against defined names, omitted-table-name interpretation, table-aware bind, and evaluator/FEC consequences once that context is supplied,
4. OxCalc should not expect OxFunc to own workbook table objects or table-context reconstruction.

Current intended first packet direction:
1. `table_catalog`
2. `enclosing_table_ref`
3. `caller_table_region`

Current planning question for future OxCalc review:
1. is that the right first host/coordinator packet for the TreeCalc/direct-host split,
2. or does OxCalc need a narrower or differently factored table-context carrier for first implementation use?

## 2F. Structured-Reference Packet Refinement
OxFml has now compared the `W036` plan again against the current Foundation-backed `MS-OE376` extraction and wants the next OxCalc read on a slightly sharper packet.

Current refined minimum requirements:
1. if a structured reference omits `table-name`, the formula must belong to an enclosing table and that table identity must be supplied by the host/coordinator,
2. `table-name` must denote a real table rather than being silently treated as a defined name,
3. `#This Row` is a true caller-row-sensitive selector lane,
4. `#This Row` must not be combined with `#Headers`, `#Total Row`, `#Data`, or `#All`.

Current refined first packet direction:
1. `table_catalog`
2. `enclosing_table_ref`
3. `caller_table_region`

Current refined packet meaning:
1. `table_catalog` should carry stable table identity, range, columns, and header/totals presence,
2. `enclosing_table_ref` should make omitted-table-name forms honest for direct host and TreeCalc-integrated use,
3. `caller_table_region` should carry the first row/region-sensitive facts needed for `#This Row`, header/data/totals distinctions, and bind-time admissibility.

Current OxFml planning read:
1. table ownership remains with host/OxCalc like other workbook objects,
2. OxFml should own grammar, disambiguation against defined names, bind normalization, and evaluator/FEC consequences once the packet is supplied,
3. OxFunc should stay out of table-object ownership and consume only normalized reference/value consequences downstream.

Current OxFml question back to OxCalc:
1. is `table_catalog + enclosing_table_ref + caller_table_region` the right first semantic packet for the direct-host and TreeCalc split,
2. does OxCalc need any narrower anchor or region facts in that first packet before `W036` execution starts,
3. should totals/header/data region identity remain explicit in the packet even if the first executed table floor is smaller than the full workbook table model.

## 3. Current Evidence In OxFml
The following OxFml canonical docs and exercised artifacts now carry the relevant coordinator-facing floor:

### 3.1 Canonical docs
1. `docs/spec/fec-f3e/FEC_F3E_DESIGN_SPEC.md`
2. `docs/spec/fec-f3e/FEC_F3E_TESTING_AND_REPLAY.md`
3. `docs/spec/fec-f3e/FEC_F3E_SCHEMA_REPLAY_FIXTURE_PLAN.md`
4. `docs/spec/fec-f3e/FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md`
5. `docs/spec/OXFML_REPLAY_APPLIANCE_ADAPTER_V1.md`
6. `docs/spec/OXFML_DNA_ONECALC_HOST_POLICY_BASELINE.md`
7. `docs/spec/OXFML_EMPIRICAL_PACK_PLANNING.md`

### 3.2 Exercised local evidence
1. `crates/oxfml_core/tests/session_service_tests.rs`
2. `crates/oxfml_core/tests/session_replay_fixture_tests.rs`
3. `crates/oxfml_core/tests/replay_adapter_and_witness_tests.rs`
4. `crates/oxfml_core/tests/replay_retained_and_host_policy_tests.rs`
5. `crates/oxfml_core/tests/fixtures/session_lifecycle_replay_cases.json`
6. `crates/oxfml_core/tests/fixtures/replay_bundle_normalization/pack_candidate_index.json`
7. `crates/oxfml_core/tests/fixtures/witness_distillation/`
8. `formal/lean/OxFmlSessionLifecycle.lean`
9. `formal/tla/FecSessionLifecycle.tla`

## 4. Observations That Matter To OxCalc

### 4.1 Managed session behavior is stronger than the earlier seam floor
The current local floor exercises:
1. typed invalid phase transitions,
2. stale-fence commit rejection,
3. capability denial paths,
4. abort and expiry no-publish paths,
5. surfaced execution-restriction effects on candidates.

This is still a local managed-runtime baseline, not full distributed or multi-host arbitration semantics.

### 4.2 Candidate/publication separation remains explicit
OxFml continues to preserve:
1. accepted candidate result versus committed publication,
2. typed reject outcomes,
3. reject-is-no-publish semantics,
4. fence and capability consequences as typed evaluator/runtime outputs rather than coordinator-invented meanings.

### 4.3 Replay governance has widened materially
OxFml now has explicit local policy and evidence for:
1. replay adapter capability claims through the current `cap.C3.explain_valid` floor,
2. reduced-witness planning and first reduced-witness executions,
3. retained-local witness policy,
4. non-pack-eligible pack-candidate normalization rehearsal artifacts,
5. quarantine and explanatory-only distinctions.

This does not yet claim pack-grade promotion and does not relax OxFml replay-safe transform constraints.

### 4.4 DNA OneCalc host policy is now explicit
OxFml now carries a reduced-profile host baseline for downstream single-formula proving-host use.

OxCalc should read that as:
1. an explicit boundary for what belongs to a reduced host lane,
2. not a replacement for OxCalc graph coordination,
3. not permission to collapse coordinator-facing semantics into the proving-host model.

### 4.5 Direct cell bindings matter
The current proving-host and replay policy now explicitly preserves direct cell bindings where semantic truth depends on concrete resolution.

Coordinator implication:
1. future retained witnesses or host/scenario packs must not collapse those lanes into name-only or prose-only artifacts when reference-sensitive truth depends on cell identity.

## 5. Interface Implications
For the next OxCalc round, the practical integration implications are:
1. OxCalc can consume stronger typed reject and fence-sensitive consequences from OxFml without redefining their meaning,
2. execution-restriction and capability-sensitive effects are now available as surfaced evaluator/runtime facts rather than hidden scheduler assumptions,
3. replay, retained-witness, and pack-candidate planning should treat OxFml identity, fence, reject, and capability semantics as authoritative,
4. DNA OneCalc planning should stay explicitly downstream of OxFml and separate from OxCalc’s broader coordinator responsibilities,
5. any future multi-session publish-arbitration or graph-wide policy should build on OxFml artifact meaning, not replace it.

## 6. Minimum Invariants
The following invariants remain mandatory from the OxFml side:
1. candidate and commit are distinct artifact stages,
2. reject remains no-publish unless OxFml later declares a different typed path explicitly,
3. fence meaning remains OxFml-owned and replay-preserved,
4. capability-sensitive denials remain typed outcomes, not generic coordinator failures,
5. replay-safe identity categories remain authoritative from OxFml canonical docs,
6. quarantined or explanatory-only witnesses are not pack-eligible,
7. proving-host reductions must not be mistaken for OxCalc coordinator semantics.

## 7. Open OxFml-Side Gaps Still Relevant To OxCalc
The following lanes remain open on the OxFml side:
1. broader async and distributed runtime semantics beyond the current managed local floor,
2. broader topology-fact and publication consequence breadth beyond the currently exercised cases,
3. pack-grade replay promotion and any claim above the current local replay floor,
4. broader formal families beyond the checked local session-lifecycle artifacts,
5. broader Excel semantic breadth outside the current local host/oracle scenario floor.

## 8. Requests For The Next OxCalc Round
The next useful OxCalc-side feedback would be:
1. which surfaced execution-restriction or capability facts are most important for the next scheduler/coordinator integration slice,
2. whether current candidate/commit/reject artifacts are sufficient for the next coordinator-side trace and replay consumption pass,
3. which retained-witness or pack-candidate families are most useful for coordinator-facing validation next,
4. whether there are coordinator-facing publication or topology consequences OxCalc expects but OxFml has not yet surfaced explicitly.

## 9. OxCalc Intake Processed On The OxFml Side
OxCalc's current upstream note at `../OxCalc/docs/upstream/NOTES_FOR_OXFML.md` materially aligns with the current OxFml direction.

The most important intake points now processed on the OxFml side are:
1. OxCalc explicitly accepts the stronger candidate-versus-commit separation and the current minimum typed-schema direction for the active local floor,
2. OxCalc wants `candidate_result_id`, `commit_attempt_id`, `reject_record_id`, and optional fence-snapshot references to remain stable correlation keys in replay-facing families,
3. OxCalc continues to prioritize typed fence mismatch, capability denial, session termination, and execution-restriction effects over generic coordinator failure classes,
4. OxCalc wants dependency additions/removals/reclassifications to remain surfaced evaluator/runtime facts rather than coordinator-inferred policy,
5. OxCalc explicitly agrees that retained-witness and pack-candidate families must preserve direct cell bindings where semantic truth depends on concrete resolution.

Current OxFml reading of the open pressure from that note:
1. keep correlation-key and typed-context stability explicit as replay families widen,
2. keep execution-restriction effects in canonical object families rather than letting them drift into trace-only or prose-only reporting,
3. keep retained-local and reduced-witness families aligned with commit-bundle fact surfaces where publication or dependency meaning depends on them,
4. keep direct-binding-sensitive pack-candidate families explicitly distinct from name-only families once broader rehearsal widens.

## 10. Topic-By-Topic Response To OxCalc Section 6
OxCalc asked for an explicit status read on its proposed alignment topics.
Current OxFml response is:

### 10.1 Identity and fence vocabulary consumption
Status: `already canonical`

Current OxFml reading:
1. stable-id, version-key, fingerprint, and runtime-handle categories are already canonical in `docs/spec/OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`,
2. the most relevant current consumed subset is `formula_stable_id`, `formula_token`, `snapshot_epoch`, `bind_hash`, and `profile_version`,
3. `capability_view_key` is canonical and replay-preserved, but still narrower than a fully locked first-class fence member in every clause.

### 10.2 Candidate-result and commit-bundle consequence shape
Status: `already canonical`

Current OxFml reading:
1. candidate-result versus committed publication is already canonical,
2. `value_delta`, `shape_delta`, `topology_delta`, optional `format_delta`, optional `display_delta`, spill events, and surfaced evaluator facts are already canonical seam categories,
3. replay families are expected to preserve this separation rather than collapsing them into one generic publication summary.

### 10.3 Dependency consequence taxonomy
Status: `canonical but narrower`

Current OxFml reading:
1. dependency additions/removals/reclassifications are intended to remain evaluator/runtime facts,
2. those facts already belong in topology/effect surfaces rather than coordinator-inferred policy,
3. the exact explicit reduced-witness and retained-witness projection rules for every dependency subfamily are still narrower than a full closed taxonomy.

### 10.4 Host-query and direct-binding-sensitive truth
Status: `already canonical`

Current OxFml reading:
1. typed host-query capability views are canonical,
2. direct-cell-binding-sensitive truth is already canonical in the proving-host and empirical-pack planning docs,
3. retained and pack-candidate families are expected to preserve direct cell bindings where semantic correctness depends on them.

### 10.5 Semantic-display boundary
Status: `canonical but narrower`

Current OxFml reading:
1. `format_delta` and `display_delta` are already distinct canonical bundle categories,
2. OxFml agrees this boundary still needs narrower shared reading before broader retained/pack-candidate widening,
3. this is a good note-exchange topic but not yet a new handoff trigger by itself.

## 11. Responses To OxCalc Section 9 Questions
OxCalc asked whether the current floor is stable enough to consume in a few specific places.

### 11.1 Execution-restriction effects
Current OxFml answer:
1. consume them now as canonical surfaced evaluator/runtime facts,
2. do not assume one final frozen single-object carrier yet,
3. treat the current floor as stable enough to consume semantically, but still narrower than a final transport lock.

### 11.2 Dependency additions/removals/reclassifications in replay-facing families
Current OxFml answer:
1. yes in semantic intent,
2. they are expected to remain evaluator/runtime facts rather than coordinator inference,
3. exact retained/reduced family projection closure is still narrower than a universal frozen rule.

### 11.3 `commit_attempt_id` and optional fence snapshot refs across retained/reduced families
Current OxFml answer:
1. `commit_attempt_id` should be treated as stable enough to consume now,
2. optional fence snapshot refs should be treated as stable where present,
3. optionality and exact projection breadth remain open rather than universally guaranteed.

### 11.4 Distinguishing direct-binding-sensitive pack-candidate families
Current OxFml answer:
1. OxFml intends to preserve this distinction,
2. current proving-host and empirical-pack planning docs already require direct cell bindings where semantic truth depends on them,
3. the exact broader naming/indexing convention for those families remains open.

### 11.5 Consuming a more explicit identity-category subset now
Current OxFml answer:
1. yes for `formula_stable_id`, `formula_token`, `snapshot_epoch`, `bind_hash`, and `profile_version`,
2. treat `capability_view_key` as important consumed compatibility state now,
3. but still read it as canonical-but-narrower rather than fully locked as a first-class fence member in every clause.

### 11.6 Separate note on semantic-format versus display-facing consequences
Current OxFml answer:
1. yes, this is a good next note-exchange topic,
2. no, it does not yet require a formal handoff packet by itself,
3. OxFml currently sees it as a clarifying seam-reading topic rather than a seam-shape change.

## 12. OxCalc Intake Seen Through The Current OxFunc Refinement
The current OxFunc refinement matters to OxCalc, but mostly indirectly.

Current OxFml reading:
1. library-context snapshot work is primarily an OxFml/OxFunc seam topic and does not yet change coordinator-facing seam meaning,
2. availability/feature/provider taxonomy is also primarily an OxFml/OxFunc semantic-boundary topic, but it may later affect typed reject or execution-fact breadth where runtime provider failure becomes coordinator-visible,
3. callable-value carrier work is primarily an OxFml/OxFunc semantic topic today, but publication restrictions on callable values could become coordinator-relevant later if callable publication paths widen,
4. the next OxFml/OxFunc narrowing round is likely to focus specifically on the `LET` / `LAMBDA` callable seam, but that still remains upstream-semantic at the current stage,
5. operator/literal/value-universe refinement remains upstream semantic-boundary work and does not currently require an OxCalc-facing seam change,
6. the latest OxFunc round closure position does not add a new coordinator-facing pressure point; it mostly confirms that current OxFml canonical seam docs are the right active baseline until a narrower trigger appears.

Working rule from this combined read:
1. do not prematurely project OxFunc transport narrowing into OxCalc coordinator assumptions,
2. do keep watching the availability/provider-failure and callable-publication lanes because they are the most likely to become coordinator-visible later.

## 13. What This Note Does Not Authorize
This note should not be read as authorizing:
1. coordinator-side redefinition of candidate, commit, reject, fence, or capability semantics,
2. pack-grade replay claims or any claim above the current local `cap.C3.explain_valid` floor,
3. collapse of DNA OneCalc proving-host policy into OxCalc coordinator policy,
4. formula, bind, fence, or capability-view rewrites as replay-safe transforms,
5. closure of any existing handoff packet.

## 14. Current Cross-Repo Status Reminder
`HANDOFF-FML-001` remains filed and not yet acknowledged in the local register.
This note does not treat that handoff as closed.

`HANDOFF-CALC-001` remains incorporated on the OxFml side through the current canonical seam docs and this note does not replace that earlier acknowledgment path.

OxCalc's new upstream note is treated as an observation ledger input, not as canonical OxFml seam text.

## 15. Current OxFml Position On Follow-Up
No new formal handoff is being filed from this intake pass.

Current OxFml reading:
1. most of OxCalc's current pressure is answerable from existing canonical docs plus note-level clarification,
2. the two most likely future formal-handoff triggers remain:
   - narrower execution-restriction fact consumption,
   - narrower publication/topology consequence breadth,
3. the most likely cross-lane trigger from the OxFunc refinement would be availability/provider-failure handling if it starts changing coordinator-visible reject or publication consequences,
4. the latest OxFunc round-closure posture reinforces that no new OxCalc-facing packet is warranted until one of those narrower triggers actually materializes.

## 16. Working Rule
Until the open lanes narrow further:
1. treat OxFml seam docs and replay docs as the source of truth for evaluator/runtime/replay artifact meaning,
2. treat current local runtime and replay evidence as a stronger floor than before, but still local rather than pack-grade,
3. file a formal handoff only when a coordinator-facing seam clause changes, not for routine observation exchange.

## 16A. Current Read On OxCalc W026 Topic-Matrix Framing
OxCalc's newer TreeCalc-facing topic-matrix framing makes sense for the coordinator-facing seam and is the right next shape for the note exchange.

Current OxFml read:
1. formula and bind artifact identity carriage: `already canonical` for the first TreeCalc subset,
2. direct and relative reference descriptor carriage: `canonical but narrower`,
3. unresolved and host-sensitive reference carrier rules: `canonical but narrower`,
4. dependency fact carriage for additions, removals, and reclassifications: `already canonical` semantically, but still narrower in some retained/reduced witness projections,
5. candidate-result consequence optionality and correlation guarantees: `already canonical`,
6. reject-context carrier and diagnostic guarantees: `already canonical` for the current typed families,
7. runtime-derived effect and execution-restriction transport: `canonical but narrower`,
8. direct-binding-sensitive witness preservation rules: `already canonical`,
9. semantic-format versus display-facing consequence boundary: `canonical but narrower`.

The most important caution points for OxCalc are:
1. do not over-read relative-reference closure as fully locked just because first TreeCalc direct-reference carriage is likely ready sooner,
2. do not over-read execution-restriction transport as one final frozen single-object carrier,
3. do not over-read semantic-format versus display-facing consequences as fully settled beyond the current narrower seam-reading floor.

Current cross-lane reading:
1. this W026 topic-matrix framing still fits the OxFml/OxCalc seam cleanly,
2. it does not create a new formal handoff trigger by itself,
3. it remains compatible with the current OxFml/OxFunc refinement because the latest OxFunc pressure is still upstream-semantic rather than coordinator-shaping.

## 16B. Current Reply Shape For OxCalc's Narrower W026 Pass
OxCalc has now narrowed the W026-focused questions to four remaining topics. Current OxFml response is:

### 16B.1 Topic A: Relative-reference descriptor carriage
Classification: `canonical but narrower`

Carrier surface OxCalc should consume now:
1. current normalized reference-expression and bound-reference artifacts,
2. current bind identity and structure-context identity carried through the formula/bind package,
3. the current direct absolute and first relative-reference subset only where the bound artifact already preserves the needed contextual dependence honestly.

Explicit non-assumptions:
1. do not assume the full future relative-reference universe is already closed,
2. do not assume every relative form is fully frozen just because a first direct subset is consumable,
3. do not assume all contextual relative meaning has been reduced to one final transport shape.

W026 sufficiency now:
1. yes for a narrowed first subset,
2. no for broad relative-reference closure.

Current handoff read:
1. note-level topic for now,
2. becomes a narrower handoff only if the first TreeCalc subset exposes a concrete insufficiency in the carried descriptor floor.

### 16B.2 Topic B: Unresolved and host-sensitive reference carriers
Classification: `canonical but narrower`

Carrier surface OxCalc should consume now:
1. current accepted-unresolved versus reject distinction,
2. current typed reject contexts and unresolved/bind diagnostics,
3. current host-query capability-view and host-sensitive evaluator fact surfaces where resolution depends on concrete host truth.

Explicit non-assumptions:
1. do not collapse unresolved-at-bind, unresolved-at-evaluate, and host-sensitive-but-resolvable-only-with-concrete-host-truth into one generic resolution failure bucket,
2. do not assume one final single carrier family already covers every unresolved/reference-sensitive case.

W026 sufficiency now:
1. yes if the first in-scope unresolved and host-sensitive families remain explicitly named,
2. broader host/reference closure remains deferred.

Current handoff read:
1. note-level topic for now.

### 16B.3 Topic C: Runtime-derived effects and execution-restriction transport
Classification: `canonical but narrower`

Carrier surface OxCalc should consume now:
1. current candidate-result and commit-bundle surfaced evaluator facts,
2. current topology/effect fact refs,
3. current capability-sensitive and execution-restriction observations as surfaced evaluator/runtime facts.

Explicit non-assumptions:
1. do not assume one final frozen single-object transport carrier yet,
2. do not collapse execution-restriction observations into scheduler policy,
3. do not collapse capability-sensitive and execution-restriction-sensitive observations unless OxFml later locks that transport explicitly.

W026 sufficiency now:
1. yes semantically,
2. final transport-carrier closure remains narrower and may become a future handoff trigger if live TreeCalc evidence shows insufficiency.

Current handoff read:
1. still note-level today,
2. one of the more likely future narrow handoff triggers if pressure increases.

### 16B.4 Topic D: Semantic-format versus display-facing boundary
Classification: `canonical but narrower`

Carrier surface OxCalc should consume now:
1. `format_delta` and `display_delta` as distinct canonical categories,
2. current format-sensitive evaluator facts where semantic correctness or replay truth depends on them.

Explicit non-assumptions:
1. do not treat broader display-facing closure as already promised,
2. do not collapse `display_delta` into `format_delta`,
3. do not assume all display-facing categories are publication-critical in the first TreeCalc subset.

W026 sufficiency now:
1. yes for a semantics-first first phase,
2. broader display-facing closure remains deferred.

Current handoff read:
1. note-level topic for now,
2. does not yet justify a new formal handoff by itself.

## 17. New Bounded Round Proposal: Immutable Edit And Validated Completion Packet
OxFml now has a first local editor/language-service floor under `W048`.

Current OxFml-local capabilities:
1. immutable formula-edit request/result handling,
2. explicit smallest text-change-range reporting,
3. subtree-reuse summaries over incremental parse/red/bind,
4. unified diagnostics snapshots,
5. deterministic completion proposals,
6. validation and application of completion proposals through the ordinary parse/bind path.

Current OxFml reading:
1. the remaining host/editor lane is now mainly a packet-freeze question, not a formula-semantics uncertainty,
2. OxFml should not own the containing immutable workbook/document tree,
3. OxFml should return replacement-ready immutable formula artifacts and diagnostics,
4. direct host and OxCalc-integrated host should ideally consume the same packet family.

Current OxFml best-effort proposal:
1. split the first shared editor packet into:
   - immutable edit request,
   - immutable edit result,
   - validated completion application result,
2. keep larger document-spine replacement explicitly host/coordinator-owned.

Proposed immutable edit request:
1. `formula_stable_id`
2. `previous_formula_token`
3. `previous_green_tree_key`
4. `new_formula_text`
5. optional `text_change_range`
6. `formula_channel_kind`
7. `structure_context_version`
8. explicit bind-visible context summary:
   - visible names
   - visible tables
   - caller anchor when already part of the formula slot
9. requested follow-on stage

Proposed immutable edit result:
1. `new_formula_token`
2. `green_tree_key`
3. `text_change_range`
4. subtree reuse summary
5. diagnostics snapshot
6. optional `bind_hash`
7. optional `semantic_plan_key`

Proposed validated completion application result:
1. `proposal_id`
2. applied replacement span
3. updated immutable edit result
4. explicit rule that host/coordinator still owns acceptance plus containing-spine replacement

Current OxFml ask for the next bounded OxCalc round:
1. is this the right first shared packet for direct host and OxCalc-integrated host,
2. should validated completion application remain host-local or become coordinator-visible,
3. are any additional identity or acknowledgment fields needed before the packet is useful in TreeCalc-facing implementation work,
4. does OxCalc want the same packet reused for cell formulas, host-managed defined-name formulas, and later other formula-bearing slots.

Current OxFml working rule:
1. this lane should reopen only as a bounded immutable-edit packet round,
2. it should not reopen broader host/runtime clarification that is already converged for the first implementation slice.

## 18. New Bounded Round Proposal: Fixture Host And Coordinator Stand-In Packet
OxFml now has two new OxFunc-facing work lanes:
1. `W049` OxFunc preparation adapter and consumer harness
2. `W050` OxFunc snapshot-pinned seam fixture families

Those lanes are nominally OxFunc-facing, but some of their required fixture inputs are actually stand-ins for host/coordinator-owned truths.

Current OxFml read is:
1. this does not reopen the already-converged first-slice host/runtime packet,
2. it does create a narrower OxCalc-facing question:
   - what is the right first deterministic stand-in host/coordinator packet for integration artifacts,
   - without pretending we have frozen the production OxCalc coordinator API.

Current OxFml best-effort proposal is now documented in:
1. `docs/spec/OXFML_FIXTURE_HOST_AND_COORDINATOR_STANDIN_PACKET.md`

Current proposed first packet families are:
1. formula slot facts:
   - `formula_text`
   - `formula_channel`
   - `caller_anchor`
   - optional `active_selection_anchor`
   - structure-context identity,
2. binding-world facts:
   - `cell_fixture`
   - optional defined-name bindings
   - optional table packet inputs,
3. typed host/query facts:
   - optional `ReferenceResolver`
   - optional `HostInfoProvider`
   - optional `RtdProvider`
   - optional `RegisteredExternalProvider`
   - `LocaleFormatContext`
   - scalar context such as `now_serial`, `random_value`, and date-system identity,
4. runtime catalog facts:
   - pinned `library_context_snapshot_ref`
   - local or pinned `LibraryContextProvider`.

Current ownership rule OxFml wants OxCalc to review:
1. the fixture harness may stand in for host/coordinator-owned truths locally,
2. but those truths must still be modeled as host/coordinator-supplied inputs rather than evaluator-owned meaning.

Current OxFml questions for the next bounded OxCalc round are:
1. is this the right first stand-in packet for coordinator-owned truths in deterministic integration artifacts,
2. should `RegisteredExternalProvider` stay present in the stand-in packet from the start even though the first OxFunc wave still defers worksheet `CALL` / `REGISTER.ID`,
3. should candidate/commit/reject packet capture remain a separate projection layer rather than part of the stand-in input packet,
4. does OxCalc want any additional identity or acknowledgment fields before this packet is useful for later TreeCalc-facing integration tests.

Current OxFml working rule:
1. this lane should stay bounded to stand-in fixture-host reuse,
2. it should not be read as a request to freeze the production OxCalc coordinator API early.

## 19. Current Intake Of OxCalc's Fixture-Host Response
OxCalc has now processed the stand-in packet proposal and OxFml reads that pass as convergent.

Current OxFml intake is:
1. OxCalc agrees the stand-in fixture-host packet is the right bounded next packet for deterministic integration artifacts,
2. OxCalc agrees those packet inputs must remain modeled as host/coordinator-supplied truths rather than evaluator-owned meaning,
3. OxCalc agrees this packet should not be read as freezing the production coordinator API,
4. OxCalc agrees `RegisteredExternalProvider` may remain present as an optional stand-in field from the start even while worksheet `CALL` / `REGISTER.ID` stay deferred,
5. OxCalc agrees candidate/commit/reject capture should remain a separate projection layer rather than being folded into the stand-in input packet.

Current useful additions from OxCalc are now accepted by OxFml as good first packet refinements:
1. a stand-in packet identity or fixture-input identity,
2. explicit structure-context identity,
3. explicit formula-slot identity when the packet is reused across multiple slot families.

Current OxFml reply back to OxCalc:
1. yes, OxFml accepts those three identity refinements as the right next packet sharpening,
2. yes, OxFml will keep candidate/commit/reject capture as a separate projection layer,
3. yes, OxFml will keep `RegisteredExternalProvider` optional rather than treating its presence as first-wave execution scope,
4. no, OxFml does not read this confirmation as shared coordinator-API freeze.

Current working rule after this pass:
1. `W051` should now move from draft-only packet proposal toward a narrower packet-freeze/readiness owner,
2. the next OxCalc round on this topic should be mismatch-driven or implementation-driven,
3. broader host/runtime, execution-restriction, and publication/topology watch lanes remain separate.

## 20. Current W052 Read Relevant To OxCalc
OxFml has now exercised the first local registered-external packet family under `W052`.

Current OxFml read that matters to OxCalc:
1. this does not freeze the broader production coordinator API,
2. it does make the first stand-in and host-facing packet sharper for later coordinator use,
3. built-in function and operator catalog truth remains OxFunc-owned,
4. runtime registered-external catalog truth also remains OxFunc-owned,
5. host- or coordinator-initiated registration channels should therefore be modeled as typed mutation requests funneled into OxFunc-owned catalog mutation rather than as local coordinator-owned catalog edits.

Current local initiating channels now carried explicitly are:
1. `WorksheetRegisterId`
2. `HostApiRegistration`
3. `VbaProjectShimRegistration`

Current local host-facing packet additions are:
1. `RegisteredExternalCatalogMutationRequest::{ Register, Unregister }`
2. `RegisteredExternalCatalogMutationResult::{ RegisterApplied, UnregisterApplied }`
3. `RegisteredExternalCatalogController`

Current OxFml reading for OxCalc:
1. the first stand-in packet should keep `RegisteredExternalProvider` optional,
2. if later TreeCalc-facing integration wants host-side function registration, it should preserve the initiating channel and stable registration identity,
3. OxCalc should not infer from this packet that it owns the function catalog itself,
4. later snapshot-generation and coordinator acknowledgment consequences from register/unregister remain narrower than the current first packet and are not yet being frozen here.

## 21. Current Intake Of OxCalc's Latest Structured-Reference, Immutable-Edit, Stand-In, And W052 Note
OxCalc has now replied on the newer bounded host/coordinator lanes and OxFml reads that pass as materially convergent.

Current OxFml intake is:
1. OxCalc confirms `table_catalog + enclosing_table_ref + caller_table_region` is the right first semantic structured-reference packet,
2. OxCalc confirms totals, header, data, and current-row-sensitive region identity should remain explicit in that packet even while broader workbook-table closure remains deferred,
3. OxCalc confirms the `W048` immutable-edit split into immutable edit request, immutable edit result, and validated completion application result is the right first packetization,
4. OxCalc confirms containing-spine replacement and validated-completion acceptance remain host/coordinator-owned,
5. OxCalc confirms the `W051` stand-in packet is settled enough for deterministic fixture-host and TreeCalc-facing integration reuse on the current first wave,
6. OxCalc confirms the sharpened `W052` reading that host- or coordinator-initiated registration should remain typed mutation requests funneled into OxFunc-owned catalog truth,
7. OxCalc confirms that registration-channel identity and stable registration identity should remain preserved where later TreeCalc-facing integration uses that lane,
8. OxCalc continues to treat snapshot-generation and coordinator acknowledgment consequences from register/unregister as narrower than the current first packet and not yet frozen.

Current OxFml working read after this pass:
1. no new broad host/coordinator seam ambiguity was introduced,
2. `W036`, `W048`, `W051`, and `W052` are all directionally aligned enough to keep moving on local implementation and narrower packet refinement,
3. the remaining OxCalc-facing pressure is now concentrated in the already-known `W026` residual topics rather than in the newer packet families themselves.

## 21A. Current Narrowed W052 Read
After the latest OxFunc and OxFml narrowing, OxFml now reads `W052` as OxFunc-aligned at note level on:
1. exact shared field naming for the adopted direct packet set:
   - `RegisterIdRequest { library_name, procedure, declared_type_text }`
   - `RegisteredExternalDescriptor { stable_registration_id, register_id, origin_kind, display_name, library_name, procedure, declared_type_text }`
   - `RegisteredExternalCallRequest { target, invocation_args }`
   - `RegisteredExternalTarget::{ RegisterId, Direct }`
2. the minimum final shared `RegisteredExternalDescriptor` field set:
   - the current seven-field descriptor
3. mutation/controller family ownership:
   - `RegisteredExternalCatalogMutation*` and `RegisteredExternalCatalogController` remain OxFml-owned host/coordinator funnel packets for the current phase
4. the minimum shared snapshot-generation and invalidation consequences:
   - bind-visible registration or unregister => new `LibraryContextSnapshot` generation plus bind invalidation where the visible function or name world changes
   - `CALL` / `REGISTER.ID`-only descriptor mutation => targeted reevaluation by default

Current non-open points from the OxFml side are:
1. `RegisteredExternalProvider` remains separate from `HostInfoProvider`
2. host- or coordinator-initiated registration remains typed mutation requests funneled into OxFunc-owned catalog truth
3. direct adoption of OxFunc-owned `RegisterIdRequest`, `RegisteredExternalDescriptor`, and `RegisteredExternalCallRequest` is the settled current direction
4. registration-channel identity and stable registration identity should remain preserved where later TreeCalc-facing integration uses that lane

Current OxCalc-specific read now needed is narrower:
1. whether OxCalc is content to align with the same direct-packet field names, seven-field descriptor, and current-phase funnel-family ownership
2. whether the current snapshot-generation and invalidation split is sufficient for first TreeCalc-facing planning

Current processing note:
1. the latest inbound OxCalc note now explicitly answers this sharper packet and aligns with the same direct-packet, seven-field-descriptor, current-phase funnel-family, and snapshot-consequence read
2. so this `21A` section is no longer awaiting first acknowledgment from OxCalc; it now records the converged note-level packet
3. no broader `W052` seam expansion is requested from OxCalc at this stage; only promotion from note-level convergence to shared seam-freeze text or concrete mismatch if later implementation evidence forces one

## 22. OxFml Read On OxCalc's New Residual W026 Round
OxCalc has now packetized the remaining TreeCalc-facing note-level residuals into three narrower sequences:
1. caller-anchor and address-mode carriage,
2. execution-restriction transport breadth,
3. publication and topology consequence breadth.

Current OxFml read is:
1. this is the right next note shape,
2. it remains note-level rather than a current handoff trigger,
3. the current OxFml answers remain:
   - caller-anchor and address-mode carriage: `canonical but narrower`
   - execution-restriction transport breadth: `canonical but narrower`
   - publication and topology consequence breadth: `canonical but narrower`

Current consumed-now carriers OxCalc should continue to rely on are:
1. for caller-anchor and address-mode carriage:
   - `FormulaSourceRecord`
   - `caller_anchor`
   - formula-channel and address-mode context
   - structure-context identity
   - the current bound-reference and normalized-reference families only where the current first subset already preserves contextual dependence honestly,
2. for execution-restriction transport breadth:
   - current candidate-result and commit-bundle surfaced evaluator facts
   - current topology/effect fact refs
   - current typed capability-sensitive and execution-restriction-sensitive observations,
3. for publication and topology consequence breadth:
   - `value_delta`
   - `shape_delta`
   - `topology_delta`
   - optional `format_delta`
   - optional `display_delta`
   - current spill and dependency-sensitive surfaced evaluator/runtime fact refs where present.

Current explicit non-assumptions OxFml still wants OxCalc to preserve are:
1. do not treat caller-anchor and address-mode carriage as full relative-reference closure,
2. do not treat execution-restriction transport as one final frozen single-object carrier,
3. do not treat currently optional consequence families as universally mandatory across the broader future TreeCalc scope,
4. do not treat the current local publication and topology breadth as equal to the full eventual coordinator publication universe.

Current working rule after this residual round:
1. continue TreeCalc intake work on the current converged host/runtime, table-context, immutable-edit, stand-in, and registered-external packet floor,
2. keep the three residual sequences as note-level narrowing lanes under `W026`,
3. promote a narrower handoff only if live TreeCalc evidence exposes a concrete insufficiency in one of those carried families.

## 23. Current Public Surface Update
After the latest Wave 4 packaging cut, OxFml's current public consumer read is:
1. ordinary downstream use should target:
   - `consumer::runtime`
   - `consumer::editor`
   - `consumer::replay`
2. public `substrate::...` access is gone from the library surface,
3. any remaining host/session/adapter reach that still exists is explicit
   `test_support` support surface for bounded internal or integration-test use,
   not ordinary downstream integration contract.
