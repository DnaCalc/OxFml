# HANDOFF_OXFUNC_002 — MultiArea Value Materialization Through OxFunc Resolver-Driven Combination

## Header
1. handoff id: `HANDOFF-OXFUNC-002`
2. date: `2026-04-07`
3. from repo: `OxFml`
4. to repo: `OxFunc`
5. related workset or feature id: `W059`

## Purpose
Request a seam narrowing so same-sheet `ReferenceKind::MultiArea` can be consumed in value-required lanes through OxFunc-owned combination semantics, using the existing `ReferenceResolver` as the dereference source.

This is the Style A direction discussed between repos:
1. OxFml and the host continue to provide single-reference resolution through the existing resolver,
2. OxFunc owns the semantic rule for how a `MultiArea` reference becomes a value-side payload when a caller needs values rather than preserved reference identity,
3. OxFml removes the remaining local same-sheet multi-area aggregation helper once this lands.

## Current OxFml Read
Current OxFml read of the boundary is:
1. `ReferenceKind::MultiArea` is now a real shared seam shape and should remain so,
2. `FUNC.OP_UNION_REF` should keep returning `EvalValue::Reference(ReferenceLike { kind: MultiArea, ... })`,
3. OxFml should not keep owning the semantic combination rule for multi-area-to-value materialization in the long term,
4. OxFml/host should continue to own workbook access and single-reference resolution,
5. OxFunc should own the multi-area combination semantics for value-required lanes.

## Problem Statement
Today the seam is only partially narrowed:
1. same-sheet multi-area reference identity is shared correctly,
2. reference-visible consumers can preserve and inspect `MultiArea` honestly,
3. but when a value-required lane needs concrete values from a `MultiArea`, OxFml still splits member targets locally, resolves each member area locally, and combines the results into an `EvalValue::Array`.

That remaining local helper is not a missing-type problem.
It is a remaining semantic-ownership problem.

## Requested Target Shape
OxFml wants OxFunc to support the following value-required flow for same-sheet `MultiArea`:

1. OxFml passes `CallArgValue::Reference(ReferenceLike { kind: MultiArea, ... })`,
2. OxFunc determines that the current lane requires value materialization,
3. OxFunc splits the multi-area carrier using `multi_area_targets()`,
4. OxFunc calls the supplied `ReferenceResolver` for each member target as a normal reference,
5. OxFunc combines the resolved member values under OxFunc-owned rules,
6. OxFunc returns the combined value payload to the caller.

The important ownership split is:
1. OxFml/host resolver: resolve one reference target,
2. OxFunc: define and execute the combination rule across the resolved members.

## Requested Seam Update
The requested seam update is conceptual, not necessarily a brand-new public type:
1. keep the current `ReferenceResolver::resolve_reference(&ReferenceLike) -> Result<EvalValue, RefResolutionError>`,
2. add OxFunc-side helper logic that accepts `ReferenceKind::MultiArea`,
3. for each member target, create a single-target `ReferenceLike` of the appropriate kind and resolve it through the existing resolver,
4. combine the resulting `EvalValue`s under OxFunc-owned rules before returning a prepared/value result.

OxFml does not need a new host callback if OxFunc can do this over the current resolver interface.

## Why Style A Is Preferred
Style A keeps the seam narrow and ownership-correct:
1. OxFml does not reimplement multi-area value semantics locally,
2. OxFunc owns the combination semantics alongside other semantic rules such as coercion and array lifting,
3. host/OxFml continue to own workbook-native dereference of individual references only,
4. the same logic becomes reusable for any OxFunc function/operator that needs values from a `MultiArea`.

## Expected OxFunc Implementation Shape
OxFml is not prescribing exact file layout, but expects something like:

1. a helper near adapters/resolution code that can materialize a `MultiArea` through the existing resolver,
2. that helper should be used by value-preparation paths rather than leaving OxFml to flatten and aggregate locally.

### Conceptual helper

```rust
fn resolve_multi_area_value(
    reference: &ReferenceLike,
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, CoercionError> {
    if !matches!(reference.kind, ReferenceKind::MultiArea) {
        return resolve_eval_value(resolver, reference).map_err(CoercionError::RefResolution);
    }

    let targets = reference.multi_area_targets().ok_or(
        CoercionError::RefResolution(RefResolutionError::ProviderFailure {
            detail: "invalid_multi_area_reference".to_string(),
        }),
    )?;

    let mut parts = Vec::with_capacity(targets.len());
    for target in targets {
        let member = ReferenceLike::new(ReferenceKind::Area, target).normalized();
        let value = resolve_eval_value(resolver, &member).map_err(CoercionError::RefResolution)?;
        parts.push(value);
    }

    combine_multi_area_member_values(&parts)
}
```

### Conceptual combination helper

```rust
fn combine_multi_area_member_values(parts: &[EvalValue]) -> Result<EvalValue, CoercionError> {
    // OxFunc-owned rule:
    // 1. arrays concatenate in member order
    // 2. scalars become 1x1 cells in that same order
    // 3. errors are preserved as cells
    // 4. references should not survive this lane; if they do, reject or recurse explicitly
}
```

The exact final function signatures may differ, but OxFml wants the ownership and behavior above.

## Likely OxFunc Touchpoints
Based on current local reading, OxFml expects the main OxFunc touchpoints to be:
1. `../OxFunc/crates/oxfunc_core/src/functions/adapters.rs`
2. `../OxFunc/crates/oxfunc_core/src/resolver.rs`
3. `../OxFunc/crates/oxfunc_core/src/value.rs`
4. any function-surface helpers that currently resolve `CallArgValue::Reference` into scalar/array prepared values

The primary place likely to matter is:
1. `prepare_arg_values_only(...)`
2. `expand_arg_values_only(...)`
3. any lookup/vector/aggregate preparation paths that currently resolve a reference once and normalize the result immediately

## Intended Semantic Rule
For this handoff, OxFml is asking OxFunc to own only the same-sheet multi-area combination rule.

Requested rule:
1. member order is preserved,
2. each member target is resolved independently through the resolver,
3. combined value payload is formed in OxFunc, not OxFml,
4. mixed-sheet multi-area is not part of this rule and should remain a distinct construct outside this admitted slice,
5. 3D references remain separate from multi-area and must not be merged into this logic.

## Requested Validation In OxFunc
OxFml requests deterministic evidence for at least:
1. `OP_UNION_REF(A1:A2,G1:G2)` returning `ReferenceKind::MultiArea`,
2. a value-preparation path consuming that `MultiArea` via the resolver and producing a combined value payload,
3. a mixed member case where one member resolves to an error cell and the combined payload preserves it,
4. proof that reference-visible lanes still preserve `MultiArea` identity and do not over-eagerly materialize it.

## Expected OxFml Follow-On Once Acknowledged
Once OxFunc lands this:
1. OxFml will remove the remaining local same-sheet multi-area value-materialization helper in `crates/oxfml_core/src/eval/mod.rs`,
2. OxFml will update `W059` and the seam docs so this is no longer carried as a local helper note,
3. OxFml will rerun the local floor and adjust the boundary report accordingly.

## Current OxFml Evidence
1. `crates/oxfml_core/src/eval/mod.rs`
2. `crates/oxfml_core/tests/evaluator_tests.rs`
3. `docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
4. `docs/worksets/W059_operator_semantic_dispatch_boundary_correction.md`
5. `docs/upstream/NOTES_FOR_OXFUNC.md`

## Current OxFunc Evidence OxFml Is Reading
1. `../OxFunc/crates/oxfunc_core/src/value.rs`
2. `../OxFunc/crates/oxfunc_core/src/resolver.rs`
3. `../OxFunc/crates/oxfunc_core/src/functions/adapters.rs`
4. `../OxFunc/crates/oxfunc_core/src/functions/operator_reference_family.rs`
5. `../OxFunc/crates/oxfunc_core/src/functions/surface_dispatch.rs`
