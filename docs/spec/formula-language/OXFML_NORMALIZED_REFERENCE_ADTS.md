# OxFml Normalized Reference ADTs

## 1. Purpose
This document defines the current normalized reference ADT baseline for OxFml binder work.

It exists to make explicit:
1. which reference distinctions the binder must preserve,
2. which parts belong in normalized reference atoms versus reference-expression nodes,
3. how dynamic and runtime-discovered references fit into the same model,
4. what later semantic planning and replay work may rely on.

Read together with:
1. `OXFML_PARSER_AND_BINDER_REALIZATION.md`
2. `OXFML_FORMULA_ENGINE_ARCHITECTURE.md`
3. `OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
4. `../OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`
5. `../OXFML_CANONICAL_ARTIFACT_SHAPES.md`

## 2. Working Rule
Reference normalization must preserve semantic shape without pretending that all references are statically resolved or already dereferenced.

The binder should therefore separate:
1. normalized reference atoms,
2. reference-expression composition nodes,
3. unresolved-reference records,
4. runtime-reference plans that are only fully discovered during evaluation.

## 3. Layer Split
The normalized reference model has three layers.

### 3.1 Syntax reference fragments
These are parser-level shapes such as:
1. `A1`
2. `A1:B3`
3. `A:A`
4. `1:3`
5. `Table1[Amount]`
6. `NameX`
7. `A1#`
8. `A1 B2:C3`

They remain visible in syntax trees and red views.

### 3.2 Normalized reference atoms
These are binder outputs representing stable reference intent under a caller/context interpretation.

The current canonical atom families are:
1. `CellRef`
2. `AreaRef`
3. `WholeRowRef`
4. `WholeColumnRef`
5. `NameRef`
6. `StructuredRef`
7. `ExternalRef`
8. `ErrorRef`

### 3.3 Reference-expression bindings
These are bound expression nodes that combine or transform reference atoms.

The current canonical families are:
1. `RangeRefExpr`
2. `UnionRefExpr`
3. `IntersectionRefExpr`
4. `SpillRefExpr`
5. `RuntimeReferenceExpr`

This split matters because `A1:B3` and `OFFSET(A1,0,0,2,2)` may both eventually denote an area, but they do not have the same bind-time status or replay story.

## 4. Canonical Atom Families
### 4.1 `CellRef`
Represents one normalized cell reference.

Minimum fields:
1. workbook scope identity
2. sheet scope identity
3. row index
4. column index
5. address-mode metadata
6. caller-anchor or rewrite-origin metadata when relative interpretation mattered

### 4.2 `AreaRef`
Represents one normalized rectangular cell area.

Minimum fields:
1. workbook scope identity
2. sheet scope identity
3. top-left cell coordinate
4. height
5. width
6. address-mode metadata
7. caller-anchor or rewrite-origin metadata when relative interpretation mattered

### 4.3 `WholeRowRef`
Represents one or more whole rows on a known sheet scope.

Minimum fields:
1. workbook scope identity
2. sheet scope identity
3. row-start
4. row-count
5. address-mode metadata

### 4.4 `WholeColumnRef`
Represents one or more whole columns on a known sheet scope.

Minimum fields:
1. workbook scope identity
2. sheet scope identity
3. column-start
4. column-count
5. address-mode metadata

### 4.5 `NameRef`
Represents a bound defined-name reference.

Minimum fields:
1. bound name identity
2. workbook or sheet scope identity
3. declared name kind
   - reference-like
   - value-like
   - mixed or deferred
4. caller-context flags when the name meaning depends on caller location

Working rule:
1. binding a name does not imply eager dereference,
2. names that may yield references must remain distinguishable from plain value names.

### 4.6 `StructuredRef`
Represents a bound table or structured-reference lane.

Minimum fields:
1. table identity
2. structured selector kind
3. selected column or section identities
4. caller-row dependence marker where row-context semantics apply
5. workbook or sheet scope identity

### 4.7 `ExternalRef`
Represents a reference that crosses workbook or external-source boundaries.

Minimum fields:
1. external target identity
2. sheet or area selector summary
3. capability or availability requirement marker
4. replay-visible external reference class

### 4.8 `ErrorRef`
Represents a syntactically reference-shaped expression that binds to a typed reference error.

Minimum fields:
1. error class
2. source reference kind
3. optional scoped target summary
4. bind-stage diagnostic linkage

Working rule:
1. `ErrorRef` is different from a parser failure,
2. `ErrorRef` is different from an unresolved-reference record that still requires host or later bind context.

## 5. Reference-Expression Binding Families
### 5.1 `RangeRefExpr`
Represents a range-construction expression with normalized endpoints or endpoint expressions.

Use when:
1. endpoints are not best modeled as one direct `AreaRef`,
2. replay or diagnostics need the bound range construction to remain explicit.

### 5.2 `UnionRefExpr`
Represents comma-union reference composition.

Canonical property:
1. union composition remains explicit rather than flattened into a generic list too early.

### 5.3 `IntersectionRefExpr`
Represents whitespace-intersection or other explicit reference-intersection composition.

Canonical property:
1. intersection semantics must not be erased by token normalization.

### 5.4 `SpillRefExpr`
Represents spill-anchor selection such as `A1#`.

Minimum fields:
1. anchor reference or anchor expression
2. spill-selection mode
3. caller-context and capability requirements where spill visibility depends on them

Working rule:
1. spill selection is not a plain `AreaRef` at bind time,
2. the runtime-discovered spill shape belongs to evaluator facts and deltas, not to the binder pretending the final area is already known.

### 5.5 `RuntimeReferenceExpr`
Represents a reference-yielding expression whose concrete target is discovered only at evaluation.

Typical producers:
1. `OFFSET`
2. `INDIRECT`
3. reference-returning `INDEX`
4. reference-returning `XLOOKUP`

Minimum fields:
1. operator or function identity
2. normalized child operands
3. declared runtime-reference class
4. capability requirements
5. replay-visible dynamic-reference marker

Working rule:
1. this is a first-class bound shape,
2. the binder must not materialize it as a value only because the runtime target is not yet known.

## 6. Unresolved and Invalid Cases
The binder must keep these cases separate:
1. `ErrorRef`
   - binding has a typed reference error outcome
2. unresolved-reference record
   - binding could not honestly finish with current context and must record why
3. unsupported-lane marker
   - syntax is accepted but current profile or capability blocks semantic use
4. runtime-reference plan
   - semantic completion is intentionally deferred to evaluation

This distinction is required for replay, diagnostics, and later host/profile evolution.

## 7. Caller Context and Address Mode
Normalized references must preserve enough context to explain:
1. relative versus absolute address interpretation,
2. caller-anchor-sensitive binding,
3. row-context-sensitive structured references,
4. later `@` and scalarization interactions.

Working rule:
1. caller context may influence normalization,
2. it must not be silently baked in with no record that caller context mattered.

## 8. Dynamic Discovery and Evaluator Interaction
Binding may emit dependency seeds without claiming full dependency closure.

The current contract is:
1. static reference atoms feed initial dependency seeds,
2. `RuntimeReferenceExpr` feeds dynamic-dependency and runtime-reference discovery later,
3. evaluator facts and topology deltas carry the runtime-discovered consequences.

This is the main reason normalized reference ADTs and dynamic-reference facts must stay aligned across `W002`, `W003`, and `W004`.

## 9. Non-Goals
The normalized reference ADT layer should not:
1. force all references into a rectangular-area-only model,
2. erase structured-reference selectors,
3. erase spill-anchor versus concrete-spill-area distinction,
4. approximate runtime-reference expressions as eager values,
5. collapse unresolved-reference records into parser diagnostics.

## 10. Workset Implications
Current expected primary owners:
1. `W002`: finalize atom and reference-expression families for implementation-start
2. `W003`: align runtime-reference and reference-preserving execution requirements with OxFunc semantic planning
3. `W004`: align dynamic-reference discoveries with FEC/F3E surfaced facts and replay payloads

## 11. Open Decisions
The following remain open:
1. exact encoding of workbook and sheet identities,
2. whether `RangeRefExpr` is retained only when endpoints are dynamic or more broadly,
3. whether `ExternalRef` needs finer target classes in the first code slice,
4. how much caller-anchor metadata remains inline versus in attached provenance records,
5. whether table-structured selectors should be split into smaller ADTs in the first implementation.
