*Posted by Codex agent on behalf of @govert*

# HANDOFF-DNAONECALC-004: W068 Registry-Backed Completion Proposals

Status: `filed`
Source repo/workset: `OxFml/W068 follow-up`
Target repo/workset: `DnaOneCalc/TBD`
Filed date: `2026-05-04`
Related inbound: `../DnaOneCalc/docs/HANDOFF_OXFML_COMPLETION_PROPOSALS_FROM_REGISTRY.md`

## Purpose

Record the OxFml-side landing for DNA OneCalc's follow-up observation that editor completion proposals still treated `LibraryContextSnapshot` as a function-name catalog after function help had moved to the OxFunc registry.

## OxFml Landing Summary

OxFml editor completion proposals now use the same canonical registry source as editor function help:

1. `CompletionRequest` carries `FunctionRegistry` and optional `CapabilityOverlay`,
2. `EditorEnvironment::new(...)` supplies OxFunc `builtin_registry()` by default,
3. hosts can pass a UDF-mutated registry through `EditorEnvironment::with_function_registry(...)`,
4. capability-denied registry entries are filtered from deterministic completion proposals,
5. proposal `documentation_ref` comes from `FunctionEntry.registry_metadata.interface_contract_ref`,
6. `LibraryContextSnapshot` is no longer used as a function-name source for completion proposals.

`LibraryContextSnapshot` still contributes pinned snapshot identity, availability summaries, admission/provenance overlay data, and intelligent-completion context where those are the intended packet responsibilities.

## Downstream Cleanup Now Unblocked

DNA OneCalc can remove the temporary `library_context_snapshot_from_registry()` bridge whose only purpose was to feed OxFml function-name proposals.

For the built-in-only editor surface, DNA OneCalc should be able to construct:

```rust
EditorEnvironment::new(BindContext::default())
```

and still receive built-in function proposals such as `SUM`, `SUMIF`, `SUMIFS`, `SUMPRODUCT`, and `SUBSTITUTE` for the `=SU` prefix.

For UDF-aware editor surfaces, DNA OneCalc should pass the host-mutated OxFunc `FunctionRegistry` into `EditorEnvironment`; registered UDFs then appear in completion proposals through the same path.

For capability-profile tweaks, DNA OneCalc should pass a `CapabilityOverlay`; denied functions are not proposed, while function help for an explicitly typed denied function can still expose registry signature metadata with availability state.

## Evidence

Relevant OxFml files:

1. `crates/oxfml_core/src/language_service/mod.rs`
2. `crates/oxfml_core/src/consumer/editor/mod.rs`
3. `crates/oxfml_core/tests/language_service_tests.rs`
4. `crates/oxfml_core/tests/editor_consumer_facade_tests.rs`

Focused evidence:

1. default editor environment, no library snapshot, `=SU` includes registry-backed built-ins,
2. pinned snapshot omitting `SUMIF` still produces `SUMIF` from the registry,
3. UDF registry mutation surfaces `MYFUNC` in function proposals,
4. `CapabilityOverlay` denial filters `RTD` from `=R` proposals,
5. editor facade evidence now separates registry-backed completion from pinned-snapshot function-help overlay behavior.

## Non-Claims

This handoff does not claim:

1. DNA OneCalc has already removed the temporary bridge,
2. broad UDF execution semantics are finished,
3. external intelligent completion has been frozen as a shared host/coordinator packet.

Those remain downstream-owned after DNA OneCalc consumes this OxFml revision.
