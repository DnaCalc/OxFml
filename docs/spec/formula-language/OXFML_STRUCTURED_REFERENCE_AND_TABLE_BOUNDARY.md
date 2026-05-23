# OxFml Structured Reference and Table Boundary

## Purpose
Define the intended first implementation-facing boundary for structured table references.

This document separates:
1. host/coordinator ownership of table objects and metadata,
2. OxFml ownership of grammar, bind, and evaluator consequences,
3. OxFunc ownership of function semantics after structured references have been normalized into ordinary reference or scalar inputs.

It is a planning and draft-boundary artifact for `W036`, not a claim that the full structured-reference floor is already exercised locally.

## Core Ownership Rule
1. the host or coordinator owns table objects, table identities, table ranges, column metadata, and current-row context for table formulas,
2. OxFml owns structured-reference grammar, selector parsing, table-aware bind semantics, and evaluator/FEC consequences once the required table context is presented,
3. OxFunc should not own workbook table objects or table-context reconstruction,
4. OxFunc should consume either:
   - normalized structured-reference meaning preserved as a reference-like lane, or
   - already-dereferenced scalar/array/reference values after OxFml/host resolution,
   depending on the existing prepared-argument mode.

## Why This Split
The reviewed `MS-OE376` and Excel-support sources imply:
1. structured references are first-class formula syntax,
2. omitted-table-name forms cannot be resolved from syntax alone,
3. `#This Row` and `[@...]` are true row-context-sensitive lanes,
4. table identifiers must remain distinct from defined names,
5. calculated-column and table-scope behavior is host/object-context-sensitive rather than pure function semantics.

That means table ownership belongs with the host/coordinator layer, but formula meaning still belongs with OxFml once that context is supplied.

## Minimum Spec-Derived Requirements
The current Foundation-backed `MS-OE376` extraction already requires this boundary to preserve at least:
1. if `table-name` is missing, the formula containing the structured reference must be entered into a cell that belongs to a table, and that enclosing table name is used as the effective table name,
2. `table-name` must be the name of a table rather than an interchangeable defined-name or workbook identifier,
3. `#This Row` denotes current-row-sensitive selection,
4. `#This Row` must not be combined with `#Headers`, `#Total Row`, `#Data`, or `#All`,
5. qualifier combinations such as `[[#All],[column-name]]`, `[[#Data],[column-name1]:[column-name2]]`, and `[[#This Row],[column-name]]` remain distinct bind-time selector families.

Working rule:
1. the first local floor may still be smaller than the whole spec family,
2. but any narrower first floor must preserve these distinctions honestly rather than silently collapsing them.

## Boundary To OxCalc / Host

### Host-owned truth
The host should own and present:
1. `table_id`
2. `table_name`
3. `table_range_ref`
4. `header_row_present`
5. `totals_row_present`
6. column identity map:
   - canonical column ids
   - display/header names
   - ordinal positions
7. caller-table context where applicable:
   - enclosing table id
   - caller row offset within the data body
   - whether the caller is in header/data/totals region

### First table context packet
The smallest honest first packet is:
1. `table_context_bundle`
2. `enclosing_table_ref`
3. `table_catalog`
4. `caller_table_region`

Suggested first packet shape:
1. `table_catalog: [TableDescriptor]`
2. `enclosing_table_ref: Option<TableRef>`
3. `caller_table_region: Option<TableCallerRegion>`

Suggested first descriptor fields:
1. `table_id`
2. `table_name`
3. `workbook_scope_ref`
4. `sheet_scope_ref`
5. `table_range_ref`
6. `row_membership_identity: Option<String>`
7. `row_order_identity: Option<String>`
8. `header_region_ref: Option<String>`
9. `totals_region_ref: Option<String>`
10. `header_row_present`
11. `totals_row_present`
12. `columns: [TableColumnDescriptor]`

The row membership/order identities are stable host/coordinator facts for the
data rows. They are prepared-identity inputs, not TreeCalc semantics. The
header/totals region refs name the exact host-owned regions for `#Headers` and
`#Totals` when present; if omitted, the current first-slice resolver derives
the row from `table_range_ref`.

Suggested first column descriptor fields:
1. `column_id`
2. `column_name`
3. `ordinal`
4. `column_range_ref`

Zero-row data-body rule:
1. `column_range_ref` may be empty when the host-owned table currently has no
   data-body rows for that column.
2. An empty `column_range_ref` is not a parseable A1 area and must not force
   dense/eager materialization.
3. `TableDescriptor.row_membership_identity` and `row_order_identity` remain
   the stable data-body identity inputs for empty and non-empty bodies.
4. `header_region_ref`, `totals_region_ref`, `table_range_ref`, column
   identity, and ordinal are sufficient for `#Headers`, `#Totals`, and `#All`
   packet projection when the data body is empty.

Suggested first caller-region fields:
1. `table_id`
2. `region_kind`
3. `data_row_offset`

Additional packet rule:
1. `enclosing_table_ref` is required for omitted-table-name and current-row-sensitive forms,
2. `caller_table_region` is required whenever `#This Row`, `[@...]`, or totals/header/data region semantics could differ,
3. direct hosts and OxCalc-integrated hosts should supply the same semantic packet even if their surrounding transport differs.

## Boundary To OxFml
Once the host provides the table context packet, OxFml should own:
1. structured-reference tokenization and bracket/qualifier parsing,
2. distinction between:
   - explicit table forms `Table1[Amount]`
   - omitted-table-name forms `[@Amount]`
   - qualifier forms like `[#All]`, `[#Headers]`, `[#Totals]`, `[#Data]`, `[#This Row]`,
3. disambiguation between:
   - defined names,
   - table names,
   - column selectors,
4. normalization into a bound `StructuredRef`,
5. dereference policy into:
   - preserved reference-like prepared args where reference semantics matter,
   - eager scalar/array materialization where the normal pipeline already does that.

### Generic structured-reference bind record packet

OxFml now exposes a public, product-neutral structured-reference bind record
alongside the normalized `StructuredRef`. This packet is intended for hosts and
coordinators such as OxCalc that need dependency and invalidation facts without
parsing formula text.

Each structured-reference bind record carries:
1. `bind_record_handle`,
2. `source_span_utf8: TextSpan`,
3. exact `source_token_text`,
4. `explicit_table_name` plus `omitted_table_name`,
5. resolved/effective table identity when bind succeeds,
6. `selected_column_ids`,
7. selected section qualifiers and selected region descriptors,
8. `uses_this_row` / `caller_context_dependent`,
9. typed resolved-reference descriptor when bind succeeds,
10. typed diagnostic links when the structured-reference parser recognized the
    syntax but bind failed.

For empty data bodies, selected `#Data` and data-column bind records use an
explicit empty resolved-reference descriptor rather than inventing an A1 data
area. Their selected-region descriptor marks the data region empty, preserves
the selected column ids, and carries no data-column range refs. Current-row
forms such as `[@Amount]` remain caller-context-dependent but produce a typed
bind diagnostic when no data row exists for the supplied caller table region.

Runtime prepared identity and formal-reference projection preserve these
records. `RuntimeFormalReference.structured_reference_bind_record_handle`
links a formal reference back to the corresponding bind record where available.

Current W074/W056 runtime/replay identity evidence proves the generic packet is
prepared-identity and replay-projection relevant without hardcoding any
TreeCalc dependency meaning:
1. selected column range changes update formal references and the prepared key,
2. table id and selected column id changes update structured-reference bind
   records and the prepared key,
3. table range and selected column ordinal changes update the conservative
   `table_context_fingerprint` and prepared key even when the selected data
   reference and value stay stable,
4. row membership/order identity changes update `table_context_fingerprint`
   and prepared identity without changing the already-resolved reference,
5. exact header/totals region refs update selected-region descriptors,
   resolved references, and prepared identity for `#Headers` / `#Totals`,
6. unrelated catalog entry mutation is conservative identity input only: the
   referenced table bind record and formal reference stay stable while the full
   table-context fingerprint changes,
7. omitted-table-name references preserve enclosing table identity and
   caller-table data-row offset in the resolved reference and prepared key.
8. zero-row data-body references preserve source span/token, effective table
   identity, selected columns/sections, empty selected-region markers,
   prepared identity, and replay projection without requiring a parseable
   non-empty data-column A1 area; zero-row current-row forms preserve the same
   packet identity plus typed diagnostics.

## Boundary To OxFunc
OxFunc should not receive raw table metadata as part of ordinary function semantics unless a later packet proves that necessary.

Current intended OxFunc-facing rule:
1. OxFunc receives the same kinds of prepared inputs it already understands,
2. structured-reference-specific meaning should already be reduced by OxFml and host context into:
   - reference-preserved prepared args,
   - caller-context-sensitive scalarized values,
   - eager scalar/array values,
3. if a structured reference remains visible downstream, it should be through an opaque/stable normalized reference intent, not a workbook-table object model.

Suggested seam wording to OxFunc:
1. treat structured references as an upstream OxFml bind/resolution concern,
2. do not require OxFunc catalog/runtime semantics to reconstruct omitted-table-name or row-context behavior,
3. only request a richer downstream structured-reference carrier if a concrete function-semantic mismatch proves the current prepared-argument modes are insufficient.

## First Normalized Reference Shape
The existing normalized-reference plan already points to `StructuredRef`.

For `W036`, the minimum bound fields should be:
1. `table_id`
2. `selector_kind`
3. `selected_column_ids`
4. `caller_row_sensitive`
5. `workbook_scope_ref`
6. `sheet_scope_ref`

The first selector families to support should be:
1. `Column`
2. `ThisRowColumn`
3. `Section`
4. `All`
5. `Data`
6. `Headers`
7. `Totals`
8. mixed qualifier-plus-column combinations like `[[#All],[Amount]]`

## First Resolution Rules
The first local floor should preserve these rules:
1. `Table1[Amount]` resolves against explicit table identity from `table_catalog`,
2. `[@Amount]` requires `enclosing_table_ref` and caller row context,
3. omitted-table-name forms fail bind honestly if no enclosing table context exists,
4. table identifiers must not be silently treated as defined names,
5. malformed qualifier/bracket structure remains a syntax/bind failure, not a generic unknown-name lane.
6. `#This Row` must fail bind honestly when combined with `#Headers`, `#Total Row`, `#Data`, or `#All`,
7. qualifier combinations involving data/header/totals/all sections must preserve section meaning rather than collapsing immediately to one generic table-range alias.
8. empty table data bodies are representable in the generic packet model:
   `#Data` and data-column selections may resolve to an explicit empty
   structured data reference, while current-row forms against an empty data
   body fail bind with a typed diagnostic.

## First Evaluator Rules
The first local evaluator floor should cover:
1. reference-preserved use:
   - `SUM(Table1[Amount])`
2. current-row-sensitive use:
   - `=[@Amount]*2`
3. qualifier combinations:
   - `SUM(Table1[[#All],[Amount]])`
4. section-only selectors over all columns:
   - `=Table1[#Headers]`
   - `=Table1[#Totals]`
5. first multi-column section-qualified selector:
   - `SUM(Table1[[#Data],[Amount]:[Tax]])`

Out of first floor:
1. full totals-row computed semantics,
2. every table mutation behavior,
3. broad spill-growth interactions,
4. full host UI policy for calculated columns.

## Proposed Seam Changes

### OxFml/OxCalc
Suggested change:
1. make table metadata an explicit host/coordinator packet instead of leaving structured references as an implicit formula-only lane,
2. keep table ownership aligned with how defined names are already host-owned,
3. add explicit caller-table-region carriage for omitted-table-name and `#This Row` forms.
4. treat table-name versus defined-name disambiguation as an OxFml bind consequence over host-owned packet truth, not a coordinator-side pre-resolution rewrite.

### OxFml/OxFunc
Suggested change:
1. explicitly state that structured references are not an OxFunc object-model concern,
2. keep OxFunc on prepared-reference and value lanes only,
3. only reopen this seam if a concrete prepared-argument insufficiency appears.
4. if a structured reference must remain visible downstream, prefer a stable opaque normalized-reference intent over table-object reconstruction.

## Edge Cases To Cover
The first test plan should include:
1. explicit table column:
   - `=SUM(Table1[Amount])`
2. explicit qualifier combination:
   - `=SUM(Table1[[#All],[Amount]])`
3. omitted table name inside row context:
   - `=[@Amount]*2`
4. omitted table name without enclosing table:
   - bind failure, not silent fallback
5. table name versus defined-name collision
6. malformed bracket nesting
7. `#This Row` versus plain column selector distinction
8. totals/header/data qualifier selection
9. reference-preserved versus eager-scalarized downstream behavior where current pipeline modes differ
10. replay preservation of table identity and row-context-sensitive truth
11. illegal `#This Row` combination rejection
12. omitted-table-name forms in totals or header regions where current-row-sensitive meaning is not admissible

## Source Alignment
This plan is aligned with current local source interpretation:
1. `MS-OE376` review outcome in `MS_OE376_FORMULA_AND_FORMATTING_REVIEW.md`
2. current rule rows `FML-R-009` and `FML-R-013`
3. support-source anchors:
   - `ECS-012`
   - `ECS-013`
   - `ECS-014`

## Current Recommendation For W036
Execute `W036` in this order:
1. freeze the OxCalc/host table-context packet,
2. implement first parser/binder structured-reference floor,
3. prove disambiguation against defined names,
4. wire the first reference-preserved and current-row-sensitive evaluator lanes,
5. only then decide whether any narrower OxFunc-facing carrier change is actually needed.

## First Cross-Repo Review Questions
For OxCalc:
1. is `table_catalog + enclosing_table_ref + caller_table_region` the right first semantic packet,
2. does OxCalc need any narrower region or anchor facts for first TreeCalc integration,
3. should totals/header/data region identity stay in the packet even when first execution support is smaller than the full table object model.

For OxFunc:
1. are current preserved-reference and ordinary prepared-value lanes enough for first structured-reference consumers,
2. is any richer downstream carrier needed before there is real evaluator evidence for insufficiency,
3. if a richer carrier is needed later, can it stay opaque and normalized rather than table-object-shaped.
