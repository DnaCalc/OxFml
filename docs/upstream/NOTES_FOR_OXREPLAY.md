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
2. the remaining replay-facade work is implementation and metadata export
   realization, not broad contract disagreement,
3. the consumer contract is now treated as provisionally frozen for the next
   implementation wave.

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
4. no, this packaging work does not widen replay capability claims by itself;
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

Current first-wave narrowing:
1. shared-scenario alias publication should be embedded directly in projection
   results in the first wave,
2. session lifecycle is the preferred first post-FEC family for broader replay
   projection uptake.

## 5. Current Implementation Reality
Historical helper and adapter projection paths still exist in code today, but:
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
