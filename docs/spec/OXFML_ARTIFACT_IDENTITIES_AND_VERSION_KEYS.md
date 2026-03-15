# OxFml Artifact Identities and Version Keys

## 1. Purpose
This document defines the current OxFml vocabulary for artifact identity, versioning, fingerprints, and runtime handles.

The immediate goal is not to lock exact encodings.
The goal is to prevent semantic drift by making the categories explicit before implementation work begins.

This document should be read together with:
1. `OXFML_SYSTEM_DESIGN.md`
2. `OXFML_IMPLEMENTATION_SURFACES_AND_STATE_OPTIONS.md`
3. `formula-language/OXFML_FORMULA_ENGINE_ARCHITECTURE.md`
4. `fec-f3e/FEC_F3E_DESIGN_SPEC.md`

## 2. Working Rule
OxFml should not overload one token to mean all of:
1. logical identity,
2. version identity,
3. content fingerprint,
4. runtime residency handle,
5. fence eligibility.

These are different categories and should stay different even when some implementations encode them compactly.

## 3. Identity Categories
The canonical categories are:

1. stable logical identity
   - "which logical formula/artifact is this?"
2. version key
   - "which declared revision of that artifact is this?"
3. content fingerprint
   - "what exact semantic or syntactic payload does this artifact currently have?"
4. runtime handle
   - "where is a currently resident copy of this artifact stored?"
5. fence tuple member
   - "which keys must match before an operational step may publish?"

Not every artifact family needs all five categories, but the distinction remains mandatory.

## 4. Formula-Level Vocabulary
### 4.1 `formula_stable_id`
`formula_stable_id` is the stable logical identity of a formula-bearing locus as exposed to OxFml.

Working meaning:
1. it identifies the logical formula slot under the host/coordinator model,
2. it survives ordinary formula-text edits,
3. it may survive some structure edits if the enclosing host preserves locus identity,
4. it is not a content hash.

Open detail:
1. whether this identity is cell-based, node-based, or host-object-based depends on the enclosing host model.

### 4.2 `formula_text_version`
`formula_text_version` is the declared version key for entered/stored formula text associated with a `formula_stable_id`.

Working meaning:
1. it changes when the formula source text changes,
2. it does not need to change when only workbook structure changes,
3. it is distinct from any content hash or parse-tree key.

### 4.3 `formula_token`
`formula_token` is the evaluator-facing fence token for the current formula payload.

Working meaning:
1. it is the token used in FEC/F3E fence checks,
2. it must change whenever a publish-relevant formula payload change would invalidate a prepared evaluation,
3. it may be derived from one or more lower-level version keys,
4. it should not be treated as the stable logical identity of the formula.

## 5. Syntax Artifact Vocabulary
### 5.1 `green_tree_key`
`green_tree_key` identifies a specific immutable green-tree root value.

Working meaning:
1. it corresponds to one exact full-fidelity syntax payload,
2. unchanged subtrees may be structurally reused across different `green_tree_key` roots,
3. it may be implemented as an interning key, content fingerprint, or explicit versioned object id.

### 5.2 `green_tree_fingerprint`
`green_tree_fingerprint` is the content fingerprint of a green-tree root.

Working meaning:
1. it reflects exact full-fidelity syntax content,
2. it is useful for replay, deduplication, and integrity checks,
3. it must not replace `formula_stable_id` or `formula_text_version`.

### 5.3 `red_view_key`
`red_view_key` identifies a contextual view over a green tree.

Working meaning:
1. it depends on at least the green tree plus contextual projection inputs,
2. it is normally ephemeral,
3. it should not be treated as durable semantic truth.

## 6. Structure and Bind Vocabulary
### 6.1 `structure_context_version`
`structure_context_version` is the declared version key for workbook structure relevant to binding.

Examples of contributors:
1. name scopes,
2. sheet/workbook identity graph,
3. table metadata,
4. caller anchor movement,
5. profile-gated grammar or feature enablement where binding is affected.

### 6.2 `bind_input_key`
`bind_input_key` is the conceptual key of one bind attempt.

Minimum contributors:
1. `formula_stable_id`
2. the current formula syntax identity
3. `structure_context_version`
4. `profile_version`

### 6.3 `bind_hash`
`bind_hash` is the content fingerprint of the bind result used for seam fencing.

Working meaning:
1. it changes when bound meaning changes,
2. it is stronger than a plain text-version key,
3. it may remain stable across some non-semantic changes if the binding result is identical,
4. it is not itself the stable identity of the bound formula.

### 6.4 `bound_formula_id`
`bound_formula_id` is the stable identity of a bound artifact when such an identity is needed by a repository-style implementation.

Working meaning:
1. it is optional in a purely stateless API,
2. it becomes useful in repository or cache-oriented implementations,
3. if present, it must remain distinct from `bind_hash`.

## 7. Semantic-Plan Vocabulary
### 7.1 `semantic_plan_key`
`semantic_plan_key` identifies one semantic-plan payload.

Minimum contributors:
1. `bind_hash`
2. relevant OxFunc catalog/profile information
3. evaluation-mode-affecting profile/version information

### 7.2 `semantic_plan_fingerprint`
`semantic_plan_fingerprint` is the content fingerprint of the compiled evaluator-facing plan.

Working meaning:
1. it is useful for replay and cache equivalence,
2. it must not be mistaken for a runtime handle.

## 8. Evaluation and Session Vocabulary
### 8.1 `snapshot_epoch`
`snapshot_epoch` is the version key for the workbook snapshot visible to evaluation.

Working meaning:
1. it changes when publish-relevant workbook state changes,
2. it is a fence input for evaluator sessions,
3. it is coordinator-facing in integrated mode.

### 8.2 `profile_version`
`profile_version` is the declared version key for enabled semantics/features/profile rules relevant to parsing, binding, or evaluation.

### 8.3 `capability_view_key`
`capability_view_key` identifies the evaluated capability surface for one session.

Working meaning:
1. it should change when capability-affecting rules or grants change,
2. it is distinct from `profile_version`,
3. it may contribute to commit fencing.

### 8.4 `session_id`
`session_id` is the runtime identity of an evaluator session.

Working meaning:
1. it is operational, not canonical semantic truth,
2. it may be short-lived,
3. it must still be traceable and replay-correlatable.

## 9. Overlay Vocabulary
### 9.1 `overlay_family`
The baseline overlay families are:
1. dependency overlay,
2. spill overlay,
3. format dependency overlay.

### 9.2 `overlay_scope_key`
`overlay_scope_key` identifies the fence scope under which an overlay entry is valid.

Minimum contributors:
1. `formula_stable_id`
2. `snapshot_epoch`
3. `bind_hash`
4. `profile_version`
5. overlay family

Additional contributors may be required for capability-sensitive lanes.

### 9.3 `overlay_entry_id`
`overlay_entry_id` is the runtime identity of one overlay record.

Working meaning:
1. it may be local to a repository or session store,
2. it is not a substitute for the overlay scope key,
3. replay should be expressible without depending on opaque local ids.

## 10. Publication Vocabulary
### 10.1 `commit_attempt_id`
`commit_attempt_id` identifies one publish attempt.

Working meaning:
1. it is useful for trace correlation,
2. it is not itself proof of publish success.

### 10.2 `commit_bundle_fingerprint`
`commit_bundle_fingerprint` is the fingerprint of one atomic publishable bundle.

Working meaning:
1. it is useful for replay equivalence and witness packs,
2. it must reflect the full published semantic payload, not just one delta family.

### 10.3 `reject_record_fingerprint`
`reject_record_fingerprint` is the fingerprint of a typed reject payload.

Working meaning:
1. it supports replay equivalence,
2. it does not replace the typed reject code and context fields themselves.

## 11. Minimum FEC/F3E Fence Tuple
The current minimum seam fence tuple remains:
1. `formula_stable_id`
2. `formula_token`
3. `snapshot_epoch`
4. `bind_hash`
5. `profile_version`

This document adds vocabulary around that tuple; it does not replace it.

Open question:
1. whether `capability_view_key` should become an explicit first-class fence member rather than a separately checked requirement.

## 12. Runtime Handles vs Canonical Keys
Repository-style implementations may introduce:
1. parse repository handles,
2. bound artifact handles,
3. semantic plan handles,
4. overlay store handles,
5. session handles.

Working rule:
1. runtime handles are allowed,
2. canonical replay and formal reasoning must not depend on opaque handles alone,
3. every publish-relevant outcome must be explainable in terms of canonical keys and explicit inputs.

## 13. Formalization Implications
Lean-oriented posture:
1. stable identities, version keys, and fingerprints should be modeled as distinct types or tagged aliases.

TLA+-oriented posture:
1. session ids and runtime handles may exist in the state machine,
2. publish eligibility should be defined in terms of fence members and explicit artifact relations, not accidental store addresses.

Replay posture:
1. replay packs should capture canonical keys and fingerprints,
2. local runtime handles may appear only as auxiliary debugging metadata.

## 14. Replay-Preserved Identity Rules
The Replay appliance projection for OxFml must preserve identity categories explicitly rather than compressing them into one bundle-local token.

Replay-preservation rules:
1. stable ids remain stable ids
   - `formula_stable_id` remains the logical formula-locus identity,
   - it may appear in replay correlation fields,
   - it may not be substituted by `session_id`, `commit_attempt_id`, or sidecar hash.
2. version keys remain version keys
   - `formula_text_version`, `structure_context_version`, `snapshot_epoch`, and `profile_version` remain separate version contexts,
   - replay normalization may add bundle-level configuration refs,
   - replay normalization may not reinterpret version keys as fingerprints.
3. content fingerprints remain fingerprints
   - `green_tree_fingerprint`, `bind_hash`, `semantic_plan_fingerprint`, `commit_bundle_fingerprint`, and `reject_record_fingerprint` remain content-equivalence markers,
   - fingerprints support replay equivalence and sidecar integrity,
   - fingerprints do not replace logical ids or version keys.
4. runtime handles remain auxiliary
   - `session_id`, repository handles, and overlay entry ids remain operational correlation aids,
   - replay bundles may preserve them when they are causally relevant,
   - replay-valid interpretation must remain possible without depending on opaque process-local handles alone.
5. fence-relevant keys remain explicit
   - at minimum `formula_stable_id`, `formula_token`, `snapshot_epoch`, `bind_hash`, and `profile_version` remain replay-visible for publish-safety reasoning,
   - `capability_view_key` must be preserved where present even while its final fence status remains open.
6. publication correlation ids remain explicit
   - `session_id` and `commit_attempt_id` are replay-correlatable ids and may not be folded into one generic run id,
   - replay bundles must preserve the distinction between candidate lineage and commit lineage.
7. configuration and profile context is additive, not substitutive
   - replay bundles may add capture mode, adapter version, and configuration fingerprint refs,
   - those additive refs do not replace `profile_version` or other OxFml semantic keys.

Current replay-governance pin:
1. registry-family pins for normalized replay governance are currently anchored to `oxfml.local.registry_pin.foundation_handoff_20260315_pass01`,
2. that pin governs registry interpretation, not OxFml artifact meaning.

## 15. Open Decisions
The following remain open:
1. exact derivation rule for `formula_token`,
2. whether `green_tree_key` and `green_tree_fingerprint` collapse in practice,
3. whether repository-style implementations expose `bound_formula_id` and `semantic_plan_key` publicly,
4. whether `structure_context_version` is global, partitioned, or lane-specific,
5. whether overlay scope must include capability-view identity explicitly,
6. final names for some of these keys once implementation starts.

## 16. Working Rule
Until implementation begins:
1. use this document's distinctions in prose specs,
2. avoid introducing new overloaded identity terms without defining their category,
3. prefer "stable id", "version key", "fingerprint", and "runtime handle" as separate terms.
