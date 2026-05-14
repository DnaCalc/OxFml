# HANDOFF-CALC-003 OxFml Receiving Review

## Purpose
Record the OxFml-side receiving review of
`../OxCalc/docs/handoffs/HANDOFF_CALC_003_OXFML_NUMERICAL_REDUCTION_AND_ERROR_ALGEBRA.md`.

This is a receiving acknowledgement and integration-dependency update. It does
not end the OxCalc integration dependency.

## Decision Summary
Decision: `adapt_and_split_by_owner`.

Accepted into OxFml canonical direction:
1. admit a replay-visible `CorrectnessFloorContext` or equivalent profile
   context carrying `profile_version`, `numerical_reduction_policy`, and
   `error_algebra`,
2. treat those selectors as semantic evaluation state rather than optimization
   hints,
3. thread the context through semantic-plan, runtime/session, prepared-call
   trace, and replay projection surfaces where relevant,
4. make selector mismatch replay-invalid unless a migration proof is attached.

## Clause Disposition Matrix
| Packet clause | Disposition | Owner | Notes |
|---|---|---|---|
| correctness-floor profile record | accepted | OxFml | Canonical docs now define `CorrectnessFloorContext`. |
| `profile_version` field | accepted | OxFml | Context field accepted. Runtime code does not yet expose it as a separate object. |
| `numerical_reduction_policy` field | adapted | OxFml carriage / OxFunc semantics | OxFml owns carriage; OxFunc owns meaning and enforcement. |
| `error_algebra` field | adapted | OxFml carriage / OxFunc semantics | OxFml owns carriage; OxFunc owns precedence semantics and enforcement. |
| semantic evaluation context status | accepted | OxFml | Selector state is canonical semantic context, not an optimization hint. |
| semantic-plan compile input or linked context | deferred_with_successor_owner | OxFml successor | Owner: correctness-floor context threading successor. Blocker: profile object not yet present in runtime request. |
| function-plan/per-call evaluation context threading | routed_to_oxfunc | OxFunc with OxFml adapter consumption | OxFunc must define kernel metadata/inputs; OxFml threads once exposed. |
| runtime formula request/session context | deferred_with_successor_owner | OxFml successor | Owner: runtime profile-context successor. |
| prepared-call trace and replay projection | deferred_with_successor_owner | OxFml successor | Owner: replay/profile-context successor. |
| candidate/reject diagnostics for selector mismatch | deferred_with_successor_owner | OxFml successor | Owner: replay/profile mismatch successor after selector fields exist. |
| OxCalc must not infer policy from source/scheduler/host | accepted | OxFml/OxCalc | Recorded as non-assumption. |
| replay fields `profile_version`, `numerical_reduction_policy`, `error_algebra` | deferred_with_successor_owner | OxFml successor | Context schema accepted; emitted fields deferred until runtime context exists. |
| replay rejects selector mismatch | accepted | OxFml | Canonical rule accepted; implementation deferred. |
| `PairwiseTree` tree-shape identity | routed_to_oxfunc | OxFunc | OxFunc owns deterministic algorithm shape; OxFml will carry trace identity. |
| `KahanCompensated` compensation policy identity | routed_to_oxfunc | OxFunc | OxFunc owns algorithm semantics; OxFml will carry selector/trace once available. |
| total error precedence order | routed_to_oxfunc | OxFunc | OxFunc owns worksheet-error algebra. |
| `SequentialLeftFold` exact clause | routed_to_oxfunc | OxFunc | Accepted as candidate semantics pending OxFunc acknowledgment. |
| `PairwiseTree` exact clause | routed_to_oxfunc | OxFunc | Candidate semantics; requires OxFunc metadata/evidence. |
| `KahanCompensated` exact clause | routed_to_oxfunc | OxFunc | Candidate semantics; requires OxFunc metadata/evidence. |
| `CanonicalExcelLegacy` exact error precedence | routed_to_oxfunc | OxFunc | Candidate semantics; OxFunc owns precedence truth. |
| `ExtensionRule` selector/profile-version rule | adapted | OxFml/OxFunc | OxFml accepts replay/profile invalidation rule; OxFunc owns admitted error-code ordering. |
| metadata/version signal for selector behavior changes | routed_to_oxfunc | OxFunc | OxFml consumes for prepared-package invalidation. |

## OxFml-Owned Terminology
Preferred field family:
1. `CorrectnessFloorContext`
2. `numerical_reduction_policy`
3. `error_algebra`
4. `profile_version`

These may be nested in a broader OxFml profile-context object if the eventual
runtime API consolidates profile selectors.

## OxFunc-Owned Pieces
1. reduction-sensitive kernel metadata,
2. error-collapse-sensitive kernel metadata,
3. exact numerical algorithms,
4. exact worksheet-error precedence definitions,
5. metadata/version signals that make selector behavior or affected functions
   invalidation-visible.

## OxFunc Receiving Response Consumed
OxFunc has acknowledged this split in
`../OxFunc/docs/handoffs/HANDOFF-CALC-003_OXFUNC_RECEIPT.md` and the canonical
OxFunc seed contract
`../OxFunc/docs/function-lane/OXFUNC_KERNEL_METADATA_AND_ADMISSION_PROFILE_CONTRACT.md`.

OxFml accepts the reserved OxFunc bridge:
1. `semantic_kernel_metadata_version`

OxFml interpretation:
1. this is the prepared-package invalidation signal for selector behavior,
   affected-function classification, and reduction/error-collapse metadata
   changes,
2. conservative all-function prepared-package invalidation is acceptable until a
   narrower per-function or per-family fingerprint exists,
3. runtime and replay fields for selector values and this metadata version are
   reserved until OxFml has a real profile-context and OxFunc metadata source to
   emit them.

## Deferred Pieces
1. first affected function family list,
2. exact replay tree-shape fields for pairwise reduction,
3. exact compensation-state replay fields,
4. claim that current kernels enforce the selectors,
5. replacement of OxCalc local selector artifacts.

## OxFml Counterpart Changes Landed In This Pass
1. Canonical docs define `CorrectnessFloorContext` and owner split.
2. No runtime code field was added for this packet yet because there is no
   current OxFml runtime profile-context object or OxFunc selector metadata to
   populate without inventing semantics.

## Replacement Plan
This packet should be superseded by a narrower two-owner plan:
1. OxFml plan: context carriage and replay identity.
2. OxFunc plan: kernel metadata, selector semantics, and invalidation versioning.

## Canonical Docs Updated
1. `docs/spec/OXFML_CANONICAL_ARTIFACT_SHAPES.md`
2. `docs/spec/OXFML_MINIMUM_SEAM_SCHEMAS.md`
3. `docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
4. `docs/spec/formula-language/OXFML_OXFUNC_SEMANTIC_BOUNDARY.md`
5. `docs/upstream/NOTES_FOR_OXCALC.md`

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - OxFunc Rust metadata fields and registry/export publication
  - concrete replay field names for non-left-fold policies
  - OxFml consumption of `semantic_kernel_metadata_version` in runtime/replay
    artifacts
  - selector enforcement evidence in OxFunc kernels
