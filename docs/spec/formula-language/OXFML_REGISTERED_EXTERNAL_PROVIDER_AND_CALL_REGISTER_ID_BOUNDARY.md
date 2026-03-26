# OXFML_REGISTERED_EXTERNAL_PROVIDER_AND_CALL_REGISTER_ID_BOUNDARY.md

## Purpose
This note captures OxFml's current best-effort boundary for worksheet `CALL` / `REGISTER.ID` runtime so the next OxFml <-> OxFunc round can narrow a concrete packet instead of reopening broad host/runtime theory.

## Ownership Split
Current OxFml reading:
1. OxFml owns formula parsing, bind classification, typed request normalization, and worksheet-visible consequence classification.
2. OxCalc or a direct host owns registration-handle allocation, external-library policy, actual native invocation, and runtime safety policy.
3. OxFunc should remain below that host/runtime boundary except for:
   - normalized request consumption where applicable,
   - worksheet-visible result projection,
   - any function-semantic rules that can be expressed without owning external invocation.

## Provider Separation
`RegisteredExternalProvider` should remain separate from `HostInfoProvider`.

Why:
1. `HostInfoProvider` serves typed worksheet host facts such as `INFO` and `CELL`.
2. `RegisteredExternalProvider` is about registration and external invocation lifecycle, not host-info query semantics.
3. Collapsing them would blur capability, safety, and runtime-policy boundaries that should stay explicit.

## First Bounded Typed Packet
The first bounded consumer model should carry direct typed runtime lanes:
1. `RegisterIdRequest`
2. `RegisteredExternalDescriptor`
3. `RegisteredExternalCallRequest`
4. `RegisteredExternalProvider`

Current OxFml reading:
1. these are runtime request/result packets, not merely library-context snapshot metadata,
2. they should therefore cross the seam directly where the host/runtime path needs them,
3. the runtime library-context snapshot may still carry capability and profile truth about whether worksheet `CALL` / `REGISTER.ID` is admitted or gated in a given environment,
4. but the snapshot/provider layer should not be the only place where per-request registration and invocation packets can be observed.

## Reference And Conversion Reading
Current OxFml reading should align worksheet `CALL` more closely with the built-in function seam:
1. OxFml already preserves reference-visible versus pre-dereferenced argument lanes for built-ins based on OxFunc-owned argument-preparation policy,
2. worksheet `CALL` should follow the same principle,
3. if a registered external target or direct `{ library, procedure, type_text }` call requires reference-sensitive or conversion-sensitive handling, OxFml should not eagerly flatten that into one generic value lane,
4. OxFunc should be able to consult registration metadata or direct call metadata to decide:
   - whether a reference argument must remain reference-visible,
   - whether a reference should be dereferenced before native invocation,
   - which general type coercions apply at the worksheet-to-external boundary.

Current implication:
1. `RegisteredExternalDescriptor` must be rich enough for OxFunc to see argument-policy-relevant registration facts,
2. the bounded runtime packet must let OxFunc obtain that descriptor for register-id targets and direct-call targets,
3. OxFml should preserve reference-visible prepared arguments where the descriptor may require them, rather than hard-coding one eager dereference rule in OxFml.

## Suggested First Packet Shape
Current best-effort OxFml packet split:

### `RegisterIdRequest`
1. `library_name`
2. `procedure_name`
3. optional `type_text`
4. `caller_anchor`
5. optional `host_execution_profile`

### `RegisteredExternalDescriptor`
1. `register_id`
2. `library_name`
3. `procedure_name`
4. optional `type_text`
5. `descriptor_state`

### `RegisteredExternalCallRequest`
1. `target_kind`
   - `RegisterId`
   - `DirectLibraryProcedure`
2. optional `register_id`
3. optional `library_name`
4. optional `procedure_name`
5. optional `type_text`
6. `normalized_arguments`
7. `caller_anchor`
8. optional `host_execution_profile`
9. optional `descriptor_ref`

### `RegisteredExternalProvider`
1. `register_id`
2. `describe_registration`
3. `invoke_registered`
4. `invoke_direct`

## Current Non-Claims
This note does not claim:
1. that worksheet `CALL` / `REGISTER.ID` is already admitted into the current `W049` / `W050` adapter wave,
2. that the final host/coordinator safety model is frozen,
3. that the exact field names above are canonically frozen,
4. that OxFunc must consume raw native-library details or own external invocation.

## Current OxFml Reply Direction
If OxFunc asks for a current best-effort answer, OxFml's reply is:
1. keep `RegisteredExternalProvider` separate from `HostInfoProvider`,
2. carry `RegisterIdRequest`, `RegisteredExternalDescriptor`, and `RegisteredExternalCallRequest` directly in the bounded runtime packet,
3. let OxFunc use registration metadata or direct-call metadata to decide reference dereference and general type coercion at the worksheet `CALL` boundary,
4. keep the library-context snapshot/provider lane for admission/profile truth rather than as the sole carrier of live registration/invocation packets,
5. keep worksheet `CALL` runtime above OxFunc except for request normalization, descriptor-driven argument handling, and worksheet-visible result projection unless concrete evidence forces a narrower split.
