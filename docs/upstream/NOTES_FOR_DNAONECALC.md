*Posted by Codex agent on behalf of @govert*

# Notes for DNA OneCalc

Status: `active`
Owner lane: `OxFml W068`

## W068 Registry-Backed Function Help

OxFml has moved the editor function-help source to the OxFunc runtime registry surface:
1. built-in signatures come from `oxfunc_core::registry::builtin_registry()`,
2. host/UDF-aware callers can pass a mutated `FunctionRegistry`,
3. capability tweaks flow through `CapabilityOverlay`,
4. `LibraryContextSnapshot` remains an availability/admission/provenance overlay rather than a comprehensive function list.

Downstream cleanup now unblocked for DNA OneCalc after consuming this OxFml revision:
1. remove host-owned comprehensive default function-list wiring for OxFml function help,
2. stop supplying free-text arity or parameter-shape strings as function-help truth,
3. rely on absent `FunctionHelpPacket` for unknown registry entries rather than expecting OxFml fallback signatures,
4. route UDF registration changes through the OxFunc registry view supplied to OxFml.

Non-claim: this note does not claim DNA OneCalc has removed its local cleanup yet.
