*Posted by Codex agent on behalf of @govert*

# HO-FN-013 Registry Function Meta Owned Runtime Shape Ack

Status: `acknowledged`
Direction: OxFml -> OxFunc
Responds to: `../OxFunc/docs/handoffs/HO-FN-013_registry_function_meta_owned_runtime_shape.md`
Source workset: `OxFunc/W091`
OxFml owner workset: `W068`
Acknowledged date: 2026-05-06

## Acknowledgement

OxFml acknowledges the current OxFunc registry shape:

1. `FunctionEntry.meta` is `RegistryFunctionMeta`,
2. `FunctionEntry.meta.function_id` is owned runtime data,
3. `FunctionEntry.meta.arity` is the registry arity source,
4. `FunctionEntry.display_signature` and ordered `ParameterDescriptor` rows remain the signature/help source,
5. `FunctionEntry.registry_metadata` remains the source for seam/status metadata projection.

## Local Check

OxFml editor help and completion code consumes the current registry entry shape directly. The ordinary source does not reintroduce:

1. `LibraryContextSnapshotEntry.arity_shape_note`,
2. `parse_arity_shape_note`,
3. synthetic `signature_suffix`,
4. synthetic `build_argument_help`,
5. synthetic `argN` / `additional_args` parameter help.

Static built-in `FunctionMeta` remains used in non-registry formula semantics paths where OxFunc still exposes built-in function metadata. Registry consumers do not type-assume `FunctionMeta`.

## Status Axes

- execution_state: acknowledged
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: none for HO-FN-013; broader registry mutation and bind invalidation remains HO-FN-014 / W074.
