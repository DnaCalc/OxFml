# OxFml Fixture Host And Coordinator Stand-In Packet

## Purpose
Define the first bounded OxFml-side packet for deterministic fixture hosts and coordinator stand-ins used by integration artifacts such as the OxFunc adapter wave.

This packet is not the production OxCalc coordinator API.
It is the smallest honest stand-in host packet that can:
1. drive real OxFml parse, bind, semantic-plan, and evaluation work,
2. stand in for coordinator-owned truths where needed,
3. remain compatible with the converged first-slice OxFml/OxCalc host/runtime packet.

## Why This Exists
The new OxFunc adapter and seam-fixture work under `W049` and `W050` cannot be purely OxFunc-facing.

Some of the required fixture inputs are actually stand-ins for host or coordinator-owned truths, including:
1. caller anchor,
2. direct cell bindings,
3. defined-name bindings,
4. table metadata,
5. typed host-query and provider surfaces,
6. runtime library-context snapshot selection.

OxFml therefore needs a clear rule for what the test harness may stand in for locally without pretending to have already frozen the full OxCalc coordinator API.

## Boundary Position
The fixture host packet should:
1. be sufficient to drive the current first-application seam and first host slice,
2. reuse the same semantic packet families already converged in the host/runtime contract,
3. stay deterministic and machine-readable,
4. make OxCalc-owned versus OxFml-owned truth explicit.

The fixture host packet should not:
1. replace the production OxCalc coordinator API,
2. silently widen current host/runtime ownership beyond the converged first slice,
3. collapse typed host or provider outcomes into mock-only shortcuts that hide real seam meaning.

## First Packet Families
The first stand-in fixture packet should be composed from the current converged host/runtime families.

### 1. Formula slot facts
1. `fixture_input_id`
2. optional `formula_slot_id`
3. `formula_text`
4. `formula_channel`
5. `caller_anchor`
6. optional `active_selection_anchor`
7. structure-context identity or version

### 2. Binding-world facts
1. `cell_fixture`
2. optional `defined_name_bindings`
3. optional `table_catalog`
4. optional `enclosing_table_ref`
5. optional `caller_table_region`

### 3. Typed host/query facts
1. optional `ReferenceResolver`
2. optional `HostInfoProvider`
3. optional `RtdProvider`
4. optional `RegisteredExternalProvider`
5. `LocaleFormatContext`
6. scalar context:
   - `now_serial`
   - `random_provider`
   - date-system context

### 4. Runtime catalog facts
1. `library_context_snapshot_ref`
2. a local or pinned `LibraryContextProvider`

### 5. Generic host formula context facts
For non-WorksheetA1 host channels such as the first TreeCalc-facing OxCalc W051 slice, the stand-in packet may also carry a generic host formula context.

Planned packet fields:
1. `dialect_id`
2. `capability_profile_id`
3. host reference parser or bind hook identity
4. host namespace resolver identity
5. function registry view identity
6. caller context identity
7. host namespace version and resolution-rule version
8. registry snapshot identity
9. table-context identity when structured references are admitted

The stand-in packet may fixture these fields, but it must still mark them as host/coordinator-supplied truth. OxFml owns only the formula grammar, source spans, lexical scope, bind diagnostics, prepared identity, and evaluator consequences that consume the context.

The host reference bind output should be projection-friendly for later OxCalc integration:
1. host reference handle or formal reference id
2. source span and source token identity
3. opaque selector payload
4. resolution layer
5. shape hint
6. caller-context dependency flag
7. replay-visible diagnostics
8. prepared-identity/cache contribution

TreeCalc selectors such as child/member paths remain opaque host payloads. Fixture data may name them for readability, but OxFml must not depend on their syntax or model meaning.

## Ownership Rule
For the fixture packet, ownership should be read as:

### OxFml-owned
1. parse, bind, semantic-plan, and evaluator meaning,
2. candidate, commit, reject, trace, and typed effect meaning,
3. packet projection into OxFunc-facing preparation and evaluation artifacts.

### Host/OxCalc-owned, even when fixture-backed
1. caller location and selection context,
2. direct cell and defined-name bindings,
3. table metadata and enclosing-table truth,
4. host-query answers and typed capability denial,
5. RTD and registered-external provider behavior,
6. runtime library-context selection and snapshot drift policy.
7. host namespace resolution, host reference selector payloads, TreeCalc model lowering, set-membership dependency edges, and runtime reader materialization.

Working rule:
1. the fixture harness may stand in for these host/coordinator-owned truths locally,
2. but the packet must still mark them as host/coordinator-supplied inputs rather than evaluator-owned meaning.

## First Reuse Goal
The first reuse goal is:
1. OxFunc adapter tests can use this packet as their deterministic host/coordinator stand-in,
2. later direct-host tests can use the same packet families,
3. later OxCalc-integrated tests can either reuse the packet directly or wrap it in a larger coordinator transport without changing semantic meaning.

## CALC-005 Receiving Plan
OxCalc `HANDOFF-CALC-005` is accepted into this packet as a planning addendum, not as TreeCalc syntax inside OxFml.

Current disposition:
1. `W051` owns the generic host formula context and host-reference bind-output packet shape,
2. `W074` owns the Excel oracle matrix for built-in/UDF/defined-name/defined-name-LAMBDA shadowing and cache invalidation,
3. TreeCalc host names and lambda-valued nodes map to Excel defined-name-like lanes unless later evidence forces a documented TreeCalc extension,
4. explicit host-reference syntax may bypass function-name ambiguity through the host namespace resolver,
5. OxFunc must see only ordinary values, arrays, callable carriers, or opaque reference-like carriers plus resolver authority.

Pending evidence:
1. Excel behavior for built-in, UDF, workbook-defined-name, sheet-defined-name, and defined-name `LAMBDA` collisions in bare call and non-call positions,
2. invalidation behavior when UDF registration/unregistration or defined-name add/remove/reclassification changes classification,
3. deterministic OxFml bind/replay artifacts for the selected first host namespace cases,
4. invalidation behavior when registry snapshot, structure context, host namespace version, caller context identity, table context, or resolution-rule version changes prepared identity,
5. DNA OneCalc guardrail evidence that no-host-reference `LET` / `LAMBDA` lexical variables, callable locals, captures, and returned lambdas stay OxFml-internal.

## Current Open Questions For OxCalc
The next bounded OxCalc round should answer:
1. is this the right first stand-in packet for coordinator-owned truths in test artifacts,
2. should `RegisteredExternalProvider` stay present in the fixture packet from the start even if the first OxFunc wave keeps `CALL` / `REGISTER.ID` deferred,
3. should validated candidate/commit/reject packet capture be part of the same stand-in fixture packet or remain a separate host/runtime projection layer,
4. does OxCalc want any additional identity or acknowledgment fields before this packet is useful for later TreeCalc-facing integration tests.
5. for W051, what exact explicit host-reference syntax enters the OxFml host hook and what source-span/source-token identity should be preserved,
6. what stable host namespace version, caller context identity, and selector-handle identity OxCalc can provide before first W051 implementation,
7. whether the first TreeCalc reference-collection carrier can expose a reference-preserving resolver/reader path or must begin with an explicitly labeled eager materialization fallback.

## Current OxCalc Intake
OxCalc's latest reply is now convergent on this packet direction.

Current accepted reading:
1. yes, this is the right first bounded stand-in packet for deterministic integration artifacts,
2. yes, `RegisteredExternalProvider` may stay present as an optional stand-in field from the start,
3. yes, candidate/commit/reject capture should remain a separate projection layer,
4. no, this does not freeze the production OxCalc coordinator API.

Current accepted packet refinements:
1. add a stand-in packet identity or `fixture_input_id`,
2. keep explicit structure-context identity,
3. allow explicit `formula_slot_id` when the same packet is reused across multiple formula-bearing slot families.

Current next step:
1. keep this packet narrow,
2. drive further change only from implementation reuse or concrete mismatch,
3. do not widen it into a broader coordinator API packet prematurely.
