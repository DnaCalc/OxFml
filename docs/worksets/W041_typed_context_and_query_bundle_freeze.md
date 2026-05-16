# W041: Typed Context and Query Bundle Freeze

## Purpose
Freeze the first shared OxFml/OxFunc typed context and query bundle for the currently covered built-in function scope, so implementation work can depend on a stable first-pass host capability surface.

## Position and Dependencies
- **Depends on**: `W032`, `W030`, `W038`
- **Blocks**: none
- **Cross-repo**: successor packet corresponding to OxFunc `W047`; OxFml owns capability-scoped evaluator and host-query carrier meaning; OxFunc owns function-semantic consumption of those typed queries once supplied

## Scope
### In scope
1. Freeze the first shared typed context/query bundle for the currently covered seam-heavy rows.
2. Decide whether the current OxFunc query names/result partitioning is sufficient as-is for the first freeze.
3. Keep the bundle capability-scoped and typed rather than object-handle based.
4. Add deterministic local evidence and spec wording for the covered families.
5. Answer whether any first-pass capability families need an OxFml-side merge or split before promotion.

### Out of scope
1. Full host-application runtime lifecycle.
2. Broader distributed coordinator policy.
3. New provider families not already in the current covered OxFunc scope.

## Deliverables
1. A canonical first-pass typed context/query bundle shape.
2. Explicit OxFml wording for what query families are in the first freeze.
3. Deterministic evidence for the covered query families.
4. An explicit list of any still-unfrozen names or result partitions.

## Fresh-Eyes Reset
`W041` is no longer mainly a packet-shape debate.
The shared family set is narrow enough to treat as frozen for the current phase.

The real remaining work is internal architectural consolidation:
1. `TypedContextQueryBundle` must become the canonical input packet for every runtime execution surface that needs host/query capabilities.
2. `TypedContextQueryBundleSpec` must become the canonical durable record of what capability/query families were actually admitted on that run.
3. New or refactored internal surfaces should stop growing ad hoc `host_info` / `locale_ctx` / `now_serial` / `random_provider` plumbing as separate boundary fields.
4. Family merge/split should now be mismatch-driven only; the default posture is to keep the current family set stable.

## Target End-State
The desired `W041` end-state is:
1. one canonical ephemeral runtime packet:
   `TypedContextQueryBundle`
2. one canonical durable retained/replay packet:
   `TypedContextQueryBundleSpec`
3. one canonical family vocabulary, stable for the current freeze:
   `ReferenceResolver`, grouped host-info families, `Rtd`, `RegisteredExternal`, `NowSerial`, `RandomProvider`, `LocaleFormatContext`
4. one architectural rule:
   boundary and service surfaces accept the bundle/spec rather than re-spelling the same capability set as unrelated parameters

Internal execution code may still expand the bundle into convenient local fields for hot-path evaluation.
That is an implementation detail, not the architectural source of truth.

## Implementation Meaning
This work should now be read as a comprehensive refactor lane, not a series of conservative packet tweaks.

The architectural direction is:
1. `bundle-first execution surfaces`
   host, adapter, session, replay, editor/test helpers, and future coordinator-facing execution paths should accept a `TypedContextQueryBundle` at the boundary
2. `spec-first retained surfaces`
   retained artifacts, replay packets, accepted candidate records, and session records should carry `TypedContextQueryBundleSpec` whenever they need durable capability/query-family evidence
3. `family-stability by default`
   do not reopen family naming unless a concrete unsupported/misaligned function lane forces it
4. `no new loose query fields`
   new surfaces should not add fresh one-off `host_info_enabled`-style parameters when the bundle/spec can carry the same truth

## Refactoring Direction
The best-quality endpoint is not “touch as little as possible”.
The best-quality endpoint is a clean internal rule:
all host/query capability truth crosses OxFml inside the typed bundle/spec pair.

That implies the following bold refactor direction:
1. reduce legacy helper/setup code that still constructs execution state by assigning loose fields directly
2. convert replay/oracle/helper paths to bundle-first setup so the exercised evidence matches the intended architecture
3. keep `EvaluationContext` free to cache expanded pointers/values internally, but require bundle application at its boundary
4. treat bundle/spec propagation gaps as design debt to remove, not as acceptable permanent adapter glue

## Execution Phases
1. `Phase A: Boundary normalization`
   convert remaining helper/replay/oracle paths that still build execution state through loose fields into bundle-first setup
2. `Phase B: Durable packet normalization`
   ensure replay/retained surfaces that need query-family truth carry `TypedContextQueryBundleSpec` explicitly rather than reconstructing it indirectly
3. `Phase C: Family-freeze hardening`
   state explicitly that the current family set is the phase-1 freeze unless a concrete mismatch forces merge/split
4. `Phase D: Cleanup enforcement`
   remove or quarantine redundant boundary parameters that duplicate bundle/spec truth

## Gate Model
### Entry gate
- `W032` has narrowed the catalog/callable seam enough that typed host/context surfaces are the next honest lock lane.

### Exit gate
- The first shared typed context/query bundle is explicit enough for implementation use.
- Any non-frozen query naming or partitioning is explicitly listed.
- At least one deterministic artifact exists per in-scope query family.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | |
| 2 | Conformance matrix rows updated? | |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | |
| 4 | Cross-repo impact assessed and handoff filed if needed? | |
| 5 | All required tests pass? | |
| 6 | No known semantic gaps remain in declared scope? | |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | |

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: none
- claim_confidence: validated

## Closure Reading
`W041` is closed for its declared scope.

The declared scope was:
1. freeze the first shared typed context/query bundle,
2. decide the first query-family partitioning,
3. keep the packet capability-scoped and typed,
4. add deterministic local evidence for the covered families,
5. answer whether the current family set needs merge/split before promotion.

Those are now satisfied locally:
1. `TypedContextQueryBundle` and `TypedContextQueryBundleSpec` are the exercised packet pair,
2. the phase-1 family set is explicit and mirrored in `docs/spec/formula-language/OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md`,
3. host, session, adapter, replay-capture, evaluator helper, replay-fixture helper, and selected editor/help surfaces now exercise that packet pair directly,
4. deterministic local evidence exists across host, session, replay, adapter, and editor/help surfaces,
5. OxFml's phase-1 answer is now explicit: no merge/split unless a concrete mismatch forces it.

Remaining broader architecture work is real, but it belongs to:
1. `W043` provider-plus-pinned-snapshot runtime normalization,
2. `W045` host/runtime unification,
3. later broader replay/oracle and coordinator-facing propagation.
