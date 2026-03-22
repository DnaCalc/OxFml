# W043: Runtime Library Context Provider Consumer Model

## Purpose
Turn the converged OxFml/OxFunc runtime library-context direction into a first real OxFml consumer/modeling packet, so the runtime `LibraryContextProvider` / immutable `LibraryContextSnapshot` interface exists as more than note-level agreement.

## Position and Dependencies
- **Depends on**: `W032`
- **Blocks**: none
- **Cross-repo**: successor packet corresponding to OxFunc `W049`; OxFunc owns catalog truth and generation logic; OxFml owns runtime consumption, snapshot pinning, and artifact correlation semantics

## Scope
### In scope
1. Define the first OxFml consumer/model shape for `LibraryContextProvider` and `LibraryContextSnapshot`.
2. Decide whether the runtime consumer shape should mirror the CSV closely or use a cleaner runtime-only shape plus explicit mapping layer.
3. Keep snapshot identity, generation, and registration/removal semantics explicit.
4. Add deterministic local evidence or model artifacts for the first consumer shape.
5. Separate runtime-semantic fields from export-only descriptive fields for the first freeze.

### Out of scope
1. Full registered-external invocation runtime.
2. Final cross-process transport ABI.
3. Full host routing logic for every built-in or external path.

## Deliverables
1. A first OxFml consumer/model shape for the runtime library-context interface.
2. An explicit statement on runtime-only shape versus CSV-mirroring shape.
3. Deterministic evidence or checked formal artifacts for snapshot pinning/generation behavior.
4. An explicit runtime-truth versus export-description field classification.

## Gate Model
### Entry gate
- `W032` has converged the long-term direction toward runtime provider/snapshot rather than file-ingestion coupling.

### Exit gate
- The runtime consumer/model shape is explicit enough for implementation use.
- Snapshot/generation semantics are explicitly stated.
- Remaining open transport or registration gaps are explicitly listed.

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
- execution_state: planned
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - OxFml has note-level convergence on the runtime provider/snapshot direction, but not yet a real consumer/model packet
  - OxFml now prefers a cleaner runtime-only shape plus explicit CSV/export mapping, but that preference is not yet backed by a real consumer/model packet
  - deterministic local evidence still needs to move beyond pure note agreement
  - the runtime-truth versus export-only field split is not yet explicit enough for implementation use
  - current OxFunc reading is that this is now a freeze-and-consumer packet rather than a broad semantic-open lane, and the final note round leaves local execution as the remaining next step
- claim_confidence: draft
