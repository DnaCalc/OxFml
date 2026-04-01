# W054: Consumer-Facing Interface Rearchitecture And Facade Packaging

## Purpose
Re-architect the user-facing OxFml library interface around the three real consumers:
1. OxCalc,
2. DNA OneCalc,
3. Replay appliance.

The goal is to replace the current flat, historically accreted public shape with
a smaller set of consumer-oriented facade families while preserving low-level
canonical transforms as implementation substrate.

## Position and Dependencies
- **Depends on**: `W043`, `W045`, `W046`, `W048`
- **Blocks**: later downstream consumer cleanup and public-surface freeze work
- **Cross-repo**: OxCalc and DNA OneCalc consume the runtime/editor/replay facade direction; Replay appliance consumes the replay facade direction

## Scope
### In scope
1. Define the preferred consumer-facing facade families for runtime, editor, and replay use.
2. Decide how those facade families relate to the existing flat crate exports.
3. Define promoted boundary objects such as runtime environment, editor environment, replay projection service, and provider-plus-pin runtime view.
4. Update the core user-facing OxFml docs to point to the new consumer-oriented model.
5. Plan the staged public packaging reset.
6. Keep the distinction clear between intended consumer contract and current implementation substrate during the redesign and rollout.
7. Keep the frozen OxFml/OxFunc shared interface families unchanged while consumer-facing packaging evolves.

### Out of scope
1. Immediate removal of current low-level exports.
2. Final ABI freeze for all public Rust type names.
3. Complete downstream migration in OxCalc or DNA OneCalc.
4. Replay pack-grade promotion.
5. Renaming, wrapping, or redesigning the frozen OxFml/OxFunc shared packet families.

## Deliverables
1. A canonical consumer-interface redesign document.
2. A canonical consumer-interface contract packet that downstream users can
   review against directly.
3. Updated public API, host/runtime, OneCalc, editor, and replay docs aligned to the facade model.
4. An explicit migration strategy from flat exports to facade-first usage.
5. A narrowed implementation plan for later crate packaging work.
6. A concrete implementation program tied to actual module moves,
   evidence gates, downstream uptake checkpoints, and cleanup of any temporary
   transition helpers.

## Detailed Execution Plan

### Phase 1: Coordination Packet Reset
1. Treat `W054` as a coordinated consumer migration, not a crate-local cleanup.
2. Keep the OxFml <-> OxFunc seam frozen while all consumer-facing work proceeds.
3. Publish one canonical consumer-interface packet that all three downstream
   consumers can compare against.
4. Update the outbound notes so each consumer sees:
   - target facade surface now,
   - current implementation reality,
   - frozen seam facts that must not drift,
   - local responsibilities that remain host-owned.
5. Required outbound packets in this phase:
   - `NOTES_FOR_OXCALC.md`
   - `NOTES_FOR_DNAONECALC.md`
   - `NOTES_FOR_OXREPLAY.md`
   - replay-facing guidance in `OXFML_REPLAY_APPLIANCE_ADAPTER_V1.md`

### Phase 2: Internal Packaging Design
1. Define a new explicit consumer packaging tree under OxFml:
   - `consumer::runtime`
   - `consumer::editor`
   - `consumer::replay`
2. Treat current crate-root exports and historical helper surfaces as
   implementation substrate, not as the intended consumer contract.
3. Introduce a cleanup rule:
   - any temporary transition helper stays narrow and internal in intent,
   - new docs and new downstream work target the facade surfaces first,
   - no frozen OxFml/OxFunc packet family is wrapped in a competing public
     vocabulary.
4. Define the boundary objects that each facade must expose before downstream migration starts:
   - runtime environment/request/result/session
   - editor environment/document/service/interaction result
   - replay projection request/service/result
5. Express the code-facing execution of those phases in one canonical program:
   `docs/spec/OXFML_CONSUMER_INTERFACE_IMPLEMENTATION_PROGRAM_V1.md`

### Phase 3: Runtime Facade Implementation
1. Build the runtime facade first because OxCalc and DNA OneCalc both need it.
2. Lift existing host/session/provider-plus-pin entry logic into one runtime environment model.
3. Make provider-plus-pin library-context selection explicit in the facade.
4. Carry typed query bundles, return surfaces, candidate/commit/reject families, and trace capture through that facade.
5. Refactor current host helpers toward the new runtime layer where that reduces
   duplicate logic, without letting those helpers define the consumer contract.
6. Treat active `W043` provider-plus-pin normalization as part of this wave's
   substrate, not as a competing architecture track.

### Phase 4: Editor Facade Implementation
1. Build the editor facade second on top of the existing language-service substrate.
2. Move edit application, diagnostics, completion, signature help, and function-help packet publication behind one editor service model.
3. Keep intelligent completion non-canonical until re-entry through the ordinary edit path.
4. Keep OxFunc-owned help payload truth out of the facade; only OxFml-owned request/context logic belongs here for now.

### Phase 5: Replay Facade Implementation
1. Build the replay facade third on top of the existing replay helper and projection path.
2. Make replay projection a narrower service boundary rather than a broad artifact-discovery exercise for consumers.
3. Refactor current replay/helper projection assembly toward the replay facade
   where that reduces duplicate logic.
4. Keep replay capability claims and witness lifecycle governance exactly as they already stand; this phase is packaging, not replay-grade promotion.

### Phase 6: Consumer Uptake
1. OxCalc uptake:
   - first consume the runtime facade for host/coordinator entry,
   - keep coordinator semantics and policy outside OxFml,
   - do not consume flat low-level exports as the normal integration path anymore.
2. DNA OneCalc uptake:
   - first consume the runtime facade,
   - then the editor facade,
   - then the replay facade.
3. Replay appliance uptake:
   - consume the replay projection facade as the preferred entrypoint,
   - keep broad raw artifact usage only for schema/provenance work.

### Phase 7: Cleanup And Public Tightening
1. After the three consumers have real uptake paths, remove temporary
   transition helpers that are no longer materially useful.
2. Remove or demote redundant historical entrypoints once facade-native paths are
   strong enough and downstream adoption has started.
   Current floor:
   - direct top-level `host`, `session`, `language_service`, and
     `oxfunc_adapter` exposure has been demoted
   - `language_service` is now private editor substrate rather than a public
     hidden transition path
   - advanced public substrate use now goes through the remaining explicit
     explicit `test_support::...` support namespacing
   - flat crate-root facade re-exports have been removed, so ordinary consumer
     discovery now goes through explicit `consumer::runtime`,
     `consumer::editor`, and `consumer::replay`
3. Audit all user-facing docs again so they no longer point ordinary consumers
   at the historical flat export set.

## Concrete Implementation Program
The concrete code-facing program now lives in:
1. `docs/spec/OXFML_CONSUMER_INTERFACE_IMPLEMENTATION_PROGRAM_V1.md`

Working rule:
1. this workset remains the owner for `W054`,
2. the implementation program carries the actual wave-by-wave refactor plan,
3. the implementation program now also carries the complete migration matrix:
   - every session lifecycle helper to absorb,
   - every language-service helper to absorb,
   - every replay/adapter substrate surface to absorb or demote,
   - and the full public substrate demotion map,
4. any implementation-driven clarification should update both documents rather
   than letting the workset and code plan drift apart.

## Consumer-Specific Coordination Plan
1. OxCalc:
   - coordinate around runtime/session facade uptake,
   - preserve current host/runtime packet meaning,
   - avoid coordinator-policy drift into OxFml.
2. DNA OneCalc:
   - coordinate around runtime + editor + replay facade uptake,
   - align local naming to upstream packet names before facade migration,
   - avoid building a second wrapper vocabulary over frozen OxFml/OxFunc packets.
3. Replay appliance:
   - coordinate around replay projection service uptake,
   - keep replay additive and transport-oriented,
   - avoid semantic reinterpretation of OxFml artifacts.

## Gate Model
### Entry gate
- `W043`, `W045`, `W046`, and `W048` have produced enough runtime, editor, and replay packet truth to justify a deliberate consumer-surface redesign rather than another incremental export growth step.

### Exit gate
- The preferred consumer families are explicit.
- A canonical consumer-interface contract packet exists and is referenced by the
  downstream notes.
- The runtime-facing contract explicitly names the first stable correlation
  subset, surfaced fact visibility rule, and caller-context carriage rule.
- The consumer-interface contract is frozen as the `OxFml_V1` seam packet for
  downstream implementation.
- Current docs no longer treat the flat export surface as the preferred long-term integration shape.
- Current docs clearly distinguish the supported surface now from the target facade surface later.
- Current docs do not imply that consumer-facing packaging work reopens the frozen OxFml/OxFunc seam.
- Migration phases are explicit.
- Remaining packaging and implementation work is explicitly listed.
- The implementation program is explicit enough to drive code refactoring and
  downstream uptake coordination without reopening the architecture packet.
- For `OxFml_V1`, the redesign/refactor scope is finite and any future
  broadening must be outside `W054`.

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
- claim_confidence: high

## Out Of Scope After W054
The following are not open `W054` lanes:
1. later replay breadth expansion beyond the landed replay facade floor,
2. optional further narrowing of explicit `test_support` helpers,
3. downstream uptake or migration work inside consumer repos,
4. any future bounded clarification that would require a new workset.
