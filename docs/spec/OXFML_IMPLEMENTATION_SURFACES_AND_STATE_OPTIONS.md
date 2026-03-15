# OxFml Implementation Surfaces and State Options

## 1. Purpose
This document records the current OxFml position on implementation shape, ownership, and state.

It exists because the semantic architecture is already converging on:
1. immutable parse and bind artifacts,
2. replayable evaluation semantics,
3. explicit boundary ownership with OxFunc and OxCalc,

while the concrete API and storage surface is still intentionally open.

This document is not an implementation commitment.
It is a design-constraint document for later implementation work.

The current code-start baseline derived from these options is:
1. `OXFML_IMPLEMENTATION_BASELINE.md`

The canonical artifact-key vocabulary used by these options is defined in:
1. `OXFML_ARTIFACT_IDENTITIES_AND_VERSION_KEYS.md`

The canonical field surfaces for the main artifact families are defined in:
1. `OXFML_CANONICAL_ARTIFACT_SHAPES.md`

The current code-facing API and service sketch is defined in:
1. `OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`

## 2. Fixed Semantic Constraints
The following are already fixed and are not implementation options:

1. parsing produces immutable full-fidelity green trees,
2. green trees must be suitable for inclusion in larger immutable workbook/document structures,
3. bind outputs and semantic plans should admit immutable versioned representations,
4. evaluation meaning must be explainable from explicit artifacts plus explicit context,
5. caches, registries, and sessions must not become hidden semantic truth,
6. the whole surface must admit a stateless, replayable interpretation.

Any implementation option that violates one of these is out of bounds.

## 3. Artifact Classes
The main OxFml artifact families are:

1. formula source records
   - entered text,
   - stored/normalized text,
   - diagnostics.
2. syntax artifacts
   - green trees,
   - red contextual views.
3. bind artifacts
   - `BoundFormula`,
   - normalized references,
   - dependency seeds,
   - unresolved-reference records.
4. semantic artifacts
   - `SemanticPlan`,
   - prepared-call descriptors,
   - capability requirements.
5. evaluation-operational artifacts
   - evaluator sessions,
   - overlay state,
   - execution traces.
6. publication artifacts
   - commit bundles,
   - reject records,
   - replay packs.

The first four families should be modeled as canonical immutable artifacts.
The fifth family may be operational state.
The sixth family is published evidence and should be durable/replayable even when produced by stateful machinery.

## 4. Ownership Dimensions
There are three separate ownership questions that should not be conflated:

1. semantic ownership
   - which repo/spec lane defines the meaning of an artifact.
2. API ownership
   - which component exposes the construction and consumption surface.
3. storage/runtime ownership
   - which component physically retains or indexes the artifacts between calls.

Current working position:
1. semantic ownership of parse/bind/evaluation artifacts is OxFml,
2. semantic ownership of function definitions remains OxFunc,
3. semantic ownership of workbook-wide scheduling/publication policy remains OxCalc,
4. storage/runtime ownership above the evaluator remains open.

## 5. Candidate Implementation Shapes
Three implementation shapes are currently considered viable.

### 5.1 Option A: Stateless Functional Surface
OxFml exposes primarily pure or near-pure transforms:
1. parse formula text -> green tree,
2. bind green tree + explicit context -> immutable bind artifact,
3. compile bind artifact + OxFunc catalog -> immutable semantic plan,
4. evaluate semantic plan + snapshot/context -> commit bundle or reject.

Characteristics:
1. canonical state lives outside OxFml,
2. host or coordinator retains artifact graphs and version maps,
3. testing and replay are straightforward,
4. performance may require external caching layers or repeated recomputation.

Strengths:
1. clearest formal model,
2. strongest ownership separation,
3. easiest to embed in multiple hosts.

Risks:
1. repeated context construction may be expensive,
2. cache responsibility may be fragmented across callers,
3. ergonomics may be poor for high-frequency integrated evaluation.

### 5.2 Option B: Stateful Service Surface
OxFml exposes long-lived services or repositories:
1. parse trees may be interned in OxFml-managed stores,
2. bind and semantic artifacts may be retained behind service handles,
3. evaluator sessions and overlay stores may be first-class OxFml runtime state.

Characteristics:
1. callers interact through service calls and stable handles,
2. OxFml owns more caching and residency policy,
3. integrated evaluation can be faster and operationally simpler for hosts.

Strengths:
1. performance-oriented,
2. centralizes caching policy,
3. may align better with long-lived engine processes.

Risks:
1. hidden state can blur semantic boundaries,
2. replay and formalization become harder if handles are under-specified,
3. host integration may become tightly coupled to one runtime model.

### 5.3 Option C: Hybrid Canonical-Core Plus Runtime Services
OxFml defines canonical immutable artifacts and stateless transforms, while also allowing optional runtime services for caching and session management.

Characteristics:
1. the semantic model is stateless-first,
2. the runtime model may add repositories, caches, indexes, and session registries,
3. all service behavior must be observationally reducible to the stateless model.

Strengths:
1. preserves the formal/replayable core,
2. still allows practical integrated-engine performance,
3. gives DNA OneCalc and OxCalc different integration shapes without changing semantics.

Risks:
1. requires discipline to keep service state non-authoritative,
2. requires explicit equivalence testing between direct and cached paths,
3. increases design surface area.

## 6. Current Working Preference
The current OxFml design preference is Option C.

Reason:
1. it keeps the semantic model compatible with immutable workbook/document structures,
2. it keeps replay, Lean, and TLA+ modeling tractable,
3. it still permits high-performance integrated hosts to use caches and evaluator services,
4. it avoids prematurely forcing OxCalc or DNA OneCalc into the same runtime shape.

This is a design preference, not yet a locked implementation decision.

## 7. Ownership by Artifact Family
Current expected ownership posture:

| artifact_family | semantic owner | likely storage owner in OxCalc-integrated mode | allowed OxFml runtime ownership |
|---|---|---|---|
| Green trees | OxFml | host or OxCalc immutable workbook/document structure | cache/interner only |
| Red views | OxFml | usually ephemeral/non-persistent | yes, ephemeral |
| Bound formulas | OxFml | host or OxCalc versioned structures, or shared repository | cache/repository allowed if reducible to explicit artifacts |
| Semantic plans | OxFml + OxFunc metadata dependency | host/OxCalc or OxFml repository | cache/repository allowed |
| Evaluator sessions | OxFml seam runtime | OxFml runtime or coordinator-adjacent runtime | yes, primary operational owner |
| Overlays | OxFml evaluator facts consumed by OxCalc policy | open | yes, session-local/runtime-managed |
| Commit bundles/rejects | OxFml seam outputs | host/OxCalc as published evidence | transient generation plus durable publication |

## 8. API Constraints Regardless of Option
Whichever option is chosen, the public surface should preserve:

1. explicit version inputs and outputs,
2. explicit snapshot/profile/caller context,
3. typed identities for formulas, bind artifacts, and sessions where sessions exist,
4. deterministic replay inputs for every evaluator decision that affects outputs,
5. clear separation between canonical artifacts and optional residency handles.

Handle-based APIs are acceptable only if the underlying semantic artifact can still be serialized or reconstructed in a replayable form.

## 9. DNA OneCalc and OxCalc Implications
The two main downstream hosts likely want different operational shapes.

### 9.1 DNA OneCalc
DNA OneCalc likely benefits from:
1. a direct stateless or lightly cached surface,
2. explicit immutable artifacts,
3. minimal long-lived runtime machinery.

### 9.2 OxCalc
OxCalc-integrated mode likely benefits from:
1. persistent immutable workbook/document versions above the evaluator,
2. reusable bind/semantic caches,
3. managed evaluator sessions and overlay stores.

The semantic model must support both without drift.

## 10. Formalization Implications
This design question directly affects the formal posture.

Lean-friendly baseline:
1. artifact ADTs and transforms should be defined independent of cache residency.

TLA+-friendly baseline:
1. runtime repositories, session registries, and overlay stores may be modeled as state machines,
2. those state machines must refine a simpler stateless artifact interpretation.

Replay baseline:
1. cached and uncached execution paths must be observationally equivalent for the same declared inputs.

## 11. Open Decisions
The following remain open and should stay explicit:

1. whether OxFml publishes a first-class repository API for parse/bind/semantic artifacts,
2. whether OxCalc owns all durable artifact retention in integrated mode,
3. whether bind and semantic caches are keyed by structural versions, formula tokens, or both,
4. how much evaluator session state is local to OxFml versus shared with coordinator services,
5. whether publication-oriented overlay stores live entirely inside FEC/F3E runtime services.

## 12. Working Rule
Until implementation work starts, design docs should:
1. specify canonical artifacts as immutable and replayable,
2. treat stateful services as optional operational layers,
3. avoid wording that accidentally commits the repo to pure-stateless-only or service-owned-only designs.
