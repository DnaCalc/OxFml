# OxFml Parser and Binder Realization

## 1. Purpose
This document tightens the formula-engine architecture into a practical realization baseline for parser and binder work.

It defines:
1. the intended parser substrate and recovery posture,
2. the intended green/red realization boundary,
3. the bind inputs and outputs,
4. the incremental reparse and rebind story,
5. the way immutable formula artifacts fit into higher immutable document structures in OxCalc-integrated mode.

This document is a realization note, not a host-specific implementation plan.

Read together with:
1. `OXFML_FORMULA_ENGINE_ARCHITECTURE.md`
2. `OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
3. `../OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`
4. `../OXFML_CANONICAL_ARTIFACT_SHAPES.md`
5. `../OXFML_IMPLEMENTATION_BASELINE.md`
6. `OXFML_NORMALIZED_REFERENCE_ADTS.md`

## 2. Realization Rule
The parser and binder must be implementable as explicit transforms over immutable artifacts.

Operational caches, syntax interners, and bind repositories are allowed.
They must remain reducible to:
1. explicit formula source inputs,
2. explicit structure/profile context,
3. explicit immutable outputs.

## 3. Parser Realization Baseline
### 3.1 Lexical model
The parser should operate over a full-fidelity token stream that preserves:
1. all significant formula tokens,
2. trivia needed for round-trip and editing support,
3. malformed token fragments when recoverable,
4. span information tied to entered formula text.

The lexer must not normalize away:
1. explicit `@`,
2. spill suffix `#`,
3. whitespace intersection,
4. structured-reference bracket structure,
5. workbook and sheet qualification tokens,
6. malformed-but-classifiable fragments.

### 3.2 Parse output
The canonical parser output is `GreenTreeRoot`.

The parser should produce:
1. one green root,
2. parse diagnostics,
3. recovery/error nodes where the input is malformed but still structurally representable,
4. enough token/trivia fidelity for later stored-text and entered-text comparison.

### 3.3 Parse strategy
The intended baseline is a hand-authored or generated recursive-descent style parser with explicit precedence handling and explicit recovery points.

Required properties:
1. precedence and associativity are encoded declaratively or structurally enough to be audited,
2. recovery points are deterministic,
3. malformed constructs still produce a full-fidelity tree rather than only a fatal parse failure where practical.

## 4. Green and Red Realization
### 4.1 Green realization
Green nodes are:
1. parentless,
2. immutable,
3. context-free,
4. shareable across versions.

Minimum implementation expectation:
1. unchanged subtrees are structurally reusable across formula-text edits,
2. `green_tree_key` identifies the canonical root artifact,
3. `green_tree_fingerprint` may be used for equality, caching, or replay correlation without replacing `green_tree_key`.

### 4.2 Red realization
Red nodes are contextual projections over green nodes.

A red projection should be constructible from:
1. green root identity,
2. formula identity,
3. source-span mapping,
4. caller/workbook/profile context where needed.

Red nodes may carry:
1. parent links,
2. child-position lookup,
3. absolute span computation,
4. helper views for parser diagnostics, binding, and later editing support.

### 4.3 Red authority rule
Red nodes are convenience structures, not semantic truth.

If a red node can only be explained by hidden service state, the realization is out of bounds.

## 5. Incremental Reparse Baseline
### 5.1 Formula-text edit
When formula text changes:
1. a new `formula_text_version` is created,
2. the parser rebuilds only the changed leaf payloads and ancestor spine,
3. untouched green subtrees are reused by reference where the implementation can prove equivalence,
4. a new `formula_token` is produced when the formula-source record changes semantically or textually.

### 5.2 Structure-only change
When workbook structure or caller context changes without text change:
1. the existing green tree may remain unchanged,
2. the red projection changes,
3. binding must be reevaluated against the new `structure_context_version`,
4. semantic meaning may change even though syntax does not.

This is a core reason parser and binder versions must remain separate.

## 6. Binder Realization Baseline
### 6.1 Bind inputs
Binding consumes:
1. a green root plus red contextual projection,
2. `formula_stable_id`,
3. `formula_token`,
4. `structure_context_version`,
5. scope and table metadata,
6. caller anchor and address-mode context,
7. profile and capability context,
8. locale and date-system context where binding depends on them.

### 6.2 Bind outputs
Binding produces:
1. `BoundFormula`,
2. normalized reference records,
3. dependency seeds,
4. unresolved-reference records,
5. capability requirements,
6. bind diagnostics,
7. `bind_hash`.

### 6.3 Binder responsibilities
The binder owns:
1. turning syntax references into normalized reference ADTs,
2. applying relative/absolute interpretation against caller context,
3. resolving workbook/sheet/name/table scope,
4. classifying unresolved or invalid reference states explicitly,
5. extracting dependency seeds without pretending all dynamic dependencies are known statically.

The current normalized reference baseline for this step is:
1. `OXFML_NORMALIZED_REFERENCE_ADTS.md`

The binder must not:
1. erase explicit syntax distinctions needed by later semantic planning,
2. treat runtime-discovered reference targets as static truth,
3. collapse unresolved cases into generic parser errors.

## 7. Incremental Rebind Baseline
### 7.1 Rebind triggers
A rebind is required when any of the following change:
1. `formula_token`,
2. `structure_context_version`,
3. relevant scope or table metadata,
4. profile-version inputs that affect binding legality,
5. caller anchor when relative semantics depend on it.

### 7.2 Rebind reuse
An implementation may reuse prior bind artifacts when:
1. the same `formula_token` and `structure_context_version` are presented,
2. the same effective bind context is proven,
3. the same capability/profile inputs are proven.

Reuse must be keyed explicitly.
Implicit reuse by ambient process state is out of bounds.

## 8. Immutable Document Integration
### 8.1 OxCalc-integrated mode
In OxCalc-integrated mode, the likely higher-level shape is:
1. host or OxCalc owns immutable workbook/document snapshots,
2. formula source records and green roots are retained as versioned document artifacts,
3. bind artifacts and semantic plans may be retained either above OxFml or in OxFml repositories,
4. FEC/F3E sessions consume those artifacts but do not own canonical document truth.

### 8.2 DNA OneCalc mode
In DNA OneCalc mode, the same artifact relationships apply in smaller form:
1. immutable formula and context artifacts remain canonical,
2. the host may choose lighter-weight storage,
3. no OxCalc-specific scheduling structure is implied.

## 9. Early Implementation Order
The intended first implementation order is:
1. formula source record and token stream,
2. green tree and parse diagnostics,
3. red projection helpers,
4. normalized reference ADTs,
5. binder over explicit context,
6. incremental reparse and rebind reuse keys,
7. semantic-plan compilation and evaluator preparation.

This order is chosen to keep immutable artifacts and replay hooks explicit from the start.

## 10. Open Decisions
The following remain open:
1. exact parser construction technique,
2. exact internal encoding and residency strategy for normalized reference ADTs,
3. whether red projections are lightweight structs, arenas, or generated wrappers,
4. whether bind repositories are OxFml-owned services or host-owned caches in integrated mode,
5. how far incremental subtree reuse goes in the first implementation slice.

## 11. Working Rule
Parser and binder implementation must start from:
1. immutable artifact truth,
2. explicit version keys,
3. deterministic recovery and diagnostics,
4. separation between formula-text change and structure-context change.
