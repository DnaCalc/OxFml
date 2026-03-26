# W052: Registered External Provider And CALL REGISTER.ID Boundary

## Purpose
Narrow the first OxFml-owned worksheet `CALL` / `REGISTER.ID` seam packet so future OxFml <-> OxFunc rounds can be packet-driven rather than theory-driven.

## Position and Dependencies
- **Depends on**: `W045`, `W049`, `W051`
- **Blocks**: none
- **Cross-repo**: bounded OxFml <-> OxFunc and OxFml <-> OxCalc seam packet for worksheet `CALL` / `REGISTER.ID` runtime ownership

## Scope
### In scope
1. Freeze the first OxFml best-effort ownership split for worksheet `CALL` / `REGISTER.ID`.
2. Specify whether `RegisteredExternalProvider` remains separate from `HostInfoProvider`.
3. Specify whether `RegisterIdRequest`, `RegisteredExternalDescriptor`, and `RegisteredExternalCallRequest` cross the boundary directly or only through runtime snapshot/provider lanes.
4. Specify whether descriptor metadata drives reference dereference and general type coercion inside OxFunc rather than in OxFml.
5. Prepare deterministic local harness/test packet expectations once the host/runtime side is narrow enough.
6. Record any concrete OxFunc or OxCalc mismatches against that first packet.

### Out of scope
1. Full native invocation implementation.
2. Final host/coordinator security policy.
3. Broad external-UDF product surface beyond worksheet `CALL` / `REGISTER.ID`.
4. Folding worksheet `CALL` / `REGISTER.ID` into the current `W049` / `W050` first adapter wave before the packet is narrowed.

## Deliverables
1. A canonical OxFml-side worksheet `CALL` / `REGISTER.ID` boundary note.
2. A workset-owned list of first packet fields and ownership rules.
3. A narrowed OxFml reply packet for the next OxFunc/OxCalc seam round.
4. Deterministic local harness cases for:
   - worksheet `REGISTER.ID`
   - worksheet `CALL`
   - reference-visible `CALL` arguments
   - host API registration
   - VBA shim registration
   - unregister packet carriage.

## Gate Model
### Entry gate
- `W049` / `W050` have validated the current first adapter wave strongly enough that worksheet `CALL` / `REGISTER.ID` can be treated as a separate bounded lane.
- OxFunc has asked concrete questions about provider separation and packet carriage.

### Exit gate
- The first worksheet `CALL` / `REGISTER.ID` packet is explicit enough to drive deterministic local harness evidence.
- OxFunc and OxCalc have received a narrowed OxFml position on provider separation, packet carriage, and registration-channel ownership.
- Any remaining disagreement is a concrete field or ownership mismatch, not a broad seam ambiguity.

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
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the first OxFml packet is now drafted and locally exercised through `crates/oxfml_core/tests/w052_registered_external_interface_tests.rs`, but it is not yet acknowledged by both OxFunc and OxCalc as the narrowed shared reading
  - descriptor-driven reference/coercion ownership is locally evidenced, but not yet frozen as shared seam wording
  - built-in catalog ownership plus runtime register/unregister ownership is now sharpened locally, but snapshot-generation and later coordinator-facing acknowledgment remain note-level
  - worksheet `CALL` / `REGISTER.ID` remains a separate bounded lane rather than part of the already-validated `W049` / `W050` adapter wave
- claim_confidence: provisional
