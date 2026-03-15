# OxFml Formula Engine Architecture

## 1. Purpose
This document defines the canonical OxFml architecture for Excel-compatible formula processing and single-node evaluation.

It specifies how OxFml will:
1. parse Excel formulas as full-fidelity immutable syntax trees,
2. project versioned contextual views over those trees,
3. bind names and references against workbook structure,
4. compile semantic plans that consume OxFunc function definitions,
5. execute formula evaluation and publish evaluator outputs through FEC/F3E.

This is an OxFml architecture document, not a host-specific or pathfinder-specific implementation note.

Companion documents:
1. `README.md`
2. `OXFML_PARSER_AND_BINDER_REALIZATION.md`
3. `OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
4. `EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md`
5. `../OXFML_IMPLEMENTATION_SURFACES_AND_STATE_OPTIONS.md`
6. `../OXFML_IMPLEMENTATION_BASELINE.md`
7. `../OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`
8. `../OXFML_CANONICAL_ARTIFACT_SHAPES.md`

## 2. Architectural Role
OxFml owns:
1. formula text ingestion and parse fidelity,
2. syntax-tree persistence and versioned formula views,
3. bind/reference normalization and unresolved-reference classification,
4. evaluator-side semantic planning and execution contracts,
5. FEC/F3E evaluator publication semantics,
6. evaluator-side trace and reject-detail structure.

OxFml consumes:
1. OxFunc function catalog identifiers, traits, and semantic metadata,
2. workbook/document structure supplied by the host or coordinator,
3. profile/version, locale, and date-system context from the enclosing engine surface.

OxFml exposes:
1. bound-reference and dependency evidence,
2. prepared-call contracts to OxFunc,
3. evaluator commit bundles and typed reject outcomes to OxCalc through FEC/F3E,
4. replay-stable traces for assurance tooling.

The detailed OxFml/OxFunc semantic boundary is specified in:
1. `OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`

## 3. Design Objectives
The baseline architecture must satisfy all of the following:
1. full-fidelity syntax preservation suitable for later interactive editing layers,
2. immutable versioned representations with maximal structural reuse across edits,
3. separation of syntax, binding, semantic planning, and evaluation concerns,
4. explicit distinction between references and dereferenced values,
5. deterministic replay of parse/bind/evaluate/commit behavior,
6. no hidden mutation of canonical syntax or bind truth.

## 4. Ownership and API Posture
The intended semantic posture is stronger than "use immutable data where convenient".

Canonical rule:
1. the output of parsing is an immutable green-tree value,
2. that green tree is suitable for inclusion in a larger immutable workbook/document structure owned by the enclosing host,
3. later bind and semantic artifacts should likewise admit immutable versioned representations tied to explicit structural and formula versions.

The implementation surface is still open.
Two shapes are currently acceptable:
1. a largely stateless API that consumes immutable inputs plus explicit context and returns immutable artifacts,
2. a service-oriented API that maintains caches or session state behind the boundary.

However, both shapes must satisfy the same semantic constraint:
1. canonical formula, bind, and commit truth must be representable as explicit immutable artifacts,
2. hidden mutable state must be optional optimization state only,
3. no caller should be forced to trust invisible mutation to explain formula meaning.

This means OxFml must be specifiable in a stateless, replayable form even if some implementations later use internal caches or registries.

The broader implementation-option tradeoff analysis is defined in:
1. `../OXFML_IMPLEMENTATION_SURFACES_AND_STATE_OPTIONS.md`

The current code-start implementation baseline is defined in:
1. `../OXFML_IMPLEMENTATION_BASELINE.md`

The canonical identity/version vocabulary used below is defined in:
1. `../OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`

The canonical artifact field surfaces used below are defined in:
1. `../OXFML_CANONICAL_ARTIFACT_SHAPES.md`

The parser/binder realization baseline is defined in:
1. `OXFML_PARSER_AND_BINDER_REALIZATION.md`

## 5. Layered Formula Pipeline
OxFml uses a layered pipeline:

1. **Formula source layer**
   - raw entered formula text,
   - stored/normalized formula text where the product surface maintains one,
   - source spans, token stream, and recovery diagnostics.
2. **Green syntax layer**
   - immutable, full-fidelity, parentless syntax nodes and tokens,
   - no workbook- or caller-specific context,
   - suitable for sharing across versions and views.
3. **Red syntax-view layer**
   - ephemeral contextual wrappers over green nodes,
   - parent/position/caller/workbook projection,
   - lazily materialized and safely cacheable.
4. **Binding layer**
   - name resolution,
   - reference normalization,
   - address-mode and caller-context application,
   - dependency seed derivation and capability discovery.
5. **Semantic-plan layer**
   - operator/function dispatch plan,
   - OxFunc function-definition links,
   - evaluation-mode policy and reduction policy,
   - fast-path classification and overlay requirements.
6. **Execution layer**
   - evaluation of one formula instance against snapshot context,
   - reference-preserving argument preparation,
   - dynamic dependency discovery and overlay updates.
7. **Publication layer**
   - FEC/F3E session lifecycle,
   - commit bundle construction,
   - typed reject detail,
   - trace emission.

## 6. Green/Red Syntax Architecture
### 6.1 Green nodes
The green layer is the canonical syntax substrate.

Green nodes must be:
1. immutable,
2. parentless,
3. context-free,
4. full-fidelity,
5. reusable across versions when source subtrees are unchanged.

Full-fidelity means green trees retain:
1. all operator and delimiter tokens,
2. trivia required for round-tripping and editing support,
3. exact token spans,
4. recovery/error nodes for malformed input,
5. syntax that later normalizes away at storage or bind time.

Examples of syntax that must survive in the green layer even when later stages reinterpret it:
1. explicit `@`,
2. spill suffix `#`,
3. whitespace intersection operator,
4. structured-reference tokens,
5. workbook-qualified and sheet-qualified name syntax,
6. malformed-but-recoverable token sequences.

Green trees are expected to participate in higher immutable structures.
In OxCalc-integrated mode, that likely means workbook/document versions or formula registries above the evaluator.
The exact container type remains open, but the green tree itself is not.

### 6.2 Red nodes
The red layer provides contextual views over green trees.

Red nodes add:
1. parent links,
2. absolute position/span projection,
3. owning formula identity,
4. workbook/sheet/caller context,
5. cached lookup helpers for later editing and semantic layers.

Red nodes must be:
1. lazily created,
2. disposable,
3. reconstructible from green tree plus context,
4. non-authoritative for persistent semantics.

### 6.3 Versioned views
OxFml must support versioned formula views across both formula-text edits and structure edits.

There are two important version axes:
1. **formula-text version**
   - the formula source text changed,
   - green root and formula token may change,
   - unchanged subtrees are structurally reused.
2. **structure-context version**
   - the formula text did not change,
   - workbook structure, name scopes, caller address, or profile context changed,
   - green root may remain identical while red/bind views change.

This separation is mandatory because many Excel semantics change under structure edits or caller movement without the user editing formula text.

### 6.4 Edit semantics
An edit must rebuild only the changed leaf payloads and ancestor spine to the root.
Untouched green subtrees are reused by reference.

This is the baseline for the later interactive editing layer. OxFml does not itself need to expose Roslyn-like workspace APIs yet, but the syntax model must not block them.

## 7. Binding Architecture
### 7.1 Bind context
Binding consumes the red syntax view plus a bind context containing:
1. workbook and sheet identity,
2. caller anchor identity,
3. address mode and relative-reference interpretation,
4. name-scope tables,
5. table/structured-reference metadata,
6. profile and profile version,
7. locale identity,
8. workbook date system,
9. feature/capability surface.

### 7.2 Bind outputs
Binding produces:
1. `BoundFormula`,
2. normalized reference set,
3. dependency seed set,
4. unresolved/error reference records,
5. capability requirements,
6. a `bind_hash` used as part of session and overlay fencing.

`BoundFormula` and its associated bind products should be modeled as immutable versioned artifacts.
Implementations may cache them, but cache residency must not define their semantics.

The identity and version categories for these outputs should follow:
1. `bind_input_key`
2. `bind_hash`
3. optional `bound_formula_id` where repository-style implementations need one

## 8. Semantic Planning and OxFunc Integration
### 8.1 Semantic plan
`SemanticPlan` is the compiled evaluator-facing representation derived from `BoundFormula` plus OxFunc metadata.

A semantic plan must carry:
1. operator/function dispatch identity,
2. function traits from OxFunc,
3. evaluation-mode requirements,
4. reduction policy requirements,
5. overlay participation flags,
6. format/locale service needs,
7. fast-path eligibility.

Semantic-plan identity should distinguish:
1. `semantic_plan_key` as the plan identity/version key,
2. `semantic_plan_fingerprint` as the content fingerprint where replay or cache equivalence needs one.

### 8.2 OxFunc contract
OxFunc owns function semantics, but OxFml must prepare arguments and results in a way that preserves Excel-relevant structure.

Prepared arguments must be able to distinguish:
1. direct scalar input,
2. array-like input,
3. reference-visible input,
4. omitted arguments,
5. blank cell vs empty string vs error,
6. eager vs lazy vs reference-preserved evaluation mode.

Prepared results must be able to distinguish:
1. scalar value,
2. array payload,
3. reference result,
4. error result,
5. attached format hint or publication hint when the semantic lane requires it.

The detailed field-level semantic contract is defined in:
1. `OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`

## 9. Reference Preservation and Dereferencing
OxFml must not collapse all references to plain values before function dispatch.

The architecture distinguishes:
1. **reference identity**
   - what location or region the expression denotes,
2. **reference view**
   - how that identity is exposed to the consuming function,
3. **materialized value**
   - the scalar or array payload obtained by dereferencing.

Dereferencing is a semantic act, not a parser default.

## 10. Evaluation Pipeline
The baseline OxFml evaluation sequence is:
1. parse source text to green tree,
2. materialize red view for the current formula/context,
3. bind and normalize references,
4. compile semantic plan with OxFunc metadata,
5. create or reuse an evaluation session through FEC/F3E,
6. prepare arguments and evaluate,
7. record dynamic overlays when discovered,
8. publish an atomic commit bundle or a typed reject.

Evaluation-state posture:
1. canonical evaluation meaning is a function of immutable artifacts plus explicit snapshot/context inputs,
2. evaluator sessions, registries, and caches may exist as implementation devices,
3. those stateful devices must remain replayable, fenceable, and replaceable by an observationally equivalent stateless interpretation.

The canonical minimum keys involved in evaluator fencing and publication are:
1. `formula_stable_id`
2. `formula_token`
3. `snapshot_epoch`
4. `bind_hash`
5. `profile_version`

## 11. Formalization Hooks
The formula engine architecture must be suitable for near-formal treatment.

Key formalizable surfaces are:
1. green-tree full-fidelity invariants,
2. red-view contextual reconstruction rules,
3. bind normalization and unresolved-reference classification,
4. prepared-call structure preservation for OxFunc-facing semantics,
5. fast-path soundness conditions,
6. replay equivalence between direct execution and recorded artifacts.

This layer should be designed so that:
1. syntax and bind ADTs can be mirrored in Lean,
2. replay traces can witness semantic claims concretely,
3. no architecture shortcut forces semantic collapse that the formal model cannot express honestly.

## 12. Repo Module Shape
The intended OxFml implementation shape follows the system design split:
1. `syntax`
2. `binding`
3. `semantics`
4. `evaluation`
5. `fec_f3e`
6. `replay_and_conformance`
7. `formal`

Exact source layout may evolve, but this conceptual structure is the baseline.

## 13. DNA OneCalc Mode
DNA OneCalc is a downstream proving host for OxFml and OxFunc.
It is not the source of truth for OxFml architecture.

In DNA OneCalc mode:
1. the same parse/bind/eval contracts apply,
2. the host may omit OxCalc multi-node scheduling and dependency closure,
3. a reduced FEC/F3E profile may be used for single-node proving,
4. no OxCalc-owned coordinator policy is implied by the host.

## 14. Required Artifacts and Tests
OxFml must eventually ship the following assurance layers:
1. full-fidelity parse corpus with stored-text and entered-text comparison,
2. bind/reference normalization fixtures,
3. OxFunc prepared-call contract fixtures,
4. typed reject and trace-schema fixtures,
5. deterministic replay bundles for dynamic-reference and spill scenarios,
6. pack-level conformance for FEC/F3E transaction boundaries.

Detailed pack/test strategy is defined in `../fec-f3e/FEC_F3E_TESTING_AND_REPLAY.md`.

Empirical execution plans and seed manifests are retained under:
1. `archive/`
