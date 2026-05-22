*Posted by Codex agent on behalf of @govert*

# HANDOFF-CALC-005 OxFml Receiving Review

## Purpose
Record the OxFml-side receiving review of
`../OxCalc/docs/handoffs/HANDOFF_CALC_005_OXFML_HOST_CONTEXT_AND_NAMESPACE_RESOLUTION.md`.

This is a planning receipt and ownership update. It does not settle the Excel
name/call precedence rule and does not make TreeCalc syntax part of OxFml.

## Decision Summary
Decision: `accept_as_w051_plan_with_w074_evidence_gate`.

Accepted into OxFml canonical direction:
1. `W051` owns the generic `HostFormulaContext` and stand-in packet plan for
   OxCalc W051.
2. `W074` owns the Excel oracle matrix for built-in, registered-UDF,
   workbook/sheet defined-name, and defined-name `LAMBDA` precedence.
3. OxFml must remain generic around the host hook: calls, argument lists,
   operators, literals, arrays, `LET`, `LAMBDA`, lexical scopes, source spans,
   bind diagnostics, prepared identity, and semantic plans remain OxFml-owned.
4. OxCalc remains owner of TreeCalc model structure, host names, explicit host
   reference syntax, selector payloads, reference-collection carriers,
   set-membership dependency edges, invalidation over the TreeCalc model, and
   resolver/reader materialization.
5. OxFunc must not receive TreeCalc syntax or selector payloads.

## Clause Disposition Matrix
| Packet clause | Disposition | Owner | Notes |
|---|---|---|---|
| Generic host formula context | accepted_as_plan | W051 | Planned shape includes dialect/profile identity, host reference parse/bind hook, host namespace resolver, function registry view, caller context, and version identity. |
| Surrounding formula grammar remains OxFml-owned | accepted | W051/OxFml formula lane | Calls, arguments, operators, literals, arrays, `LET`, `LAMBDA`, source spans, diagnostics, and prepared identity stay in OxFml. |
| No TreeCalc parser mode in OxFml | accepted | W051/OxCalc | Host selectors remain opaque payloads from OxCalc. |
| Name/call shadowing hierarchy | evidence_gated | W074 | Do not freeze until Excel oracle evidence covers built-ins, registered UDFs, workbook/sheet defined names, and defined-name `LAMBDA` behavior. |
| Host names and lambda-valued nodes | accepted_as_mapping_default | W051/W074 | Map to the closest Excel defined-name lane unless a later packet documents a TreeCalc extension. |
| Explicit host-reference syntax bypass | accepted_as_plan | W051 | Explicit host paths/selectors bind through the host resolver and may intentionally select objects whose display names collide with functions. |
| Host reference bind output | accepted_as_plan | W051 | Planned fields: handle, source span, source token/text, opaque selector, resolution layer, shape hint, caller-context dependency, diagnostics. |
| Runtime transport | accepted_as_plan | W051 with OxFunc boundary | Values-only calls may materialize values; reference-sensitive calls need opaque `ReferenceLike` plus resolver/reader authority. |
| Registry and cache invalidation | accepted_and_routed | W074 | Registry snapshot identity, host namespace version, caller context identity, and structure context version must participate in bind/prepared identity where they affect resolution. |

## Required Excel Oracle Matrix
Before precedence promotion, `W074` must cover:
1. built-in function name in call position and non-call bare-name position,
2. registered UDF name in call position and non-call bare-name position,
3. workbook-defined name and sheet-defined name collisions with built-ins,
4. workbook-defined name and sheet-defined name collisions with registered UDFs,
5. defined-name `LAMBDA` invocation by bare call and behavior when referenced
   in non-call position,
6. value-like, reference-like, and lambda-valued defined names with the same
   identifier across workbook and sheet scopes,
7. lexical `LET` / `LAMBDA` bindings colliding with built-ins, UDFs, and
   defined names,
8. late UDF registration, UDF unregister, capability-denial, defined-name
   mutation, and host namespace mutation as cache-invalidation triggers,
9. explicit host-reference syntax selecting a host object whose display name
   collides with a function, UDF, or defined name.

Matrix row contract:
1. rows must distinguish `call_callee`, `non_call_bare_name`,
   `let_lambda_lexical`, and `explicit_host_reference` source positions,
2. rows must list all competing candidates visible in the scenario:
   built-in function, registered UDF, workbook-defined name, sheet-defined
   name, defined-name `LAMBDA`, lexical local, and host namespace name,
3. rows must record the observed winner, value/error/callable outcome,
   invalidation inputs, and whether a prepared identity changes after the
   mutation,
4. defined-name `LAMBDA` invocation and defined-name `LAMBDA` value-reference
   behavior must be separate rows,
5. `LET` / `LAMBDA` lexical rows are guardrail rows only. OxFml must keep
   lexical variables, callable locals, captures, and returned lambdas internal
   to OxFml rather than exposing them as host namespace or OxCalc objects.

## Generic Host Hook And Bind Result
The `W051` host hook accepted by this receipt remains product-neutral.

Required host-context inputs:
1. `dialect_id`,
2. `capability_profile_id`,
3. `resolution_rule_version`,
4. host namespace version,
5. registry snapshot identity,
6. structure-context version,
7. caller context identity where resolution is caller-sensitive.

Host-reference bind results must be replay-visible and include:
1. host reference handle or formal reference id,
2. source span plus source token/text,
3. opaque selector payload,
4. resolution layer,
5. shape hint,
6. caller-context dependency flag,
7. typed diagnostics,
8. replay/cache identity contribution.

Structured references stay on the existing table-context lane:
`table_catalog + enclosing_table_ref + caller_table_region`. The generic host
hook does not replace table-name parsing, structured-reference binding, or
table-name-versus-defined-name disambiguation.

Prepared identity and cache invalidation must include every supplied identity
that can change bind or prepared-call shape: registry snapshot, structure
context, host namespace version, caller context identity, table-context identity,
and resolution-rule version.

## OxCalc Shape Changes Needed Before W051 Implementation
OxFml needs OxCalc to supply or confirm:
1. stable `dialect_id` and `capability_profile_id` values for the TreeCalc
   formula channel,
2. the exact explicit host-reference syntax that enters the host hook and the
   source-span/source-token identity OxFml should preserve,
3. a host namespace resolver contract that returns opaque selectors, stable
   handles, resolution layer, shape hint, caller-context dependency, and typed
   diagnostics,
4. a host namespace version and caller context identity suitable for prepared
   identity and cache invalidation,
5. the first TreeCalc reference-collection carrier and whether it can expose a
   reference-preserving resolver/reader path before any eager materialization
   fallback,
6. set-membership dependency and invalidation facts owned by OxCalc, kept out
   of OxFml semantics but correlated to OxFml host-reference handles.

## Canonical Docs Updated
1. `docs/worksets/W051_oxcalc_fixture_host_and_stand_in_packet.md`
2. `docs/worksets/W074_registry_mutation_and_name_resolution_invalidation.md`
3. `docs/spec/OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md`
4. `docs/spec/OXFML_FIXTURE_HOST_AND_COORDINATOR_STANDIN_PACKET.md`
5. `docs/spec/formula-language/OXFML_NAME_WORLD_AND_RUNTIME_REGISTRATION_INVALIDATION.md`
6. `docs/spec/formula-language/EXCEL_FORMULA_LANGUAGE_CONCRETE_RULES.md`
7. `docs/spec/formula-language/EXCEL_FORMULA_LANGUAGE_CONFORMANCE_MATRIX.csv`
8. `docs/IN_PROGRESS_FEATURE_WORKLIST.md`
9. `docs/handoffs/HANDOFF_REGISTER.csv`

## Pending Evidence
Behavior that remains pending evidence:
1. built-in/UDF/defined-name/defined-name-LAMBDA shadowing in bare call
   position,
2. built-in/UDF/defined-name/defined-name-LAMBDA behavior in non-call bare-name
   position,
3. exact defined-name `LAMBDA` value behavior when used as a value versus
   invoked as a callee,
4. cache invalidation after UDF registration, UDF unregister, capability
   overlay denial, defined-name mutation, and host namespace mutation,
5. reference-preserving host transport through `ReferenceLike` plus resolver,
6. deterministic replay artifacts for selected host namespace and explicit
   host-reference collision cases,
7. table-context mutation and structured-reference bind invalidation when the
   table packet changes name/column/current-row meaning,
8. DNA OneCalc no-host-reference LET/LAMBDA lexical guardrail evidence showing
   that lexical variables, callable locals, captures, and returned lambdas do
   not become host-reference or host-namespace bindings.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- reviewed inbound observations: `../OxFunc/docs/upstream/NOTES_FOR_OXFML.md`
  and `../OxCalc/docs/upstream/NOTES_FOR_OXFML.md`
- open_lanes:
  - `W074-CALC005` Excel oracle matrix,
  - `W051` public host-context and bind-output packet spelling,
  - prepared identity/cache invalidation inputs for registry, structure,
    table context, host namespace, caller context, and resolution-rule changes,
  - DnaOneCalc no-host-reference LET/LAMBDA lexical guardrail,
  - OxCalc TreeCalc reference-collection and resolver/reader carrier,
  - deterministic replay evidence for host namespace resolution and
    invalidation.
