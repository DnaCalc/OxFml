# OXFML_REGISTERED_EXTERNAL_PROVIDER_AND_CALL_REGISTER_ID_BOUNDARY.md

## Purpose
This note captures OxFml's current bounded runtime boundary for worksheet `CALL` / `REGISTER.ID`, host-driven external registration, and runtime unregister so the next OxFml <-> OxFunc and OxFml <-> OxCalc rounds can narrow concrete packets instead of reopening broad host/runtime theory.

## Ownership Split
Current OxFml reading:
1. OxFunc owns the built-in function/operator catalog and the semantic truth for built-in surface admission and execution.
2. OxFunc also owns runtime catalog mutation semantics for registered externals:
   - initial built-in catalog population,
   - runtime registration,
   - runtime unregister,
   - descriptor truth used by worksheet `CALL`.
3. OxFml owns formula parsing, bind classification, typed request normalization, and worksheet-visible consequence classification.
4. OxCalc or a direct host owns higher-level external-library policy, security policy, and source-specific registration initiation.
5. OxFml should therefore expose typed runtime packets that let a host funnel registration intent into OxFunc-owned catalog mutation rather than mutating catalog truth locally.

## Registration Channels
Current OxFml reading is that registered external functions may enter the OxFunc-owned catalog through three channels that all converge on the same OxFunc-owned mutation seam:
1. worksheet `REGISTER.ID`
   - initiated from formula evaluation,
   - resolved through `RegisteredExternalProvider::resolve_register_id(...)`,
   - yields descriptor truth used later by worksheet `CALL`,
2. host API registration
   - initiated by a host-side API call,
   - funneled through OxFml as a `RegisteredExternalCatalogMutationRequest::Register(...)`,
   - preserves richer host hints such as display/help text or execution profile,
3. VBA shim registration
   - initiated after host-owned VBA project loading,
   - funneled through the same OxFml mutation packet as host API registration,
   - preserves source-project, source-module, and source-procedure provenance.

Current unregister rule:
1. unregister should be the same bounded mutation seam,
2. OxFml should preserve the initiating channel and stable registration identity,
3. OxFunc should remain the owner of resulting catalog truth and snapshot-generation effects.

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
5. `RegisteredExternalCatalogMutationRequest`
6. `RegisteredExternalCatalogMutationResult`
7. `RegisteredExternalCatalogController`

Current OxFml reading:
1. these are runtime request/result packets, not merely library-context snapshot metadata,
2. they should therefore cross the seam directly where the host/runtime path needs them,
3. OxFml should adopt the OxFunc-owned request/result packet types directly rather than wrapping them in a second OxFml-local vocabulary when no extra OxFml-owned fields are required,
4. the current local host and adapter path now does this for `RegisterIdRequest`, `RegisteredExternalCallRequest`, and `RegisteredExternalDescriptor`,
5. normalized worksheet `REGISTER.ID` and `CALL` packets should therefore be visible in OxFml artifacts as first-class packet facts, not only implied by provider callbacks,
6. the runtime library-context snapshot may still carry capability and profile truth about whether worksheet `CALL` / `REGISTER.ID` is admitted or gated in a given environment,
7. but the snapshot/provider layer should not be the only place where per-request registration, invocation, and unregister packets can be observed.
8. current local OxFml host support now exposes:
   - `SingleFormulaHost::recalc_with_registered_external_provider(...)`
   - `SingleFormulaHost::apply_registered_external_catalog_mutation(...)`
   as the first internal host-facing surface for that packet family.
9. current local OxFml trace and adapter artifacts also expose normalized request packets through `PreparedCall` for:
   - worksheet `REGISTER.ID`
   - worksheet `CALL` direct-library targets
   - worksheet `CALL` register-id targets

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
4. the same descriptor-driven reading should govern general worksheet-to-external type conversion rather than treating coercion as an OxFml-owned pre-flattening step.

## First Packet Shape
Current OxFml-side freeze:

### `RegisterIdRequest`
1. `library_name`
2. `procedure`
3. `declared_type_text`
4. `caller_anchor` and `host_execution_profile` stay adjacent OxFml host/adaptor facts rather than widening the shared OxFunc-owned request packet

### `RegisteredExternalDescriptor`
1. `stable_registration_id`
2. `register_id`
3. `origin_kind`
4. `display_name`
5. `library_name`
6. `procedure`
7. `declared_type_text`
8. if more argument-policy facts are needed, OxFunc should extend this shared descriptor upstream and OxFml should adopt that extension directly rather than forking a sibling wrapper

### `RegisteredExternalCallRequest`
1. adopt OxFunc's current packet directly:
   - `target`
   - `invocation_args`
2. `target` remains:
   - `RegisterId(f64)`
   - `Direct(RegisterIdRequest)`
3. OxFml-owned caller-anchor and host execution profile facts remain adjacent host/adaptor packet facts, not reasons to fork the underlying call-request type

### `RegisteredExternalProvider`
1. `register_id`
2. `describe_registration`
3. `invoke_registered`
4. `invoke_direct`

### `RegisteredExternalCatalogMutationRequest`
1. `Register`
   - `registration_channel`
   - `register_id_request`
   - optional `stable_registration_id_hint`
   - optional `display_name_hint`
   - optional `help_text_hint`
   - optional VBA source provenance
   - optional `host_execution_profile`
2. `Unregister`
   - `registration_channel`
   - `stable_registration_id`
   - optional `host_execution_profile`

### `RegisteredExternalCatalogMutationResult`
1. `RegisterApplied`
   - `descriptor`
   - optional `host_execution_profile`
2. `UnregisterApplied`
   - `stable_registration_id`
   - optional `host_execution_profile`

### `RegisteredExternalCatalogController`
1. host-facing OxFml funnel surface that applies a typed mutation packet into OxFunc-owned catalog mutation logic,
2. not a claim that OxFml owns catalog mutation semantics,
3. intended to preserve the initiating channel while OxFunc remains the owner of catalog truth.

## Current Non-Claims
This note does not claim:
1. that the final host/coordinator safety model is frozen,
2. that OxFunc and OxCalc have both acknowledged the local freeze above as shared seam text yet,
3. that OxFunc must consume raw native-library details or own external invocation,
4. that VBA project loading policy belongs in OxFml,
5. that runtime snapshot-generation side effects for register/unregister are already frozen beyond the current best-effort ownership split.

## Current OxFml Reply Direction
If OxFunc asks for a current best-effort answer, OxFml's reply is:
1. keep `RegisteredExternalProvider` separate from `HostInfoProvider`,
2. carry `RegisterIdRequest`, `RegisteredExternalDescriptor`, `RegisteredExternalCallRequest`, and typed catalog-mutation packets directly in the bounded runtime packet,
3. prefer direct adoption of those OxFunc-owned request/result packet types rather than a parallel OxFml wrapper vocabulary,
4. expose normalized `REGISTER.ID` and `CALL` packets in OxFml artifacts so adapter and host evidence can prove the seam shape directly,
5. let OxFunc use registration metadata or direct-call metadata to decide reference dereference and general type coercion at the worksheet `CALL` boundary,
6. treat built-in catalog truth and runtime registered-external catalog mutation as OxFunc-owned,
7. treat host API registration and VBA shim registration as host-initiated channels that OxFml funnels into the same OxFunc-owned mutation seam,
8. keep the library-context snapshot/provider lane for admission/profile truth rather than as the sole carrier of live registration/invocation packets,
9. keep worksheet `CALL` runtime above OxFunc except for request normalization, descriptor-driven argument handling, and worksheet-visible result projection unless concrete evidence forces a narrower split.
