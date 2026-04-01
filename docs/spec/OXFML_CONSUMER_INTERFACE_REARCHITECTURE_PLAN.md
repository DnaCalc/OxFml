# OxFml Consumer Interface Rearchitecture Plan

## 1. Purpose
This document resets the design direction for the user-facing OxFml library surface.

It exists because the current crate surface and supporting docs still expose OxFml mainly as:
1. a flat bag of low-level transforms,
2. a proving-host helper,
3. accumulated seam packets.

That shape was acceptable while the internal architecture was still being discovered.
It is no longer the best end-state for the real consumers now in view:
1. OxCalc,
2. DNA OneCalc,
3. the Replay appliance.

Current design rule:
1. do not preserve the current flat export surface just because it already exists,
2. prefer a smaller set of stable consumer-oriented facades,
3. keep low-level canonical transforms available, but demote them from the primary entry surface.

Hard boundary for this redesign:
1. consumer-facing facade work may reorganize how downstream users enter OxFml,
2. it must compose over the frozen OxFml/OxFunc shared interface families,
3. it must not rename, replace, or reinterpret the frozen OxFunc-facing packet and carrier vocabulary,
4. it must not reopen `W041`, `W042`, `W043`, or `W052` as OxFml/OxFunc seam-design questions.

Read together with:
1. `OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`
2. `OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
3. `OXFML_CONSUMER_INTERFACE_IMPLEMENTATION_PROGRAM_V1.md`
4. `OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md`
5. `OXFML_DNA_ONECALC_DOWNSTREAM_CONSUMER_CONTRACT.md`
6. `OXFML_REPLAY_APPLIANCE_ADAPTER_V1.md`
7. `formula-language/OXFML_EDITOR_LANGUAGE_SERVICE_AND_HOST_INTEGRATION_PLAN.md`
8. `formula-language/OXFML_OXFUNC_LIBRARY_CONTEXT_RUNTIME_INTERFACE.md`

## 2. Current Problem
The current crate root gives downstream users too many choices and too much internal shape.

Main problems:
1. flat root exports leak internal layering and historical sequencing rather than presenting a clean user model,
2. host/runtime, session, editor, and replay-facing users all assemble overlapping request objects from lower-level pieces,
3. runtime provider-plus-pin semantics are now real internally, but the public surface still makes consumers think in terms of loose providers, raw snapshots, and ad hoc request wiring,
4. replay users are still pointed at raw artifact families and helper packets rather than a narrow projection service,
5. DNA OneCalc and OxCalc both need a host/runtime facade, but not the same policy layer,
6. editor integration currently exposes useful packets, but not yet as part of one coherent consumer-facing service family.

## 3. Architectural Goal
OxFml should present three primary consumer-facing facade families:
1. runtime execution,
2. editor/language service,
3. replay projection.

The low-level canonical transforms remain available, but they become:
1. implementation substrate,
2. advanced/diagnostic entry surface,
3. replay/spec provenance surface.

They should no longer be the default integration surface for ordinary downstream users.

Current-support rule:
1. this document defines the preferred endpoint, not a claim that facade modules already exist today,
2. `W054` now has a first real runtime-facade code slice under
   `crates/oxfml_core/src/consumer/runtime`,
3. `W054` now also has a first editor-facade code slice under
   `crates/oxfml_core/src/consumer/editor`,
4. until the full facade family lands, the current supported Rust entry surface
   remains a mixed surface:
   - existing crate exports and exercised helper/service APIs,
   - plus the new partial runtime and editor facades,
5. documentation should therefore distinguish clearly between:
   - active supported surface now,
   - preferred facade-first surface later,
6. the detailed target contract for that preferred facade-first surface now
   lives in `OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`.

Current finishing rule:
1. further design narrowing should now happen inside the consumer contract
   packet rather than by reopening broad architectural direction here,
2. broad redesign is settled enough; remaining work is facade implementation,
   migration, and bounded implementation-driven clarifications,
3. the concrete code-facing wave plan now lives in
   `OXFML_CONSUMER_INTERFACE_IMPLEMENTATION_PROGRAM_V1.md`.

## 4. Canonical Consumer Families

### 4.1 Runtime Facade
This is the primary consumer family for OxCalc and DNA OneCalc execution.

The runtime facade should gather what is currently spread across:
1. `SingleFormulaHost`,
2. parts of `SessionService`,
3. typed query bundle and provider wiring,
4. runtime library-context pinning,
5. candidate/commit/reject projection.

Preferred high-level shape:
1. `RuntimeEnvironment`
   - structure context
   - bindings
   - typed query bundle
   - library-context provider plus pinned snapshot selection
   - table context
   - registered-external context where needed
2. `RuntimeFormulaRequest`
   - formula source
   - requested execution mode
   - requested artifact depth or capture policy
3. `RuntimeFormulaResult`
   - syntax/bind/semantic diagnostics
   - semantic plan identity and execution contract
   - evaluation result
   - returned value surface
   - accepted candidate / commit / reject families
   - trace capture families
4. `RuntimeSessionFacade`
   - prepare/open/execute/commit lifecycle over the same environment model

Design rule:
1. provider-plus-pin semantics must be first-class in the runtime facade,
2. "current snapshot" must be treated as a selection convenience, not the implicit runtime truth,
3. the runtime facade should make it hard for consumers to accidentally execute against ambient mutable catalog state.

### 4.2 Editor Facade
This is the primary consumer family for DNA OneCalc editing and later richer host editors.

The editor facade should gather what is currently spread across:
1. `FormulaEditRequest` / `FormulaEditResult`,
2. completion,
3. signature help,
4. function-help lookup request building,
5. intelligent completion context.

Preferred high-level shape:
1. `EditorEnvironment`
   - bind context
   - structure context
   - library-context provider plus pinned snapshot selection
   - optional table context and visible names
2. `EditorDocument`
   - formula source
   - current immutable syntax/bind artifacts
3. `EditorEditService`
   - apply edit
   - compute diagnostics
   - compute completion
   - compute signature help
   - build function-help lookup requests
4. `EditorInteractionResult`
   - updated immutable formula artifacts
   - live diagnostics
   - completion/signature packets
   - optional function-help lookup request

Design rule:
1. editor consumers should not need to manually stitch together parse/bind/plan calls,
2. editor environment should use the same provider-plus-pin library-context rule as runtime execution,
3. intelligent completion remains non-canonical until it re-enters the ordinary edit path.

### 4.3 Replay Facade
This is the primary consumer family for the Replay appliance and replay-aware hosts.

The replay facade should gather what is currently spread across:
1. raw seam artifacts,
2. `HostRecalcOutput::to_first_host_replay_capture_packet()`,
3. replay projection docs and fixture families.

Preferred high-level shape:
1. `ReplayProjectionRequest`
   - runtime result, host result, or session record
   - requested projection family
   - sidecar policy
2. `ReplayProjectionService`
   - project host result to replay packet
   - project session lifecycle to replay packet
   - project canonical artifacts to replay source refs or sidecars
3. `ReplayProjectionResult`
   - normalized replay envelope refs
   - preserved source artifact refs
   - registry bindings
   - capability-level and lifecycle metadata

Design rule:
1. replay consumers should not need to depend directly on broad internal artifact families unless they are deliberately doing schema/provenance work,
2. replay projection remains additive and must not redefine OxFml semantics,
3. replay projection should be a service/facade, not a loose collection of helper methods.

## 5. Consumer Mapping

These mappings describe the intended post-`W054` endpoint.
They are not a claim that the named facade modules already exist in the current crate.

### 5.1 OxCalc
OxCalc should primarily consume:
1. runtime facade,
2. session facade,
3. candidate/commit/reject/trace projections from the runtime result.

OxCalc should not consume:
1. direct proving-host helpers as if they were the long-term coordinator interface,
2. flat low-level parse/bind/evaluate exports as its normal integration shape.

### 5.2 DNA OneCalc
DNA OneCalc should primarily consume:
1. runtime facade for execution,
2. editor facade for edit/diagnostic/completion/help interactions,
3. replay facade for retained local evidence and local host capture.

DNA OneCalc should not need to choose between:
1. `SingleFormulaHost`,
2. raw transform chain,
3. editor packet helpers,
4. ad hoc replay helpers,
as four unrelated integration styles.

### 5.3 Replay Appliance
Replay appliance integration should primarily consume:
1. replay facade,
2. canonical packet projection outputs,
3. sidecar-backed canonical artifact refs when needed.

Replay should not need to depend on:
1. proving-host-only helper methods as the long-term public integration surface,
2. arbitrary internal crate module boundaries.

## 6. Internal Restructuring Implications
The best endpoint is not just new docs.
It implies a crate packaging reset.

Preferred package/module direction:
1. keep low-level canonical transforms under explicit low-level modules,
2. add consumer-facing facade modules, for example:
   - `consumer::runtime`
   - `consumer::editor`
   - `consumer::replay`
3. treat historical flat root re-exports as implementation substrate rather than
   letting them define the long-term public model.

Working rule:
1. new docs should treat facade families as the intended consumer surface,
2. historical root exports may still exist in code during refactoring,
3. but they should no longer be described as the supported contract for new
   downstream design.

## 7. Boundary Objects To Promote
The following internal ideas should be promoted into first-class boundary objects:
1. `PinnedLibraryContextView`
2. runtime environment packet for provider-plus-pin and typed query bundle
3. runtime formula request/result packet
4. editor environment packet
5. replay projection request/result packet

The following should be demoted from primary user entry:
1. direct low-level transform chaining as the default host integration method,
2. proving-host helper methods as the default runtime interface,
3. flat crate-root re-export discovery as the main way to understand OxFml.

## 8. Migration Strategy
The migration should be staged.

### Phase A: Documentation Reset
1. declare the three facade families as the preferred consumer model,
2. update host/runtime, OneCalc, editor, and replay docs to point to that model,
3. state explicitly that historical root/helper surfaces are implementation
   substrate rather than intended consumer contract,
4. issue explicit outbound notes to:
   - OxCalc
   - DNA OneCalc
   - replay-facing consumers
   describing the target facade surface, current implementation reality, and
   frozen-seam constraints.

### Phase B: Public Packaging Reset
1. add facade modules and facade request/result/environment objects,
2. keep current low-level exports available,
3. document the low-level surface as advanced/internal substrate,
4. preserve the frozen OxFml/OxFunc packet families unchanged while doing so,
5. refactor current host, editor, and replay helpers toward the new facade
   layers where practical so the transition does not fork behavior.
6. execute the concrete wave plan from
   `OXFML_CONSUMER_INTERFACE_IMPLEMENTATION_PROGRAM_V1.md` rather than letting
   packaging reset devolve into ad hoc wrapper growth.

### Phase C: Consumer Uptake
1. move OxCalc-facing guidance to runtime/session facade usage first,
2. move DNA OneCalc-facing guidance to runtime/editor/replay facade usage,
3. move replay guidance to replay projection service usage,
4. keep the transition order explicit:
   - runtime first
   - editor second
   - replay third

### Phase D: Cleanup And Public Tightening
1. decide which historical root re-exports still need to exist at all,
2. collapse redundant helper paths once facade adoption is real,
3. prevent new outward-facing docs from reintroducing flat-surface debt.

## 9. Recommended Implementation Order
1. Runtime facade first
   - shared consumer need across OxCalc and DNA OneCalc
   - builds directly on `W043` and `W045`
   - first partial runtime slice now exists and should be treated as the
     implementation base rather than restarted from scratch
2. Editor facade second
   - DNA OneCalc-facing
   - builds on `W048`
   - first partial editor slice now exists and should be widened rather than
     replaced
3. Replay facade third
   - replay appliance-facing
   - builds on `W046` and existing replay helper surfaces

Reason for this order:
1. runtime facade removes the largest current downstream wrapper pressure earliest,
2. editor facade can then reuse the same provider-plus-pin discipline rather than inventing a parallel model,
3. replay facade should follow once the runtime-facing packet surface is stable enough to project cleanly.

## 10. Current Proposed Workset Split
This redesign should be executed under a dedicated workset rather than smuggled into `W043`, `W045`, or `W048`.

Recommended owner:
1. `W054` consumer-facing interface rearchitecture and facade packaging

Supporting prerequisite owners remain:
1. `W043` runtime provider-plus-pin model
2. `W045` host/runtime contract
3. `W048` editor packet substrate
4. `W046` replay-facing host packet

## 11. Working Rule
From this point onward, OxFml should optimize its external design for the real consumers:
1. OxCalc,
2. DNA OneCalc,
3. Replay appliance.

Do not let the existing flat export surface become the accidental permanent architecture just because it exists early.
Do not use consumer-surface redesign as a pretext to reopen the frozen OxFml/OxFunc seam.
Treat `OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md` as the current
OxFml-owned packet for downstream consumer review.
Treat `OXFML_CONSUMER_INTERFACE_IMPLEMENTATION_PROGRAM_V1.md` as the current
implementation-driving wave plan for that packet.
