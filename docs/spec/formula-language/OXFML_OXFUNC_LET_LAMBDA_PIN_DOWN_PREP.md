# OxFml/OxFunc `LET` / `LAMBDA` Seam Pin-Down Prep

## Purpose
Prepare the next narrowing round for the `LET` / `LAMBDA` seam between OxFml and OxFunc.

This document is a preparation artifact, not a final seam lock.
Its job is to isolate:
1. what OxFml now knows locally,
2. what should already be treated as fixed for seam purposes,
3. what remains intentionally open,
4. what exact questions OxFunc should answer next,
5. when the lane would become coordinator-visible enough to involve OxCalc.

## Authority Split
1. OxFml remains authoritative for helper syntax, helper binding structure, lexical scope, parameter shadowing, and formula-stage callable preservation.
2. OxFunc remains authoritative for callable semantic value behavior once callable meaning crosses the semantic boundary, including value-universe behavior and any later function-semantic interaction.
3. This prep note does not transfer parse, bind, or formula-artifact ownership away from OxFml.
4. This prep note does not authorize a final shared callable transport contract.

## Current Exercised OxFml Floor
The local exercised floor now includes:
1. sequential `LET` binding,
2. helper-name shadowing,
3. `LAMBDA` literal formation,
4. immediate `LAMBDA` invocation,
5. helper-bound `LAMBDA` invocation,
6. lexical rather than dynamic helper capture,
7. exact free-helper capture rather than "all visible helpers",
8. parameter-shadowing-sensitive capture exclusion,
9. callable summary/detail plus typed minimum callable carrier,
10. callable preservation through adopted defined-name context,
11. deterministic replay/proving fixtures for helper-local callable outcomes and defined-name callable transport,
12. typed higher-order helper execution for `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY` through a local callable invoker with inline and helper-bound lambdas,
13. first higher-order execution through an adopted defined-name callable carrier (`MAP(...,NamedLambda)`).

Primary exercised evidence currently lives in:
1. `crates/oxfml_core/tests/evaluator_tests.rs`
2. `crates/oxfml_core/tests/replay_fixture_tests.rs`
3. `crates/oxfml_core/tests/callable_transport_tests.rs`
4. `crates/oxfml_core/tests/fixtures/prepared_call_replay_cases.json`
5. `crates/oxfml_core/tests/fixtures/callable_transport_cases.json`

## Fixed OxFml-Side Truths For The Next Round
These points should now be treated as stable enough to narrow against.

### 1. Lexical, Not Dynamic
1. A created helper lambda must preserve lexical meaning.
2. Later helper-name shadowing must not rebind an existing lambda dynamically.
3. Any downstream callable carrier that erases lexical meaning is too weak.

### 2. Exact Capture, Not Approximate Capture
1. Capture reporting must exclude unused helper bindings.
2. Capture reporting must exclude names shadowed by lambda parameters.
3. "All helper names in visible scope" is not acceptable where exact free-helper capture is knowable.

### 3. Callable Values Are Semantic Values
1. Callable values are semantically first-class in OxFml even if publication policy remains narrower.
2. Callable preservation through adopted defined-name context is already part of the local floor.
3. Publication restrictions are not the same question as semantic admissibility.

### 4. OxFml Does Not Need A Full Final Transport To Preserve Meaning
1. OxFml can already preserve callable meaning through a typed minimum carrier plus structured callable detail.
2. That current carrier is intentionally provisional.
3. The next round should narrow it, not replace it with an opaque "callable exists" marker.

## Exact Remaining Seam Questions
These are the questions to pin down with OxFunc next.

### Q1. Minimum Callable Carrier
What is the smallest honest shared callable carrier beyond the current local floor?

Current OxFml candidate minimum:
1. origin kind,
2. invocation model,
3. capture mode,
4. arity,
5. parameter-name surface when replay/explanation requires it,
6. capture-name surface when lexical meaning would otherwise be lost,
7. body-kind class when needed for replay/explanation.

Decision needed:
1. which of those are mandatory shared-carrier fields,
2. which may remain replay-sidecar or OxFml-local detail.

Current processed OxFunc response:
1. OxFunc now prefers a smaller minimum carrier centered on opaque callable identity plus semantic minimums,
2. OxFunc does not currently want parameter-name and capture-name detail inside the minimum shared carrier,
3. OxFml can work with that direction if provenance/replay surfaces preserve the omitted detail explicitly.

Current OxFml narrowing response:
1. adopt: parameter-name, capture-name, and body-kind detail may move out of the minimum carrier and remain provenance/replay detail,
2. adopt: the shared callable carrier can be smaller than the full current replay summary/detail floor,
3. alternative to OxFunc's proposal: opaque callable identity is acceptable only if origin kind, capture mode, arity shape, and invocation contract remain recoverable as typed fields rather than hidden behind an uninterpreted token.

### Q2. Invocation Boundary
How should callable invocation cross the seam?

Current OxFml floor supports:
1. helper-local invocation,
2. adopted defined-name callable invocation.

Decision needed:
1. whether OxFunc expects typed invocation against an opaque callable token,
2. whether parameter names and captures must be directly inspectable at the seam,
3. whether invocation should consume callable values only through typed invocation or also through richer callable inspection.

Current processed OxFunc response:
1. OxFunc prefers typed invocation over opaque callable identity rather than direct seam inspection of full helper structure,
2. OxFunc currently treats parameter/capture/body detail as provenance, not minimum invocation-surface data.

Current OxFml response:
1. adopt: typed invocation over a narrower callable carrier is the right direction,
2. adopted local evidence: typed invocation over opaque callable identity is already viable for the first higher-order helper floor and does not require direct AST transfer,
3. alternative: `invocation_contract_ref` is acceptable only if it points to stable semantic invocation meaning rather than an implementation-specific callback or ABI handle.

### Q3. Callable Provenance Vs Callable Transport
What belongs in prepared-result provenance versus the callable carrier itself?

Current OxFml posture:
1. semantic meaning should not depend only on free-form summary text,
2. replay/explanation benefits from structured detail,
3. not all structured detail necessarily belongs in the minimum shared transport.

Decision needed:
1. which fields are carrier,
2. which are provenance,
3. which may remain replay-only.

Current processed OxFunc response:
1. parameter names, capture names, and body-kind detail should remain provenance/replay detail for now,
2. the minimum carrier should stay smaller.

Current OxFml response:
1. adopt: those richer fields do not need to sit in the minimum carrier in the next round,
2. alternative: they still need to survive as structured provenance rather than only free-form summary text.

### Q4. Stage Interaction With Availability / Provider States
How should callable lanes interact with unresolved/gated/provider outcomes?

Decision needed:
1. whether callable surfaces can be catalog-known but transport-restricted,
2. whether callable invocation denial is typed as semantic-plan unsupported, runtime capability denial, or provider/runtime failure depending on stage,
3. how callable-specific failure should remain distinct from unresolved-name and general provider-failure lanes.

## Questions Explicitly Not In Scope For This Round
1. full UDF or product callable transport,
2. final worksheet publication policy for callable values,
3. coordinator scheduling or retry policy,
4. raw AST transfer to OxFunc,
5. broad non-`LET`/`LAMBDA` higher-order language expansion.

## Proposed OxFml Negotiation Order
OxFml should push the next round in this order:
1. agree the fixed truths above,
2. agree the smallest honest minimum callable carrier,
3. agree the carrier vs provenance split,
4. agree the invocation boundary,
5. only then revisit callable interaction with broader provider/runtime states where still needed.

Reason:
1. lexical and capture truth is already exercised locally,
2. transport narrowing is now blocked more by carrier/provenance ambiguity than by parser/bind uncertainty, invocation viability, or the first higher-order helper runtime floor,
3. broader provider/runtime interaction can be tightened more honestly once the callable object crossing the seam is smaller and less ambiguous.

## OxCalc Trigger Condition
This lane should stay OxFml/OxFunc-local unless one of these becomes true:
1. callable publication policy changes candidate vs commit consequences,
2. callable transport changes typed reject families or replay-visible execution restrictions,
3. callable/provider interaction changes coordinator-visible no-publish or retry-relevant outcomes.

Until then:
1. no new OxCalc-facing seam packet is required,
2. OxCalc can continue reading callable narrowing as upstream semantic work rather than coordinator contract change.

## Expected Output Of The Next OxFunc Round
The next useful OxFunc reply should answer:
1. mandatory minimum callable carrier fields,
2. fields that may remain provenance or replay detail,
3. preferred callable invocation boundary,
4. callable-specific stage distinctions that must stay typed.

For the immediate next sync, OxFml wants those answers grounded in the currently exercised rows rather than in abstract carrier debate:
1. `LET`
2. `LAMBDA`
3. `MAP`
4. `REDUCE`
5. `SCAN`
6. `BYROW`
7. `BYCOL`
8. `MAKEARRAY`
9. `ISOMITTED` only if OxFunc thinks it is already necessary to avoid a misleading carrier lock

Current OxFml working question set for that sync:
1. are `origin kind`, `capture mode`, `arity shape`, and `invocation-contract meaning` all semantically required shared minimums,
2. can parameter-name, capture-name, and body-kind detail remain provenance-only for this round,
3. does OxFunc still need an explicit invocation-model field beyond an opaque callable identity plus invocation-contract-style reference,
4. should adopted defined-name callable origin already count as part of the minimum carrier discussion rather than a later extension.

## Exit Condition For This Prep Note
This prep note has served its purpose when either:
1. `W032` promotes a narrower shared callable carrier into the canonical seam docs with exercised evidence, or
2. OxFunc responds in a way that forces a narrower follow-up artifact or handoff packet.
