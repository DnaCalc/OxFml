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

Working rule:
1. the fixture harness may stand in for these host/coordinator-owned truths locally,
2. but the packet must still mark them as host/coordinator-supplied inputs rather than evaluator-owned meaning.

## First Reuse Goal
The first reuse goal is:
1. OxFunc adapter tests can use this packet as their deterministic host/coordinator stand-in,
2. later direct-host tests can use the same packet families,
3. later OxCalc-integrated tests can either reuse the packet directly or wrap it in a larger coordinator transport without changing semantic meaning.

## Current Open Questions For OxCalc
The next bounded OxCalc round should answer:
1. is this the right first stand-in packet for coordinator-owned truths in test artifacts,
2. should `RegisteredExternalProvider` stay present in the fixture packet from the start even if the first OxFunc wave keeps `CALL` / `REGISTER.ID` deferred,
3. should validated candidate/commit/reject packet capture be part of the same stand-in fixture packet or remain a separate host/runtime projection layer,
4. does OxCalc want any additional identity or acknowledgment fields before this packet is useful for later TreeCalc-facing integration tests.

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
