*Posted by Codex agent on behalf of @govert*

# HO-FN-011 Canonical Function Registry Consumption Ack

## Direction
- **From**: `OxFml`
- **To**: `OxFunc`
- **Responds to**: `../OxFunc/docs/handoffs/HO-FN-011_canonical_function_registry_consumption.md`
- **Source workset**: `OxFunc/W091`
- **OxFml owner workset**: `W068`

## Acknowledgement
OxFml accepts the new interface direction: function identity, arity, display signatures, parameter descriptors, help text, and source classification are OxFunc registry truth.

OxFml must not maintain a comprehensive function list, must not parse host-supplied free-text arity notes for ordinary function help, and must not synthesize parameter names such as `arg1` or `additional_args` when registry truth is absent.

## Local Migration Rule
The `W068` cleanup moves directly to the desired end state.

Required local consequences:
1. editor function help and signature help consume `oxfunc_core::registry` entries or immutable registry-derived views,
2. `LibraryContextSnapshotEntry.arity_shape_note` is removed rather than deprecated,
3. `parse_arity_shape_note`, `signature_suffix`, and `build_argument_help` are deleted,
4. unknown functions produce no invented help packet,
5. capability and provider availability remain an overlay/view concern and do not justify a host-owned catalog fork,
6. UDF registration is treated as runtime registry mutation rather than a separate host function list.

## OxFml-Only Fields
No OxFml-only fields are required on `FunctionEntry` before the first registry-backed migration.

Existing OxFml snapshot/status fields remain useful only as admission, capability, provenance, or replay-facing overlay facts. They should be populated from registry metadata where they mirror function identity or source truth, not from a parallel OxFml or host-maintained catalog.

## Validation Expectation
`W068` must include deterministic evidence for:
1. `NOW()` zero-argument signature display,
2. `SUM` and `IF` parameter descriptors from OxFunc registry truth,
3. no function-help packet for an unknown callee,
4. UDF signature display through a registry-mutated view,
5. structural absence of the old string-arity synthesis path.

## Cross-Repo Note
DNA OneCalc can remove its local default function-name list and host-filled comprehensive snapshot after OxFml lands the `W068` registry-backed editor/runtime cleanup.
