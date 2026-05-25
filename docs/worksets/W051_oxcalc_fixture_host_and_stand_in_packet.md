# W051: OxCalc Fixture Host and Stand-In Packet

## Purpose
Freeze the first bounded stand-in host/coordinator packet that OxFml can use in deterministic integration artifacts while keeping production OxCalc coordinator semantics out of scope.

This workset is also the OxFml owner for OxCalc `HANDOFF-CALC-005` on the generic host formula context needed by OxCalc W051. The accepted direction is a generic host context and namespace/reference hook, not a TreeCalc parser mode inside OxFml.

## Position and Dependencies
- **Depends on**: `W045`, `W049`, `W050`
- **Blocks**: none
- **Cross-repo**: bounded OxCalc seam round keyed to the reuse of host/coordinator-owned truths in OxFunc-facing adapter and fixture artifacts; inbound `HANDOFF-CALC-005` from OxCalc W051

## Scope
### In scope
1. Define the first deterministic stand-in host/coordinator packet for integration artifacts.
2. Make host/coordinator-owned versus OxFml-owned truth explicit in that packet.
3. Reuse current host/runtime packet families rather than inventing a separate ad hoc mock interface.
4. Initiate a bounded OxCalc note round on that packet.
5. Keep `CALL` / `REGISTER.ID` and broader production coordinator policy explicitly out of the first stand-in packet wave.
6. Define the OxFml-side plan for a generic `HostFormulaContext` that carries host namespace/reference authority without hardcoding TreeCalc syntax into OxFml.
7. Define the planned host-reference bind output and runtime transport requirements for host references as values or opaque reference-like carriers.
8. Route Excel-oracle-derived function/UDF/defined-name/LAMBDA name and call precedence to `W074` before any shadowing rule is frozen.

### Out of scope
1. Production OxCalc coordinator API freeze.
2. Full graph scheduler policy.
3. Full distributed/runtime ownership.
4. Deferred registered-external runtime beyond current first-wave fixture needs.
5. TreeCalc-specific reference syntax or selector semantics inside OxFml.
6. Final Excel name/call shadowing order before the `W074` oracle matrix has evidence.
7. OxCalc-owned TreeCalc reference-collection carriers, set-membership dependency edges, invalidation policy, or runtime reader materialization.

## Deliverables
1. A canonical first stand-in host/coordinator packet draft.
2. A bounded OxCalc note round keyed to that draft.
3. An explicit list of non-assumptions so fixture-host reuse does not get mistaken for coordinator API freeze.
4. A `HostFormulaContext` planning shape covering dialect/profile identity, host reference parse/bind hooks, host namespace resolution, function registry view, caller context, and version identity.
5. A host-reference bind-output plan covering handle, source span, opaque selector, resolution layer, shape hint, caller-context dependency, and diagnostics.
6. A runtime transport plan keeping OxFunc insulated from host syntax while preserving a `ReferenceLike` plus resolver path where function metadata permits reference visibility.

## Gate Model
### Entry gate
- `W045` has converged enough that host/runtime packet families are known.
- `W049` / `W050` have made it clear that some OxFunc-facing fixture inputs are actually stand-ins for OxCalc-owned host truths.

### Exit gate
- One canonical OxFml draft stand-in packet exists.
- OxCalc coordination has been initiated against that packet.
- Host/coordinator-owned truths versus OxFml-owned truths are explicit and non-collapsed.
- `HANDOFF-CALC-005` has an OxFml receipt and its W051/W074 ownership split is recorded.
- Generic host context clauses are present in the host/runtime and stand-in packet specs.
- Name/call shadowing remains explicitly evidence-gated on the `W074` Excel oracle matrix.

## CALC-005 Host Formula Context Plan

### Ownership

Primary OxFml owner: `W051`.

Related owner: `W074` for registry mutation, UDF-aware formula binding, defined-name/LAMBDA precedence, cache invalidation, and the Excel oracle matrix that must settle name/call shadowing.

OxCalc remains owner of TreeCalc model structure, TreeCalc host names, explicit host reference syntax, reference-collection carriers, set-membership dependency edges, invalidation over the TreeCalc model, and resolver/reader materialization.

OxFunc remains owner of built-in and UDF function semantics and the canonical function registry surface. OxFunc must not receive TreeCalc syntax or selector payloads.

### Planned `HostFormulaContext` Shape

The planned context shape is semantic rather than TreeCalc-specific:

1. `dialect_id` and `capability_profile_id`
2. declarative host syntax rules and a bind hook for host reference syntax in operand and explicit-host-reference positions
3. host namespace resolver for host names, paths, selectors, defined names, and host-sensitive references
4. OxFunc-backed function registry view for built-ins, registered UDFs, and capability overlays
5. caller context for relative references, caller-sensitive names, and lexical walk-up
6. version identity for prepared formula cache keys and replay, including host namespace version, structure context version, registry snapshot identity, caller context identity where relevant, and resolution rule version

OxFml owns calls, argument lists, operators, literals, arrays, `LET`, `LAMBDA`, lexical scopes, source spans, bind diagnostics, and prepared identity around that host hook.

### CALC-005 Boundary Correction

OxCalc-side formula-text recognition is a boundary defect. OxFml must parse the
formula and invoke a generic host hook; OxCalc must not rewrite authored formula
text into neutral tokens before parse/bind.

The host syntax hook is generic. It is not a TreeCalc parser mode. The hook can
be driven by declarative rules supplied by the host context, for example:

| Rule family | Pattern shape | OxFml responsibility | Host responsibility |
|---|---|---|---|
| explicit collection selector | `@CHILDREN`, `.*`, `<host-path>.@CHILDREN`, `<host-path>.*` | preserve token/span and emit an explicit host-reference packet | resolve base and collection membership |
| ordered selector | `@PRECEDING`, `@FOLLOWING`, `@ANCESTORS`, plus qualified `<host-path>.<selector>` forms | preserve selector/base/tail spans and emit a selector-family packet | resolve base, traversal order, bounds, and dependency facts |
| recursive selector | `**`, `**.<tail>`, `<host-path>.**`, `<host-path>.**.<tail>` | parse as host selector syntax only when declared by the host dialect | resolve recursive traversal and tail path |
| caller-relative path | `^`, `^.<tail>`, repeated `^`, `[]`, `[].<tail>` | distinguish declared host path syntax from ordinary operators using parser context | resolve caller-sensitive anchors and diagnostics |
| workspace-qualified path | `[workspace]<path>`, `!<path>`, bracket-escaped segments | emit host path token/span payloads, not TreeCalc semantics | resolve aliases, availability, canonical path, and invalidation |
| reference literal array | `{<host-ref>(,<host-ref>)*}` when the host dialect marks the literal as reference-only | retain element spans and reject ordinary scalar-array ambiguity through typed diagnostics | lower reference-only members or reject mixed scalar/reference arrays |
| node table structured reference | `<host-path>[...]`, `[...]` with enclosing table context | use the existing structured-reference grammar and produce generic table bind records plus any host-path payload | map node tables to generic descriptors and sparse readers |

The host hook output must preserve:

1. `source_span_utf8`,
2. exact source token text,
3. token kind or rule family,
4. handle/formal reference id,
5. opaque selector/path payload,
6. resolution layer,
7. shape hint,
8. caller-context dependency and optional caller-context identity,
9. typed diagnostics,
10. prepared identity inputs for host namespace, structure context, caller context, registry snapshot, capability overlay, table context, and resolution-rule version.

OxFml must not inspect TreeCalc topology, selectors, child/member sets, row ids,
column ids, or invalidation semantics. Those are OxCalc resolver outputs.

### Planned Host Reference Bind Output

The host reference bind output must carry:

1. host reference handle or formal reference id
2. source span plus source token identity or source text
3. active `dialect_id` and `capability_profile_id`
4. opaque host selector payload supplied by the host resolver
5. resolution layer such as `lexical`, `function`, `defined_name`, `host_name`, `explicit_host_ref`, or `unresolved`
6. shape hint such as `single`, `collection`, `dynamic`, or `unknown`
7. caller-context-dependent flag and caller context identity input when applicable
8. typed diagnostics for ambiguity, unresolved host name, capability denial, unknown function, and set/reference-as-callable mismatch

### Runtime Transport Rule

Host references may be materialized to values for values-only calls. Reference-sensitive or reference-preserving calls must have a path to receive an opaque `ReferenceLike` plus resolver/reader authority. Eager value-array materialization is permitted only as a fallback for a bounded compatibility slice and does not satisfy the reference-preserving scenario by itself.

### Evidence Gate

Name/call precedence is not frozen in this workset. `W074` must first identify and run the Excel oracle matrix for built-in functions, registered UDFs, workbook/sheet defined names, and defined-name `LAMBDA` invocation in both bare call and non-call positions. TreeCalc host names and lambda-valued nodes map to the closest Excel defined-name lane unless a future packet records a TreeCalc extension explicitly.

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
  - a canonical stand-in host/coordinator packet draft now exists and OxCalc has reviewed it as settled enough for deterministic fixture-host and first TreeCalc-facing integration reuse, but the packet is not yet frozen beyond the current narrow first wave
  - inbound `HANDOFF-CALC-005` is accepted into `W051` planning with `W074` as the required evidence gate for Excel name/call precedence
  - `HostFormulaContext`, host-reference bind output, and runtime reference transport are spec-planned but not yet exercised
  - final built-in/UDF/defined-name/LAMBDA shadowing remains pending Excel oracle evidence
  - the accepted identity refinements are now part of the packet direction:
    - `fixture_input_id`
    - structure-context identity
    - optional `formula_slot_id`
  - candidate / commit / reject capture remains intentionally separate from the stand-in input packet and that boundary is converged at note level, not yet shared seam-freeze text
  - broader coordinator API freeze and later packet reuse across wider slot families remain outside the current narrow stand-in packet
- claim_confidence: provisional
