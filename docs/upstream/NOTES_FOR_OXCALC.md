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
