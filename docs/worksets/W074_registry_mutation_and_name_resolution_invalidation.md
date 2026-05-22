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
1. each row must identify the source position as `call_callee`, `non_call_bare_name`, `let_lambda_lexical`, or `explicit_host_reference`,
2. each row must identify all visible candidates among `builtin_function`, `registered_udf`, `workbook_defined_name`, `sheet_defined_name`, `defined_name_lambda`, `lexical_local`, and `host_namespace_name`,
3. each row must record the Excel-observed winner, the observable value or error, whether a callable value remains callable after resolution, and which mutation inputs would invalidate the prepared identity,
4. defined-name `LAMBDA` rows must keep value reference and invocation behavior separate; a lambda-valued defined name is not assumed to be identical to a registered UDF,
5. lexical `LET` / `LAMBDA` rows are guardrail rows only: OxFml records their precedence against external name worlds, but lexical variables, callable locals, captures, and returned lambdas remain OxFml-internal and are not exposed as host namespace entries.

Current matrix artifact:
1. `docs/spec/formula-language/W074_CALC005_NAME_CALL_PRECEDENCE_ORACLE_MATRIX.csv`
   is the planned intake matrix for these cases,
2. rows with `oracle_status=planned_not_observed` are not evidence and do not
   freeze precedence,
3. promotion requires admissible black-box Excel observations to fill the
   winner/result/callable/identity columns.

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

- **Status**: planned
- **Owner**: OxFml with OxFunc coordination
- **Effect**: preserve the distinction between bind-visible UDF metadata and descriptor-only `REGISTER.ID` / `CALL` registered-external mutation.
- **Evidence target**: spec or handoff note plus first non-regression test if local behavior changes.

### B074-07: Generic Host Hook And Prepared Identity Inputs

- **Status**: in_progress
- **Owner**: OxFml with OxCalc input
- **Effect**: spell the generic host hook, host-reference bind result, structured-reference coexistence, and prepared-identity/cache invalidation inputs needed by `W051` without hardcoding TreeCalc syntax.
- **Evidence target**: updated spec and handoff surfaces naming required inputs and non-assumptions.

Required packet facts:
1. the host hook is generic and keyed by `dialect_id`, `capability_profile_id`, `resolution_rule_version`, host namespace version, registry snapshot identity, structure-context version, and caller context identity where relevant,
2. host-reference bind results carry a handle or formal reference id, source span/token text, opaque selector payload, resolution layer, shape hint, caller-context dependency, diagnostics, and replay identity,
3. table and structured-reference binding stays on the existing `table_catalog + enclosing_table_ref + caller_table_region` packet; generic host hooks do not replace structured-reference grammar or table-context bind,
4. prepared identity and cache keys must include the name-world and host-context version inputs that can change resolution,
5. late UDF registration, unregister, capability-overlay denial, defined-name mutation, host namespace mutation, table context mutation, and resolution-rule changes are invalidation inputs when they can change bind or prepared-call shape.

## Status

- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - registry snapshot identity packet,
  - formula-call registry lookup migration,
  - cache invalidation implementation,
  - cross-repo registry change-set shape,
  - Excel oracle matrix for built-in/UDF/defined-name/LAMBDA shadowing,
  - mapping of `W051` host namespace names and lambda-valued host nodes to Excel defined-name lanes,
  - generic host hook and host-reference bind-result packet spelling,
  - prepared identity/cache invalidation inputs for registry, structure, host namespace, table context, caller context, and resolution-rule changes,
  - DNA OneCalc no-host-reference LET/LAMBDA lexical guardrail,
  - evidence-backed cache invalidation for registry and host namespace mutation.
