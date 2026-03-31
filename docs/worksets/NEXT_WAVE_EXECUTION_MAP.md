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

## Post-W047 State
`W047` is now exercised. The immediate first-host-readiness batch has removed the narrow `R1C1`, restricted `CF` / `DV`, and first-host replay-packet blockers from the next local host packet.

## Post-W040 State
`W040` is now exercised. The first higher-order callable seam-evidence wave is strong enough that the remaining callable work is smaller carrier/provenance freeze work under `W032`, `W041`, `W042`, and `W043`, not another broad higher-order seam-reopen round.

## Next Critical Path
1. `W032` OxFunc catalog, callable transport, and provider closure
2. `W042` return surface and publication-hint freeze
3. `W043` runtime library-context provider consumer model
4. `W049` OxFunc preparation adapter and consumer harness
5. `W050` OxFunc snapshot-pinned seam fixture families
6. `W051` OxCalc fixture host and stand-in packet
7. `W052` registered external provider and CALL REGISTER.ID boundary
8. `W053` grouped aggregation and publication-class adapter expansion
9. `W045` host runtime and external requirements freeze
10. `W034` distributed runtime and coordinator consequence boundary
11. `W035` broader formal family and concurrency model expansion
12. `W038` name and external-name host resolution boundary
13. `W048` editor language service and immutable formula host plan

## Next Parallelism
1. `W033` can proceed after `W025`, `W028`, `W029`, `W030`, and the now-exercised `W031` classification floor without waiting for the full `W032 -> W034` chain.
2. `W036` can proceed after `W031` without waiting for the full runtime/distributed chain.
3. `W042` and `W043` should continue to narrow in parallel where needed, because the host packet and the remaining OxFunc seam work still depend on them.
4. `W049` should start once `W042` and `W043` are strong enough to project into one real OxFml-backed integration adapter.
5. `W050` should follow `W049` so the first pinned seam-fixture corpus is tied to a real adapter rather than a synthetic packet-only model.
6. `W051` should narrow in parallel with `W049` / `W050` because some adapter-fixture inputs are really stand-in host/coordinator truths.
7. `W052` should narrow in parallel with later `W049` / `W050` follow-on work because worksheet `CALL` / `REGISTER.ID` is a new bounded OxFunc seam lane rather than part of the already-validated first adapter wave.
8. `W053` should narrow in parallel with `W052` because OxFunc's March 27 note separates grouped-aggregation and publication-class adapter expansion from the registered-external packet lane.
9. `W045` remains the canonical host/runtime contract while those successor packets keep narrowing.
10. `W038` should follow the host/runtime and provider narrowing and remain scoped to host-managed name/external-name resolution boundary work rather than generic formula-carrier ownership.
11. `W035` should wait for both the wider replay and runtime floors so the new checked artifacts match exercised local behavior rather than speculative designs.
12. `W048` should follow the current host/runtime and parser/binder packet strengthening rather than racing ahead of `W043` and `W045`.

## Why This Next Sequence
1. `W031` has already converted the pending `MS-OE376` rule families into explicit OxFml-owned backlog and semantic classification,
2. `W032` directly addresses the narrowest active OxFml/OxFunc seam topics left open by the latest note exchanges,
3. the latest OxFunc round narrowed the successor packet work into return-surface and runtime-library-context consumer closure after the now-exercised typed context/query bundle freeze,
4. the newest OxFunc note also asks for a real OxFml-backed evaluation adapter and pinned seam-fixture corpus so OxFunc can stop relying only on mock resolvers,
5. `W049` and `W050` therefore become the next concrete artifact-driven closure lanes after the remaining successor freeze packets,
6. some of those adapter-fixture inputs are actually stand-ins for OxCalc-owned host/coordinator truths, so `W051` narrows that reuse boundary explicitly instead of letting it remain implicit test scaffolding,
7. the latest OxFunc validation round leaves `W049` / `W050` largely mismatch-driven and isolates worksheet `CALL` / `REGISTER.ID` as the next bounded OxFunc seam topic, so `W052` becomes an explicit owner rather than leaving that lane as floating note debt,
8. OxFunc's March 27 note also adds a new bounded adapter-evidence lane for `GROUPBY`, `PIVOTBY`, helper bind-time rejection parity, and publication-sensitive return classes, so `W053` should own that expansion rather than overloading `W052`,
9. `W047` has already executed the immediate first-host readiness push, so the next sequence can return to the remaining seam, host-packet, runtime, and adapter-artifact owners,
10. `W045` turns the partial `W042` / `W043` packet floor into one implementation-facing host/runtime contract for both direct-host and OxCalc-integrated host use,
11. `W033` addresses the largest remaining assurance gap by moving beyond promotion-readiness planning toward broader `cap.C4`-adjacent evidence,
12. `W034` takes the current local async/runtime floor into the next coordinator-visible consequence boundary without collapsing OxCalc policy into OxFml,
13. `W035` broadens checked local formal coverage only after replay and runtime surfaces are stronger than they are today,
14. `W036` turns structured references from a provisional rule into a wider local semantic floor,
15. `W038` is now narrowed to host-managed name/external-name boundary work rather than generic formula-carrier ownership,
16. the now-exercised `W040` callable-evidence wave means the remaining callable work is narrower carrier/provenance freeze work rather than another broad higher-order evidence push.
17. editor-grade green-tree trivia, live language services, and immutable formula-host updates are a real next-wave extension, but they should build on the current parser/binder and host packet floor rather than inventing a separate architecture.
