# FEC/F3E Formal and Assurance Map

## 1. Purpose
This document is the canonical OxFml assurance map for the FEC/F3E seam.

It makes explicit:
1. which live documents define the seam contract,
2. how the conformance matrix references those documents,
3. which seam clause families are expected to gain replay, Lean, and TLA+ coverage,
4. which assurance lanes remain open.

This is a bootstrap document.
It is not a dated execution report or a transition note.

## 2. Canonical Source Documents
The live FEC/F3E seam is defined by:
1. `FEC_F3E_DESIGN_SPEC.md`
2. `FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md`
3. `FEC_F3E_TESTING_AND_REPLAY.md`
4. `FEC_F3E_SCHEMA_REPLAY_FIXTURE_PLAN.md`
5. `../OXFML_REPLAY_APPLIANCE_ADAPTER_V1.md`
6. `../OXFML_REPLAY_ADAPTER_CAPABILITY_MANIFEST_V1.json`
7. `../OXFML_SYSTEM_DESIGN.md`
8. `../OXFML_IMPLEMENTATION_SURFACES_AND_STATE_OPTIONS.md`
9. `../OXFML_IMPLEMENTATION_BASELINE.md`
10. `../OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`
11. `../OXFML_CANONICAL_ARTIFACT_SHAPES.md`
12. `../OXFML_MINIMUM_SEAM_SCHEMAS.md`
13. `../OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md`
14. `../OXFML_FORMALIZATION_AND_VERIFICATION.md`
15. `../OXFML_FORMAL_ARTIFACT_REGISTER.md`
16. `../formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
17. `../OXFML_DNA_ONECALC_HOST_POLICY_BASELINE.md`
18. `../OXFML_EMPIRICAL_PACK_PLANNING.md`

Archive material may support evidence work later, but it is not bootstrap authority.

## 3. Conformance Matrix Document Identifiers
The FEC/F3E conformance matrix uses these document evidence identifiers:

| evidence_id | meaning |
|---|---|
| `DOC-FEC-DESIGN` | Canonical seam contract in `FEC_F3E_DESIGN_SPEC.md`. |
| `DOC-FEC-ASSURANCE` | Canonical seam assurance map in this document. |
| `DOC-FEC-TEST` | Testing, replay, and pack strategy in `FEC_F3E_TESTING_AND_REPLAY.md`. |
| `DOC-FEC-FIXTURE-PLAN` | Schema replay fixture plan in `FEC_F3E_SCHEMA_REPLAY_FIXTURE_PLAN.md`. |
| `DOC-OXFML-REPLAY-ADAPTER` | OxFml-local replay adapter rollout contract in `../OXFML_REPLAY_APPLIANCE_ADAPTER_V1.md`. |
| `DOC-OXFML-REPLAY-MANIFEST` | OxFml adapter capability manifest in `../OXFML_REPLAY_ADAPTER_CAPABILITY_MANIFEST_V1.json`. |
| `DOC-OXFML-SYSTEM` | OxFml-wide ownership and subsystem boundaries in `../OXFML_SYSTEM_DESIGN.md`. |
| `DOC-OXFML-OPTIONS` | OxFml implementation-shape constraints in `../OXFML_IMPLEMENTATION_SURFACES_AND_STATE_OPTIONS.md`. |
| `DOC-OXFML-BASELINE` | OxFml code-start implementation baseline in `../OXFML_IMPLEMENTATION_BASELINE.md`. |
| `DOC-OXFML-IDS` | OxFml identity/version vocabulary in `../OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`. |
| `DOC-OXFML-SHAPES` | OxFml canonical artifact field surfaces in `../OXFML_CANONICAL_ARTIFACT_SHAPES.md`. |
| `DOC-OXFML-SCHEMAS` | OxFml minimum schema objects for seam payload families in `../OXFML_MINIMUM_SEAM_SCHEMAS.md`. |
| `DOC-OXFML-TAXONOMY` | OxFml taxonomy layer for deltas, facts, rejects, and traces in `../OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md`. |
| `DOC-OXFML-FORMAL` | OxFml-wide formalization posture in `../OXFML_FORMALIZATION_AND_VERIFICATION.md`. |
| `DOC-OXFML-FORMAL-REGISTER` | OxFml formal artifact register in `../OXFML_FORMAL_ARTIFACT_REGISTER.md`. |
| `DOC-OXFML-OXFUNC` | OxFml to OxFunc semantic boundary in `../formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`. |
| `DOC-OXFML-DNA-HOST` | DNA OneCalc host-policy baseline in `../OXFML_DNA_ONECALC_HOST_POLICY_BASELINE.md`. |
| `DOC-OXFML-EMP-PACK` | Empirical-pack planning baseline in `../OXFML_EMPIRICAL_PACK_PLANNING.md`. |

These identifiers mean "specified by the current live spec set".
They are not claims that replay or formal artifacts already exist.

## 4. Assurance Coupling Rule
Every important FEC/F3E seam clause should map to:
1. a canonical prose clause,
2. a conformance matrix row,
3. a replay or scenario-pack obligation,
4. a Lean-friendly typed surface, a TLA+ property, or both when appropriate.

If one of these is missing, the gap remains open and must stay visible in status reporting.

## 5. Clause Families and Expected Witnesses
| clause_family | primary seam focus | replay expectation | Lean expectation | TLA+ expectation |
|---|---|---|---|---|
| Session lifecycle | `prepare -> open_session -> capability_view -> execute -> commit` | deterministic phase traces and accept/reject cases | typed session-state ADTs and transition admissibility | lifecycle state machine and legal transition invariants |
| Fences and identity | session identity, token/epoch/bind/profile fences | reject-on-mismatch replay corpus | typed fence tuple and mismatch classification | stale-commit exclusion and publish safety |
| Candidate/publication boundary | accepted candidate result distinct from published bundle | candidate-vs-published replay corpus | candidate/bundle relation invariants | accept/reject/publication separation |
| Atomic commit bundle | one publishable bundle with typed deltas | accept/reject bundle witness packs | bundle-shape invariants | atomic publish/no-publish split |
| Minimum payload schemas | minimum field sets for candidate, commit, reject, and trace payload families | schema-validation replay packs | ADT field-preservation invariants | payload sufficiency for accept/reject/publication outcomes |
| Reject taxonomy | typed non-publishing failures | reject-detail replay pack | reject-code families and no-publish-on-reject theorem surface | reject transitions and abort cleanup |
| Overlay lifecycle | dynamic refs, spill, format overlays | overlay creation/reuse/eviction scenarios | overlay token and delta typing | visibility, pinning, and epoch-safe eviction |
| Spill event semantics | takeover, clearance, blocked | spill event replay bundles | event payload typing | interaction with concurrent sessions and retries |
| Runtime-derived effect surfacing | coordinator-relevant evaluator facts and derived effects | effect-report replay packs including async-coupled external-provider lanes | fact/delta typing | coordinator-visible consequences under concurrency |
| OxFunc preparation boundary | prepared args/results and caller context | prepared-call trace packs | prepared-call ADTs and invariants | not primary unless concurrency affects evaluation context |
| Host-mode compatibility | OxCalc-integrated vs DNA OneCalc reduced profile | reduced-profile acceptance packs | profile-gated contract surfaces | reduced-profile state-space constraints |

## 6. Initial Pack Alignment
The initial pack families expected to witness the seam are:
1. `PACK.fec.transaction_boundary`
2. `PACK.fec.commit_atomicity`
3. `PACK.fec.reject_detail_replay`
4. `PACK.fec.overlay_lifecycle`
5. `PACK.fec.format_dependency_tokens`
6. `PACK.oxfml.oxfunc.prepared_contract`
7. `PACK.fec.minimum_payload_schemas`

These pack names describe intended witness families.
They are not evidence of exercised packs yet.

## 7. Adapter Capability And Replay Appliance Evidence
OxFml replay rollout claims must be backed by explicit adapter evidence.

Current evidence targets:
1. `cap.C0.ingest_valid`
   - proving artifacts should include source fixture import and normalized bundle-validation evidence
2. `cap.C1.replay_valid`
   - proving artifacts should include deterministic replay rerun for supported fixture families plus explicit unsupported-state surfacing
3. `cap.C2.diff_valid`
   - proving artifacts should include typed mismatch-family evidence over candidate, commit, reject, and effect surfaces
4. `cap.C3.explain_valid`
   - proving artifacts should include why-rejected or why-not-published explanation evidence with source refs
5. `cap.C4.distill_valid`
   - remains scaffolded only until OxFml has broader retained-local witness breadth, at least one irreducibility or unsupported case, and stronger promotion-grade governance over those retained sets
6. `cap.C5.pack_valid`
   - remains out of scope in this pass

Current rollout target:
1. OxFml claims through `cap.C3.explain_valid`,
2. OxFml scaffolds but does not claim `cap.C4.distill_valid`,
3. OxFml does not claim `cap.C5.pack_valid`.

## 8. Witness Lifecycle And Quarantine Assurance
Witness lifecycle state affects assurance claims whenever replay outputs are promoted beyond local witness tier.

Current rules:
1. explanatory-only witnesses may support local understanding but not pack-facing assurance,
2. quarantined witnesses may support triage but not promotion claims,
3. retained or promoted witness claims require explicit lifecycle refs and resolved capability preconditions,
4. lifecycle governance is additive and does not change OxFml semantic truth.

## 9. Open Assurance Lanes
The following remain explicitly open:
1. the current replay corpus is still local and not yet promoted into pack-grade seam artifacts,
2. the first local Lean and TLA+ session lifecycle artifacts are now checked locally, but broader formal families remain unproved,
3. no TLA+ model has yet been authored for concurrent evaluator sessions or publish fences beyond the checked local sequential lifecycle model,
4. recorded OxCalc-facing seam handoffs remain open for ad hoc future coordination where coordinator-facing clauses materially change,
5. minimum provenance vocabulary for prepared-call and prepared-result structures is still being tightened with the OxFunc boundary,
6. timeout, abort, and overlay-cleanup closure remains open before Stage 2 promotion,
7. current adapter capability claims still rely on local witness-tier evidence rather than pack-grade corpus,
8. witness lifecycle and quarantine governance are now specified and broadened local reduced-witness coverage exists across host and oracle families, but pack-facing promotion does not exist yet,
9. local normalized pack-candidate bundles now exist as rehearsal evidence, but remain intentionally non-pack-eligible,
10. DNA OneCalc host-policy and empirical-pack planning are now explicit, but they remain planning-only and do not imply host or pack maturity.

Current checked local formal floor also includes:
1. `formal/lean/OxFmlExternalReferenceDeferred.lean`
   - external-provider admissibility plus async-capability consequence lemmas
2. `formal/tla/FecExternalCapabilityGate.tla`
   - external-provider gate with checked async-consequence invariant
3. `formal/run_formal.ps1`
   - canonical local runner for those checked artifacts

## 10. Working Rule
Use the live design and assurance docs for bootstrap and implementation planning.
Use archive materials only for migration history or later evidence work.
