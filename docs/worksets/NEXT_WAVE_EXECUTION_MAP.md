# OxFml Next-Wave Execution Map

## Purpose
This document records the historical execution map for the exercised `W025 -> W030` wave after the earlier `W019 -> W024` sequence.

It exists to keep the next wave dependency-driven and to make the largest remaining repo-level gaps visible without re-reading every workset.

## Historical Wave
The exercised `W025 -> W030` wave was:
1. `W025` pack-grade replay promotion baseline
2. `W026` library-context snapshot and availability taxonomy
3. `W027` callable-value and helper-transport narrowing
4. `W028` commit, publication, and topology breadth
5. `W029` runtime async and distributed consequences
6. `W030` semantic formatting and display boundary closure

Planned follow-on review lane after that wave:
7. `W031` `MS-OE376` formula and formatting rule review

## Historical Critical Path
1. `W025` pack-grade replay promotion baseline
2. `W026` library-context snapshot and availability taxonomy
3. `W028` commit, publication, and topology breadth
4. `W029` runtime async and distributed consequences
5. `W030` semantic formatting and display boundary closure

## Historical Parallelism
1. `W027` can proceed after `W026` without waiting for the full `W028 -> W029` runtime chain.
2. `W025` and `W026` attack different bottlenecks and should be kept conceptually distinct even if they overlap in replay-facing docs.
3. `W030` should not start early enough to mask unresolved publication/runtime consequence ownership from `W028` and `W029`.

## Why This Sequence
1. replay is still local rather than pack-grade, so `W025` addresses the broadest repo-level assurance gap first,
2. the latest OxFml/OxFunc exchange says library-context snapshot and availability taxonomy are the narrowest honest next seam topics, so `W026` comes before further callable transport narrowing,
3. commit/publication and topology breadth remain materially open at the repo level, so `W028` must be explicit rather than assumed as fallout from adjacent runtime work,
4. async/distributed runtime consequences should build on broader publication/topology truth rather than racing ahead of it,
5. the semantic-format versus display boundary should be narrowed only after publication/runtime consequence surfaces are stronger than they are today,
6. the broader `MS-OE376` review should follow once grammar, availability, and semantic-format boundaries are strong enough to classify incoming rule surfaces cleanly instead of dumping them into one generic parser backlog.

## Working Rule
1. do not skip directly to pack-grade replay claims while replay promotion criteria remain local-only
2. do not reopen OxFml/OxFunc transport narrowing indefinitely without a concrete trigger such as a field-set lock or proving-host pressure
3. keep callable transport narrowing downstream of library-context and availability closure
4. keep commit/publication ownership explicit in the runtime lane, not implied by adjacent seam work
5. keep semantic-format versus display closure coupled to actual publication/runtime evidence rather than prose-only clarification

## Post-W031 State
`W031` is now exercised and should be treated as the classification bridge between the earlier `W025 -> W030` wave and the next execution wave.

## Next Critical Path
1. `W032` OxFunc catalog, callable transport, and provider closure
2. `W041` typed context and query bundle freeze
3. `W042` return surface and publication-hint freeze
4. `W043` runtime library-context provider consumer model
5. `W034` distributed runtime and coordinator consequence boundary
6. `W035` broader formal family and concurrency model expansion
7. `W038` name and external-name formula carriers
8. `W039` conditional-formatting and data-validation formula sublanguages
9. `W040` higher-order callable evidence and seam reopen

## Next Parallelism
1. `W033` can proceed after `W025`, `W028`, `W029`, `W030`, and the now-exercised `W031` classification floor without waiting for the full `W032 -> W034` chain.
2. `W036` and `W037` can proceed after `W031` without waiting for the full runtime/distributed chain.
3. `W041`, `W042`, and `W043` should follow the now-converged OxFunc note sequence and can proceed as the next bounded seam-hardening packets after `W032`.
4. `W038` should follow the OxFunc/provider narrowing from `W032`.
5. `W039` should follow `W030` and `W034` so CF/DV semantics do not outrun current semantic-format and runtime consequence boundaries.
6. `W035` should wait for both the wider replay and runtime floors so the new checked artifacts match exercised local behavior rather than speculative designs.
7. `W040` should wait until `W032` has narrowed the first callable carrier/provenance split and `W038` has kept callable/name-carrier pressure explicit, so higher-order callable evidence reopens the seam on local proof rather than note pressure.

## Why This Next Sequence
1. `W031` has already converted the pending `MS-OE376` rule families into explicit OxFml-owned backlog and semantic classification,
2. `W032` directly addresses the narrowest active OxFml/OxFunc seam topics left open by the latest note exchanges,
3. the latest OxFunc round now makes three narrower successor packets explicit:
   - typed context/query bundle freeze
   - return-surface and publication-hint freeze
   - runtime library-context provider consumer model,
4. the final OxFunc note in that exchange treats those packets as the remaining first-application work rather than leaving further clarification debt,
5. `W033` addresses the largest remaining assurance gap by moving beyond promotion-readiness planning toward broader `cap.C4`-adjacent evidence,
6. `W034` takes the current local async/runtime floor into the next coordinator-visible consequence boundary without collapsing OxCalc policy into OxFml,
7. `W035` broadens checked local formal coverage only after replay and runtime surfaces are stronger than they are today,
8. `W036` turns structured references from a provisional rule into a wider local semantic floor,
9. `W037` adds the missing R1C1 formula channel instead of silently overloading the A1-first parser,
10. `W038` makes name and external-name formulas first-class carriers rather than generic scoped-name fallout,
11. `W039` treats conditional-formatting and data-validation formulas as distinct sublanguage lanes instead of assuming worksheet-formula parity,
12. `W040` gives the deferred higher-order callable lanes a future evidence path instead of forcing them into the current callable seam exchange prematurely.
