*Posted by Codex agent on behalf of @govert*

# HANDOFF-OXFUNC-005: W068 Canonical Registry Consumption Landing

Status: `filed`
Source repo/workset: `OxFml/W068`
Target repo/workset: `OxFunc/TBD`
Filed date: `2026-05-04`
Related inbound: `../OxFunc/docs/handoffs/HO-FN-011_canonical_function_registry_consumption.md`

## Purpose

Record that OxFml has consumed the OxFunc canonical runtime function registry interface from `HO-FN-011` for editor function-help metadata.

This is primarily a landing confirmation and contract-dependency note. It is not a request for a new OxFunc implementation lane unless OxFunc intends to change the registry API shape.

## OxFml Landing Summary

OxFml `W068` now treats `oxfunc_core::registry` as the function metadata source for editor function help:

1. built-in editor help uses `builtin_registry()`,
2. callers may pass a host/UDF-mutated `FunctionRegistry` through `EditorEnvironment`,
3. capability tweaks may be supplied through `CapabilityOverlay`,
4. display signatures use `FunctionEntry.display_signature.signature_display`,
5. argument labels use ordered `ParameterDescriptor` rows,
6. arity bounds use `FunctionEntry.meta.arity`,
7. unknown registry lookup returns no `FunctionHelpPacket`.

`LibraryContextSnapshot` remains an availability, admission, provenance, and replay overlay. It no longer carries or supplies function-help signature truth.

## Removed OxFml Paths

OxFml removed:

1. `LibraryContextSnapshotEntry.arity_shape_note`,
2. the editor `parse_arity_shape_note` path,
3. synthetic signature suffix generation,
4. synthetic `argN` / `additional_args` argument-help generation,
5. fixture and export/import fields that existed only to carry free-text arity.

OxFml retained registry-derived fallback mapping for equivalent metadata fields exposed by `FunctionEntry.registry_metadata`, including surface identity and interface-classification fields.

## Evidence

Relevant OxFml evidence:

1. `crates/oxfml_core/src/consumer/editor/mod.rs`
2. `crates/oxfml_core/src/semantics/mod.rs`
3. `crates/oxfml_core/tests/language_service_tests.rs`
   - built-in registry signatures: `NOW`, `SUM`, `IF`
   - unknown callee returns no help packet
   - capability overlay preserves registry signature metadata
   - UDF registry mutation updates editor help
   - structural no-synthesis guard
4. `docs/worksets/W068_canonical_function_registry_consumption_cleanup.md`

Validation run in OxFml:

1. `cargo fmt --all -- --check`
2. `git diff --check` with line-ending warnings only
3. source guard for removed synthesis tokens under `crates/oxfml_core/src`
4. `cargo test -p oxfml_core --test language_service_tests`
5. `cargo test -p oxfml_core`

Landing commit: `712ca21 Consume canonical function registry for editor help`

## OxFunc Dependencies OxFml Now Relies On

OxFml now depends on the following OxFunc registry surfaces continuing to be the canonical path for function-help metadata:

1. `builtin_registry()`,
2. `FunctionRegistry::lookup_by_surface_name(...)`,
3. `FunctionRegistry::register_udf(...)`,
4. `FunctionRegistry::unregister_udf(...)`,
5. `FunctionRegistry::with_capability_overlay(...)`,
6. `FunctionEntry.display_signature`,
7. `SignatureForm.parameters`,
8. `ParameterDescriptor`,
9. `FunctionEntry.meta.arity`,
10. `FunctionEntry.registry_metadata`,
11. `CapabilityOverlay`.

## Non-Asks

OxFml is not asking OxFunc to:

1. restore or expose `arity_shape_note`,
2. add a parallel registry just for OxFml,
3. change UDF execution semantics as part of this handoff,
4. change `LibraryContextSnapshot` ownership.

## Requested OxFunc Response

Please acknowledge whether the landed OxFml consumption matches the intended `HO-FN-011` contract.

If OxFunc expects a near-term API rename or metadata-field reshaping, file a narrow follow-up handoff before making that change so OxFml and DNA OneCalc can avoid reintroducing host-side function-list shims.
