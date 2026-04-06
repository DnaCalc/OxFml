# Notes for OxReplay

Status: `active`
Owner lane: `OxFml`
Relationship: outbound replay-consumer and projection-facade note from OxFml for the next OxReplay integration round

## 1. Purpose
Record the current OxFml-side answer to OxReplay's replay-consumer and
projection-surface requests.

This note is an OxFml-owned consumer and packaging note.
It does not redefine OxReplay adapter semantics and it does not reopen the
frozen OxFml <-> OxFunc seam.

## 2. Core Message
OxFml has now turned the replay-facing consumer direction into one canonical
consumer packet:
1. `docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`

Navigation packet for OxReplay:
1. start with `docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
2. then read `docs/spec/OXFML_REPLAY_APPLIANCE_ADAPTER_V1.md`
3. then read `docs/spec/OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`
4. for replay-governance companions, use `docs/spec/fec-f3e/FEC_F3E_TESTING_AND_REPLAY.md`

Ordinary `OxFml_V1` entry surface for OxReplay:
1. `oxfml_core::consumer::replay`
2. `oxfml_core::consumer::runtime` only where replay projection begins from runtime result/session objects

Current OxFml message to OxReplay is:
1. replay projection is the preferred long-term consumer entry surface,
2. replay projection remains additive over OxFml semantic truth,
3. machine-readable projection metadata should become part of the preferred
   replay result rather than remaining private to consumers,
4. historical helper and adapter projection paths should be treated as
   implementation substrate while the replay facade is being strengthened, not
   as the intended long-term consumer contract.

Current OxFml read:
1. OxReplay is treated as aligned with this replay-facing direction at the
   contract level,
2. the consumer-facing replay contract is now landed and should be treated as
   the current OxFml replay surface,
3. the consumer contract is now treated as the landed `OxFml_V1` surface for
   downstream implementation.

## 3. Current OxFml Reply To OxReplay's Main Requests
OxFml's current answer is:
1. yes, OxReplay is right to ask for a narrower replay projection surface rather
   than broad helper and fixture-family dependence,
2. yes, source-case id to shared-scenario alias bindings should be published as
   machine-readable projection metadata,
3. yes, replay projection results should preserve replay-relevant metadata such
   as source schema, source family, pinned library-context reference, replay
   fence members when present, registry bindings, capability floor, and
   lifecycle metadata,
4. yes, runtime-result and first-host-capture projection now publish additive
   `comparison_views` for the current XML verification lane when
   `verification_publication_surface` facts are present,
5. no, this packaging work does not widen replay capability claims by itself;
   current capability stance remains as already documented.

## 4. Replay-Facade Direction
Current target replay facade shape is:
1. `ReplayProjectionRequest`
2. `ReplayProjectionService`
3. `ReplayProjectionResult`

Current required result truth is:
1. source case id
2. shared scenario alias when present
3. source schema id and source artifact family
4. pinned `LibraryContextSnapshotRef` when present
5. replay-relevant fence members when present
6. registry bindings and capability floor
7. lifecycle metadata when applicable
8. canonical replay envelope refs and sidecar refs
9. additive `comparison_views` entries for any admitted family OxFml can state directly

Current first-wave narrowing:
1. shared-scenario alias publication should be embedded directly in projection
   results in the first wave,
2. the current admitted XML verification family set is:
   - `visible_value`
   - `effective_display_text`
   - `formatting_view`
   - `conditional_formatting_view`
3. these remain adapter-declared facts built from `VerificationPublicationSurface`
   rather than host-local convenience strings,
4. session lifecycle is the preferred first post-FEC family for broader replay
   projection uptake.

## 5. Current Implementation Reality
Historical helper and adapter projection paths may still exist in code today, but:
1. the replay facade is the intended consumer contract,
2. broad internal artifact use should be treated as advanced provenance or
   schema work, not as the preferred long-term consumer integration path,
3. any temporary transition helper should be treated as a refactor aid to be
   removed once replay-facade coverage is strong enough.

## 6. Questions Back To OxReplay
The next useful OxReplay reply should answer:
1. whether the proposed replay projection result contains enough mandatory
   metadata for OxReplay intake,
2. which projection family should be the first post-FEC family after the
   current runtime/result wave,
3. whether OxReplay prefers shared-scenario alias publication as:
   - embedded fields in projection results,
   - or a dedicated machine-readable sidecar descriptor.

## 7. Current Public Surface Update
After the latest Wave 4 packaging cut, OxFml's current public consumer read is:
1. ordinary downstream use should target:
   - `consumer::runtime`
   - `consumer::editor`
   - `consumer::replay`
2. public `substrate::...` access is gone from the library surface,
3. any remaining host/session/adapter reach that still exists is explicit
   `test_support` support surface for bounded internal or integration-test use,
   not ordinary downstream integration contract.
