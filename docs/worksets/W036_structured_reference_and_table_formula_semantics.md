# W036: Structured Reference and Table Formula Semantics

## Purpose
Realize the structured-reference and table-formula lanes classified by `W031` so OxFml has an explicit parser, binder, evaluator, and proving-host floor for table-aware formula semantics rather than only a provisional acceptance rule.

## Position and Dependencies
- **Depends on**: `W031`, `W032`
- **Blocks**: `W038`, later broader table/host-policy work
- **Cross-repo**: OxFml owns grammar, bind, runtime, and seam-significant table semantics; OxCalc consumes resulting effects and OxFunc consumes prepared-call/reference consequences

## Scope
### In scope
1. Broaden structured-reference grammar and qualifier coverage.
2. Define table-context-sensitive bind and normalized reference shapes.
3. Exercise local evaluator/runtime behavior for the first nontrivial structured-reference families.
4. Classify formula-significant table surfaces that affect bind meaning, admission, or seam-significant effects.
5. Add deterministic replay/proving artifacts for the widened structured-reference floor.
6. Freeze the intended host/coordinator table-context packet and the intended narrow OxFunc-facing downstream seam for structured-reference resolution.
7. Compare the first local floor directly against the current `MS-OE376` structure-reference requirements already extracted into the Foundation reference library.

### Out of scope
1. Every Excel table feature.
2. UI-only table styling.
3. Broad conditional-formatting or data-validation sublanguage work.

## Deliverables
1. Narrower canonical structured-reference and table-formula semantics in OxFml docs.
2. Wider parser/binder/evaluator coverage with deterministic replay evidence.
3. Explicit residual list for remaining table lanes.
4. Explicit seam split:
   - host/OxCalc owns tables and table metadata,
   - OxFml owns structured-reference grammar, bind, and evaluator consequences,
   - OxFunc consumes only normalized reference or dereferenced prepared-argument consequences.
5. First implementation-facing table-context packet for direct host and OxCalc-integrated host use.
6. First deterministic structured-reference test matrix covering syntax, bind, disambiguation, row context, and replay-visible identity preservation.

## Gate Model
### Entry gate
- `W031` has classified the structured-reference lane as partial rather than implicitly complete.
- `W032` has kept provider/catalog pressure from distorting reference semantics.

### Exit gate
- Structured references are no longer only a provisional parse-acceptance lane.
- Table-context-sensitive bind meaning is canonical and replay-backed for the exercised local floor.
- Remaining table semantics are explicitly listed rather than implied.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | |
| 2 | Conformance matrix rows updated? | |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | |
| 4 | Cross-repo impact assessed and handoff filed if needed? | |
| 5 | All required tests pass? | |
| 6 | No known semantic gaps remain in declared scope? | |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | |

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - broader qualifier unions and totals/header/data mixed-selector breadth are still outside the exercised local floor
  - the current local evaluator floor uses bind-resolved structured targets rather than a broader runtime table subsystem
  - replay/proving-host fixtures for structured-reference identity preservation are still narrower than the eventual full `W036` floor
  - OxCalc has now confirmed `table_catalog + enclosing_table_ref + caller_table_region` as the right first semantic packet, but that packet is still not promoted as shared seam-freeze text and broader workbook-table closure remains deferred
  - OxFunc and OxCalc have the refined packet shape in notes, but there is not yet a mismatch-driven acknowledgment round on the newer code floor
- claim_confidence: draft

## Structured-Reference Seams
Current intended split for `W036`:
1. OxCalc or the direct host owns tables as workbook objects, including:
   - table identity,
   - range,
   - column catalog,
   - header/totals presence,
   - enclosing-table and caller-row context,
2. OxFml owns:
   - structured-reference grammar,
   - omitted-table-name and `#This Row` interpretation,
   - disambiguation against defined names,
   - normalization into `StructuredRef`,
   - evaluator/FEC consequences once table context is supplied,
3. OxFunc should receive:
   - ordinary dereferenced scalar/array inputs,
   - or preserved reference-like prepared args,
   but not a workbook-table object model unless a concrete insufficiency proves the need.

Current draft canonical pointer:
1. `docs/spec/formula-language/OXFML_STRUCTURED_REFERENCE_AND_TABLE_BOUNDARY.md`

## Spec-Derived Minimum Requirements
The current `MS-OE376` review and extracted Foundation conformance candidates require `W036` to preserve at least these truths:
1. if the `table-name` is omitted, the formula containing the structured reference must be in a cell that belongs to a table, and that enclosing table name is the effective table name,
2. `table-name` must denote a real table rather than being silently treated as a defined name or other workbook object,
3. `#This Row` is a real current-row-sensitive selector lane,
4. `#This Row` must not be combined with `#Headers`, `#Total Row`, `#Data`, or `#All`,
5. qualifier-plus-column combinations such as `[[#All],[Amount]]` are not reducible to plain bracket sugar and must survive bind honestly,
6. structured-reference bind meaning depends on explicit host-owned table context, not on parser-only inference.

## First Host And Coordinator Packet
The first packet to freeze under `W036` should be small and explicit.

Required top-level host/coordinator fields:
1. `table_catalog`
2. `enclosing_table_ref`
3. `caller_table_region`

Required `table_catalog` descriptor fields:
1. `table_id`
2. `table_name`
3. `workbook_scope_ref`
4. `sheet_scope_ref`
5. `table_range_ref`
6. `row_membership_identity`
7. `row_order_identity`
8. `header_region_ref`
9. `totals_region_ref`
10. `header_row_present`
11. `totals_row_present`
12. `columns`

Required column fields:
1. `column_id`
2. `column_name`
3. `ordinal`
4. `column_range_ref`

Required `caller_table_region` fields:
1. `table_id`
2. `region_kind`
3. `data_row_offset`

Required structured-reference bind record fields for downstream consumers:
1. `bind_record_handle`
2. `source_span_utf8`
3. exact `source_token_text`
4. `explicit_table_name` and `omitted_table_name`
5. resolved/effective table identity where bind succeeds
6. `selected_column_ids`
7. selected section qualifiers and selected region descriptors
8. `uses_this_row` / `caller_context_dependent`
9. resolved-reference descriptor
10. typed diagnostic links for recognized structured-reference bind failures

Working rule:
1. direct hosts and OxCalc-integrated hosts should present the same semantic table packet,
2. OxCalc may wrap or correlate that packet with broader workbook/coordinator identities,
3. OxFml should not reconstruct omitted-table-name or current-row semantics from workbook globals once this packet exists.

## OxFunc-Facing Packet Rule
The intended first OxFunc-facing rule is narrow:
1. OxFunc consumes ordinary prepared values or preserved reference-like prepared arguments,
2. a structured reference may remain visible downstream only as an opaque normalized reference intent or preserved-reference lane already owned by OxFml,
3. OxFunc should not ingest a table catalog, current-row context, or workbook table object model for first support,
4. any richer OxFunc carrier must be justified by a concrete function-semantic insufficiency found after the first evaluator floor exists.

## Execution Plan
`W036` should execute in this order:
1. freeze the host/OxCalc table-context packet and update the host-runtime contract to reference it explicitly,
2. broaden syntax coverage for:
   - `Table1[Amount]`
   - `[@Amount]`
   - `Table1[[#All],[Amount]]`
   - first section selectors `#Headers`, `#Data`, `#Totals`, `#All`, `#This Row`,
3. implement bind-time disambiguation:
   - table-name versus defined-name collision,
   - omitted-table-name without enclosing table,
   - illegal `#This Row` combinations,
4. normalize the first bound `StructuredRef` floor with stable table and column identities,
5. wire the first evaluator lanes:
   - reference-preserved aggregate consumer,
   - current-row-sensitive scalar lane,
6. add deterministic replay and proving-host evidence preserving:
   - table identity,
   - column identity,
   - enclosing-table dependence,
   - caller-row-sensitive meaning,
7. only then decide whether any narrower OxFunc seam change is actually needed.

## First Execution Slice
The recommended first exercised slice is:
1. parse and bind `Table1[Amount]`,
2. parse and bind `[@Amount]` with explicit enclosing-table context,
3. parse and bind `Table1[[#All],[Amount]]`,
4. prove table-name versus defined-name disambiguation,
5. evaluate one reference-preserved lane such as `SUM(Table1[Amount])`,
6. evaluate one current-row-sensitive lane such as `=[@Amount]*2`,
7. add deterministic replay fixtures preserving table identity and caller-row-sensitive truth.

Current exercised local floor:
1. bind and normalize `Table1[Amount]` into a first `StructuredRef`,
2. bind and normalize `[@Amount]` with explicit enclosing-table and caller-row context,
3. reject omitted-table-name forms without enclosing table context as typed bind failures,
4. reject illegal `#This Row` combinations at bind,
5. disambiguate structured-reference syntax against colliding defined names,
6. bind and normalize section-only selectors such as `Table1[#Headers]` across all table columns,
7. bind and normalize multi-column section selectors such as `Table1[[#All],[Amount]:[Tax]]` and `Table1[[#Data],[Amount]:[Tax]]`,
8. evaluate one explicit-column aggregate lane through the host path,
9. evaluate one current-row-sensitive scalar lane through the host path,
10. evaluate first section-only header/totals lanes through the host path,
11. evaluate one section-qualified multi-column aggregate lane through the host path.

## Edge-Case Plan
The required edge-case plan for this workset is:
1. malformed bracket nesting,
2. omitted-table-name without enclosing table context,
3. `#This Row` versus explicit column selector distinction,
4. totals/header/data qualifier combinations,
5. table-name versus defined-name collision,
6. replay-visible preservation of table identity and row-context-sensitive meaning,
7. first spill or scalarization interaction only where the existing pipeline already forces a distinction.
8. `#This Row` combined illegally with `#Headers`, `#Total Row`, `#Data`, or `#All`,
9. multi-column range selectors such as `[Amount]:[Tax]`,
10. totals-row and headers/data union selectors where the spec treats them as distinct unions rather than plain aliases.

## Test Plan
The first deterministic test matrix should be split into:
1. syntax acceptance/rejection
   - valid bracket and qualifier forms,
   - malformed bracket nesting and malformed qualifier combinations,
2. bind classification
   - explicit table name,
   - omitted table name with enclosing table,
   - omitted table name without enclosing table,
   - table-name versus defined-name collision,
3. normalized-reference identity
   - stable `table_id`,
   - stable `selected_column_ids`,
   - explicit `caller_row_sensitive` marker,
4. evaluator behavior
   - aggregate over explicit column reference,
   - current-row-sensitive arithmetic lane,
   - qualifier-sensitive section selection,
5. replay/proving-host evidence
   - retained identity of table and column selections,
   - retained caller-row dependence,
   - explicit failure classification when required table context is missing.

Current local evidence added in this slice:
1. `crates/oxfml_core/tests/structured_reference_tests.rs`
2. first host packet consumption through `SingleFormulaHost`
3. section-only `#Headers` / `#Totals` and multi-column `#All` / `#Data` bind coverage
4. regression validation against `crates/oxfml_core/tests/parse_bind_fixture_tests.rs`
5. public structured-reference bind records on `BoundFormula`, runtime prepared
   identity, runtime result, and formal-reference projection for explicit
   `Table1[Amount]`, omitted `[@Amount]`, `#Headers`, `#Totals`, and
   section-plus-column forms.
