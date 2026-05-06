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

## Status Axes

- execution_state: acknowledged
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - W074 workset creation and execution,
  - shared registry-change packet shape,
  - formula-call registry lookup migration,
  - cache invalidation evidence.
