# W074: Registry Mutation And Name-Resolution Invalidation

## Purpose

Process OxFunc `HO-FN-014` by defining the OxFml work needed when the canonical function registry changes at runtime due to UDF registration, UDF unregistration, or capability overlay changes.

W068 closed the editor metadata source-of-truth cleanup. W074 is the broader formula binding, name-resolution, and cache-invalidation follow-up.

W074 is also the required evidence gate for OxCalc `HANDOFF-CALC-005` name/call precedence. It must identify Excel oracle behavior for built-ins, registered UDFs, workbook/sheet defined names, and defined-name `LAMBDA` invocation before OxFml freezes any generic host namespace shadowing rule for W051.

## Position and Dependencies

- **Depends on**: `W068`, OxFunc `W091`, OxFunc `W093`
- **Responds to**: `../OxFunc/docs/handoffs/HO-FN-014_udf_registry_mutation_and_name_resolution_invalidation.md`
- **Cross-repo**: OxFunc owns registry mutation and registry entry truth; OxFml owns formula parse/bind, name resolution, and cache invalidation. Also responds to OxCalc `HANDOFF-CALC-005` as the name/call precedence and invalidation counterpart to `W051`.

## Scope

### In Scope

1. Identify current bind, semantic-plan, editor-help, and host cache keys that need registry snapshot identity.
2. Define formula-call binding against an OxFunc registry view or immutable registry-derived snapshot for UDF-aware contexts.
3. Define `#NAME?` recovery after late UDF registration.
4. Define unregister and capability-denial invalidation for previously bindable formulas.
5. Document UDF-vs-defined-name precedence only after the Excel oracle matrix covers bare call and non-call positions.
6. Identify any OxFml-only metadata needed in an OxFunc `RegistryChangeSet`.
7. Reconcile bind-visible UDF registration with `REGISTER.ID` / `CALL` registered-external mutation.
8. Add deterministic tests for the first admitted invalidation and name-resolution cases.
9. Define how host namespace names from `W051` map onto Excel defined-name lanes, including lambda-valued host nodes, while recording any TreeCalc-only behavior as an explicit extension rather than an implicit precedence rule.

### Out of Scope

1. OxFunc registry implementation.
2. DNA OneCalc UI for UDF management.
3. Broad UDF execution semantics beyond first bind/name-resolution and invalidation evidence.
4. Moving workbook/sheet defined names into OxFunc.
5. Freezing TreeCalc-specific name/call precedence before Excel oracle evidence exists.

## Bead Set

### B074-01: Current Cache Inventory

- **Status**: planned
- **Owner**: OxFml
- **Effect**: inventory bind, semantic-plan, editor-help, runtime-host, and session cache artifacts that may reuse stale function-resolution facts.
- **Evidence target**: workset note records exact cache keys and invalidation boundaries.

### B074-02: Registry Snapshot Identity Model

- **Status**: planned
- **Owner**: OxFml with OxFunc coordination
- **Effect**: define the registry snapshot identity or registry-derived view identity that OxFml carries through bind and editor cache keys.
- **Evidence target**: first compile-checked packet fields or explicit blocker if OxFunc must expose additional identity data.

### B074-03: UDF-Aware Formula Binding

- **Status**: planned
- **Owner**: OxFml
- **Effect**: migrate formula-call binding for UDF-aware contexts from static built-in lookup to a registry view while preserving current built-in behavior.
- **Evidence target**: formula that is `#NAME?` before registration binds after registry mutation.

### B074-04: Name Precedence And Collision Rules

- **Status**: in_progress
- **Owner**: OxFml
- **Effect**: document and test precedence across built-ins, UDF registry entries, workbook/sheet defined names, defined-name `LAMBDA` values, helper-local names, and the `W051` host namespace lane mapped through defined-name-like behavior.
- **Evidence target**: Excel oracle matrix plus deterministic bind tests for selected collision and invalidation cases.

Required oracle cases before freeze:
1. built-in function name in call position and non-call bare-name position,
2. registered UDF name in call position and non-call bare-name position,
3. workbook-defined name and sheet-defined name collisions with built-ins,
4. workbook-defined name and sheet-defined name collisions with registered UDFs,
5. defined-name `LAMBDA` invocation by bare call and behavior when referenced in non-call position,
6. value-like, reference-like, and lambda-valued defined names with the same identifier across workbook/sheet scopes,
7. lexical `LET` / `LAMBDA` bindings colliding with built-ins, UDFs, and defined names,
8. late UDF registration changing an unresolved call into a bindable call,
9. UDF unregister and capability-denial changing a previously bindable call,
10. defined-name add/remove/reclassification changing non-call and call classification,
11. explicit host-reference syntax selecting a host object whose display name collides with a function, UDF, or defined name.

Current oracle-matrix shape:
1. each row must identify the source position as `call_callee`, `non_call_bare_name`, `let_lambda_lexical`, `explicit_host_reference`, or `structured_reference` for table-specific rows that are not ordinary name/call precedence claims,
2. each row must identify all visible candidates among `builtin_function`, `registered_udf`, `workbook_defined_name`, `sheet_defined_name`, `defined_name_lambda`, `lexical_local`, `host_namespace_name`, `table_name`, and `table_column`,
3. each row must record the Excel-observed winner, the observable value or error, whether a callable value remains callable after resolution, and which mutation inputs would invalidate the prepared identity,
4. defined-name `LAMBDA` rows must keep value reference and invocation behavior separate; a lambda-valued defined name is not assumed to be identical to a registered UDF,
5. lexical `LET` / `LAMBDA` rows are guardrail rows only: OxFml records their precedence against external name worlds, but lexical variables, callable locals, captures, and returned lambdas remain OxFml-internal and are not exposed as host namespace entries.

Current matrix artifact:
1. `docs/spec/formula-language/W074_CALC005_NAME_CALL_PRECEDENCE_ORACLE_MATRIX.csv`
   is the intake matrix for these cases,
2. rows with `oracle_status=planned_not_observed` are not evidence and do not
   freeze precedence,
3. promotion requires admissible black-box Excel observations to fill the
   winner/result/callable/identity columns.

Current observed tranche:
1. Excel COM 16.0 black-box probes on 2026-05-22 now cover selected built-in
   versus defined-name, UDF versus defined-name, sheet-versus-workbook
   defined-name, defined-name `LAMBDA`, lexical-local, late-UDF-registration,
   and UDF-removal rows in the matrix.
2. Deterministic non-Excel evidence now covers the explicit host-reference
   bypass row through OxCalc host-resolver/replay facts plus OxFml runtime and
   replay facade preservation of `resolution_layer=explicit_host_ref`, source
   token/span, opaque selector payload, and prepared identity inputs.
   A follow-on OxFml runtime/replay slice also proves that an opt-in generic
   host formula context with no explicit host-reference bind result still
   carries `host_namespace_version` through prepared identity and replay
   projection, and changing that version changes the prepared formula key.
   This is conservative host-context invalidation evidence only; it does not
   freeze bare host-name precedence.
3. Focused OxFml/OxFunc probes now cover capability-overlay denial at the
   registry/editor layer and the runtime formula-call layer: the denied
   registry entry remains present but unavailable, editor completion filters
   it, runtime semantic availability preserves the entry identity, execution
   blocks before ordinary built-in `SurfaceCallSite` dispatch, and replay
   projection carries the registry snapshot plus capability-denial identity.
   The same runtime tranche admits registered UDF calls as registry-present
   without implementing actual UDF invocation and returns to `#NAME?`-style
   unknown classification after unregister/default registry.
4. Table-context evidence now covers the W056 table-adjacent residuals needed
   by OxCalc without making TreeCalc-specific claims. Excel COM 16.0 probes on
   2026-05-22 observed the workbook defined-name/table-name collision row:
   table-created-first `Table1` rejects adding a same-named workbook defined
   name; a defined-name-created-first `Table1 = 99` can coexist with a
   ListObject renamed `Table1`; bare `=Table1` then resolves to the workbook
   defined name; and `Table1[Amount]`, `SUM(Table1[Amount])`, and
   `ROWS(Table1[Amount])` are rejected at formula authoring with `0x800A03EC`.
   Excel COM 16.0 build 20026 probes on 2026-05-23 add table-only bare/call
   classification, sheet-defined-name/table-name collision, table/column rename
   formula rewrite, and table/UDF collision evidence. Separately,
   non-collision structured syntax binds through the generic table-context
   packet, and local runtime/replay tests prove prepared-identity or bind-record
   identity coverage for table id, table range, row membership/order identity,
   exact header/totals refs, selected column id/ordinal/range, enclosing table,
   caller row, and unrelated catalog mutation as conservative table-context
   input. Broader full structured-reference grammar/table semantics remain
   W036 work, not W074 name/call freeze evidence.
5. The observed rows are provisional evidence for those row shapes only; they
   do not freeze the full name/call rule.
6. Name/call freeze remains blocked until the rows still marked partial or
   open are resolved, including formula-call registry/capability-overlay
   invalidation, host namespace mutation invalidation, broader
   workbook/sheet/UDF/defined-name scope combinations, and full structured
   table semantics outside the W056 table-context packet.

Current product-host mapping rule:
1. TreeCalc host names map to the closest Excel defined-name lane until this matrix proves a different extension is needed,
2. TreeCalc lambda-valued host nodes map to the closest Excel defined-name `LAMBDA` lane until evidence justifies a separate extension,
3. explicit host-reference syntax may bypass ordinary name/call ambiguity only through the generic host hook and must still emit replay-visible resolution-layer facts.

### B074-05: Unregister And Capability-Denial Invalidation

- **Status**: planned
- **Owner**: OxFml
- **Effect**: invalidate or reclassify previously bindable formulas when a UDF unregisters or capability overlay denies a function.
- **Evidence target**: deterministic transition test from bindable to `#NAME?` or capability-blocked state.

### B074-06: Registered-External Reconciliation

- **Status**: current scoped intake satisfied
- **Owner**: OxFml with OxFunc coordination
- **Effect**: preserve the distinction between bind-visible UDF metadata and descriptor-only `REGISTER.ID` / `CALL` registered-external mutation.
- **Evidence target**: spec or handoff note plus first non-regression test if local behavior changes.

Current reconciliation intake:
1. OxFunc W093 now agrees with the W046/W052 split: descriptor-only
   `REGISTER.ID` / `CALL` mutation is adjacent registered-external state, not
   ordinary UDF registration.
2. Plain worksheet `REGISTER.ID` creation does not create an editor
   completion entry, signature-help row, or bind-visible ordinary function
   entry.
3. A registered-external-backed ordinary UDF entry exists only when the host
   supplies friendly worksheet-visible metadata: stable surface name,
   arity/signature, callable metadata, source registration identity, and an
   invocation target descriptor.
4. Descriptor-only catalog mutation may produce targeted reevaluation with
   unchanged ordinary function-registry snapshot identity by default.
5. Bind-visible function registration/unregister remains the path that changes
   registry snapshot identity and can invalidate bind/editor artifacts.

This narrows the registered-external reconciliation item only. Source adapters,
formula-call registry lookup, broad UDF execution, and name/call precedence
freeze remain open W074/W093 work.

### B074-07: Generic Host Hook And Prepared Identity Inputs

- **Status**: in_progress
- **Owner**: OxFml with OxCalc input
- **Effect**: spell the generic host hook, host-reference bind result, structured-reference coexistence, and prepared-identity/cache invalidation inputs needed by `W051` without hardcoding TreeCalc syntax.
- **Evidence target**: updated spec and handoff surfaces naming required inputs and non-assumptions.

Required packet facts:
1. the host hook is generic and keyed by `dialect_id`, `capability_profile_id`, `resolution_rule_version`, host namespace version, registry snapshot identity, structure-context version, and caller context identity where relevant,
2. host-reference bind results carry a handle or formal reference id, source span/token text, opaque selector payload, resolution layer, shape hint, caller-context dependency, diagnostics, and replay identity,
3. table and structured-reference binding stays on the existing `table_catalog + enclosing_table_ref + caller_table_region` packet, with `TableDescriptor` carrying optional stable row membership/order identities and exact header/totals region refs; generic host hooks do not replace structured-reference grammar or table-context bind,
4. public structured-reference bind records carry `source_span_utf8`, exact `source_token_text`, stable bind-record handle, explicit-table versus omitted-table facts, resolved/effective table identity, selected columns/sections/regions, `uses_this_row` / caller-context dependence, resolved-reference descriptor, and typed diagnostic links for recognized structured-reference bind failures,
5. prepared identity and cache keys must include the name-world and host-context version inputs that can change resolution,
6. late UDF registration, unregister, capability-overlay denial, defined-name mutation, host namespace mutation, table context mutation, and resolution-rule changes are invalidation inputs when they can change bind or prepared-call shape.

Current runtime/replay evidence:
1. `RuntimeHostFormulaContext` and `RuntimeHostReferenceBindResult` are
   product-neutral runtime facade packets; the focused tests now use generic
   opaque host-reference labels rather than TreeCalc-shaped examples.
2. The same runtime and replay facade tests prove the DNA OneCalc no-host
   namespace guardrail with a returned `LAMBDA` that preserves a lexical
   capture through `LET` and invocation without producing host formula context
   or host-reference bind-result facts.
3. A focused runtime/replay identity slice now proves that when a caller opts
   into generic host formula context without any explicit host-reference bind
   result, `host_namespace_version` is still replay-visible and participates in
   prepared identity; changing it invalidates the prepared key while preserving
   empty host-reference bind results.
4. Runtime table-context evidence now proves that a structured-reference
   table-context mutation changing the selected column target changes the
   prepared formula identity and formal reference projection.
5. Runtime table-context evidence now proves that stable row membership/order
   identity changes affect `table_context_fingerprint` and prepared identity
   without changing resolved structured-reference behavior, and exact
   header/totals region refs change the resolved structured-reference identity
   for `#Headers` / `#Totals`.
6. Runtime/replay table-context evidence now proves the remaining W056 prepared
   identity inputs: table id, table range, selected column id, selected column
   ordinal, selected column range, unrelated catalog entries, enclosing table
   ref, and caller-table data-row offset either change the
   structured-reference bind record or the conservative
   `table_context_fingerprint`/prepared key and replay projection while
   preserving generic `TableDescriptor` ownership.
7. Public structured-reference bind records now project through `BoundFormula`,
   runtime prepared identity, runtime result, and formal-reference handles for
   explicit `Table1[Amount]`, omitted `[@Amount]`, `#Headers`, `#Totals`, and
   section-plus-column forms. This gives OxCalc a generic packet to consume
   without formula-text parsing.

## Status

- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - registry snapshot identity packet,
  - cache invalidation implementation,
  - formula-call registry lookup migration,
  - Excel oracle matrix for built-in/UDF/defined-name/LAMBDA shadowing,
  - mapping of `W051` host namespace names and lambda-valued host nodes to Excel defined-name lanes,
  - broader formula-call name/call precedence freeze beyond the bounded registry-view admission and capability-denial runtime classification tranche,
  - full structured-reference grammar/table semantics beyond stable packet facts, structured bind-record projection, structured-syntax disambiguation, and W056 prepared-identity mutation,
  - prepared identity/cache invalidation inputs for registry, structure, host namespace, table context, caller context, and resolution-rule changes,
  - evidence-backed cache invalidation for registry mutation and for bare host-name/host-namespace resolution beyond the conservative opt-in host-context identity slice.
