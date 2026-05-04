*Posted by Codex agent on behalf of @govert*

# Notes for DNA OneCalc

Status: `active`
Owner lane: `OxFml W068`

## W068 Registry-Backed Function Help And Completion Proposals

OxFml has moved editor function-help and deterministic function-completion proposal sources to the OxFunc runtime registry surface:
1. built-in signatures come from `oxfunc_core::registry::builtin_registry()`,
2. host/UDF-aware callers can pass a mutated `FunctionRegistry`,
3. built-in function proposals also come from `builtin_registry()` when no library snapshot exists,
4. UDF entries registered into the supplied registry appear in function proposals,
5. capability tweaks flow through `CapabilityOverlay`,
6. capability-denied functions are filtered from deterministic completion proposals,
7. `LibraryContextSnapshot` remains an availability/admission/provenance overlay rather than a comprehensive function list.

Downstream cleanup now unblocked for DNA OneCalc after consuming this OxFml revision:
1. remove host-owned comprehensive default function-list wiring for OxFml function help,
2. stop supplying free-text arity or parameter-shape strings as function-help truth,
3. rely on absent `FunctionHelpPacket` for unknown registry entries rather than expecting OxFml fallback signatures,
4. remove any temporary `LibraryContextSnapshot` bridge whose only purpose is to populate function-name completion proposals,
5. route UDF registration changes through the OxFunc registry view supplied to OxFml.

Related handoffs:
1. `docs/handoffs/HANDOFF-DNAONECALC-003_W068_REGISTRY_BACKED_FUNCTION_HELP.md`
2. `docs/handoffs/HANDOFF-DNAONECALC-004_W068_REGISTRY_BACKED_COMPLETION_PROPOSALS.md`

Non-claim: this note does not claim DNA OneCalc has removed its local cleanup yet.

## W069 Format Engine Time, Fraction, And Accounting

OxFml has accepted `../DnaOneCalc/docs/HANDOFF_OXFML_FORMAT_ENGINE_TIME_FRACTION_ACCOUNTING.md` as a real OxFml-owned formatter gap.

Bounded OxFml support now covers:
1. time tokens such as `HH:mm:ss`,
2. AM/PM time rendering,
3. datetime composites such as `yyyy-mm-dd hh:mm:ss`,
4. elapsed-time tokens `[h]`, `[m]`, and `[s]`,
5. simple fraction codes such as `# ?/?`,
6. common accounting parentheses patterns.

Downstream cleanup now unblocked after consuming this OxFml revision:
1. remove `<NOT IMPLEMENTED>` markers for the bounded live time/datetime/fraction/accounting preset families,
2. keep markers for full custom-format grammar, text sections, and UI-specific accounting alignment until separate evidence lands.

Related handoff:
1. `docs/handoffs/HANDOFF-DNAONECALC-005_W069_FORMAT_ENGINE_TIME_FRACTION_ACCOUNTING.md`
