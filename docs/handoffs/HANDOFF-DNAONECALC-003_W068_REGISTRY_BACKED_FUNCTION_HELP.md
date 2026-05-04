*Posted by Codex agent on behalf of @govert*

# HANDOFF-DNAONECALC-003: W068 Registry-Backed Function Help

Status: `filed`
Source repo/workset: `OxFml/W068`
Target repo/workset: `DnaOneCalc/TBD`
Filed date: `2026-05-04`
Related inbound: `../DnaOneCalc/docs/HANDOFF_OXFML_FUNCTION_HELP_FROM_OXFUNC_REGISTRY.md`

## Purpose

Record the OxFml-side landing that unblocks DNA OneCalc removal of host-owned comprehensive function-list wiring for OxFml editor function help.

## OxFml Landing Summary

OxFml editor function help now uses OxFunc registry metadata rather than a host-filled function list:

1. built-in signatures come from `oxfunc_core::registry::builtin_registry()`,
2. UDF-aware hosts can pass a mutated `FunctionRegistry` into `EditorEnvironment`,
3. capability tweaks can be passed with `CapabilityOverlay`,
4. display signatures come from `FunctionEntry.display_signature.signature_display`,
5. argument labels come from ordered `ParameterDescriptor` rows,
6. unknown registry entries produce no `FunctionHelpPacket`,
7. `LibraryContextSnapshot` remains an availability/admission/provenance overlay only.

Landing commit: `712ca21 Consume canonical function registry for editor help`

## Downstream Cleanup Now Unblocked

DNA OneCalc can remove local function-help compensation paths that:

1. maintain a comprehensive default function list for OxFml,
2. send free-text arity or parameter-shape strings to OxFml,
3. expect OxFml to synthesize fallback signatures such as `arg1` or `additional_args`,
4. treat `LibraryContextSnapshot` as the source of parameter or arity truth.

For built-in-only hosts, no explicit registry handoff is required because OxFml defaults to OxFunc `builtin_registry()`.

For UDF-aware hosts, DNA OneCalc should supply an OxFunc `FunctionRegistry` view that has already applied the relevant `register_udf(...)` / `unregister_udf(...)` mutations, then construct `EditorEnvironment` with that registry.

For capability-profile tweaks, DNA OneCalc should provide a `CapabilityOverlay` rather than removing functions from the registry or inventing host-local replacement entries.

## Expected Consumer Behavior

After consuming this OxFml revision:

1. `=NOW(` help should display `NOW()` with no synthetic argument rows,
2. `=SUM(` help should display the OxFunc registry signature and parameter names,
3. `=IF(TRUE,` should mark the active parameter using registry descriptor order,
4. `=ZZZNOTAFUNCTION(` should have no `FunctionHelpPacket`,
5. a registered UDF should display the UDF registry signature and parameter names,
6. gated or capability-denied known functions may still show registry signature metadata while availability/deferred state remains explicit.

## Evidence

Relevant OxFml evidence:

1. `crates/oxfml_core/src/consumer/editor/mod.rs`
2. `crates/oxfml_core/tests/language_service_tests.rs`
3. `docs/worksets/W068_canonical_function_registry_consumption_cleanup.md`

Validation run in OxFml:

1. `cargo fmt --all -- --check`
2. `git diff --check` with line-ending warnings only
3. source guard for removed synthesis tokens under `crates/oxfml_core/src`
4. `cargo test -p oxfml_core --test language_service_tests`
5. `cargo test -p oxfml_core`

## Non-Claims

This handoff does not claim:

1. DNA OneCalc has already removed its local function-list wiring,
2. broad UDF execution semantics are finished,
3. DNA OneCalc UI rendering behavior has been validated locally in OxFml.

Those remain downstream-owned after DNA OneCalc consumes this OxFml revision.
