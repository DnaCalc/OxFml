*Posted by Codex agent on behalf of @govert*

# HO-FN-014 UDF Registry Mutation And Name-Resolution Invalidation Ack

Status: `acknowledged`
Direction: OxFml -> OxFunc
Responds to: `../OxFunc/docs/handoffs/HO-FN-014_udf_registry_mutation_and_name_resolution_invalidation.md`
Source workset: `OxFunc/W093`
OxFml owner workset: `W074`
Acknowledged date: 2026-05-06

## Acknowledgement

OxFml acknowledges the HO-FN-014 ownership split:

1. OxFunc owns callable function registry entries and UDF mutations.
2. OxFml owns formula parse/bind, name resolution, editor cache, and host cache invalidation.
3. Workbook and sheet defined names remain document/formula environment state, not OxFunc registry state.
4. `REGISTER.ID` / `CALL` descriptor mutation remains distinct from ordinary bind-visible UDF registration unless the host supplies worksheet-visible UDF metadata.

## Initial OxFml Response

The needed OxFml follow-up is opened as planned workset `W074`.

W074 must cover:

1. registry snapshot identity in bind/editor cache keys,
2. formula-call binding against registry views for UDF-aware contexts,
3. `#NAME?` recovery after late UDF registration,
4. unregister and capability-denial invalidation,
5. UDF-vs-defined-name precedence,
6. OxFml-only metadata needs for any OxFunc `RegistryChangeSet`,
7. registered-external reconciliation.

## 2026-05-22 W093 Reconciliation Intake

OxFml has now processed the OxFunc W093 registered-external reconciliation.

Current scoped agreement:

1. descriptor-only `REGISTER.ID` / `CALL` mutation remains adjacent
   registered-external state, not ordinary UDF function registration,
2. plain worksheet `REGISTER.ID` does not create editor completion,
   signature-help, or bind-visible ordinary function entries,
3. registered-external-backed ordinary UDF entries require friendly
   worksheet-visible metadata from the host,
4. descriptor-only change sets may preserve ordinary registry snapshot identity
   and drive targeted reevaluation,
5. bind-visible function registration/unregister remains the registry snapshot
   identity path for bind/editor invalidation.

This narrows the registered-external reconciliation item. Formula-call registry
lookup migration, cache invalidation evidence, source adapters, broad UDF
execution, and name/call precedence remain W074/W093 follow-up.

## Status Axes

- execution_state: acknowledged
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - W074 workset creation and execution,
  - formula-call registry lookup migration,
  - cache invalidation evidence.
