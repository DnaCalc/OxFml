# OxFml DNA OneCalc Host Policy Baseline

## 1. Purpose
This document defines the current OxFml-owned policy baseline for future DNA OneCalc host consumption.

It exists to separate:
1. OxFml semantic authority,
2. the current single-formula proving-host floor,
3. later DNA OneCalc host specification work.

Read together with:
1. `OXFML_TEST_LADDER_AND_PROVING_HOSTS.md`
2. `OXFML_EMPIRICAL_PACK_PLANNING.md`
3. `fec-f3e/FEC_F3E_TESTING_AND_REPLAY.md`

## 2. Authority Boundary
OxFml remains authoritative for:
1. formula parsing, bind, and semantic-plan meaning,
2. evaluator-owned candidate, commit, reject, and trace artifacts,
3. typed capability, effect, and reject semantics,
4. replay-safe identity and fence rules.

DNA OneCalc, as a downstream host, may own:
1. host-supplied input binding surfaces,
2. local recalc trigger policy,
3. host-query provider implementation,
4. packaging and user-facing harness policy.

DNA OneCalc must not:
1. redefine OxFml formula semantics,
2. collapse candidate versus publication boundaries,
3. replace typed reject outcomes with host-specific generic failures,
4. introduce scheduler-policy meaning into OxFml artifacts.

## 3. Current Supported Host Shape
The current OxFml proving-host baseline is a single-formula host with:
1. one formula under test,
2. mutable defined-name inputs,
3. mutable direct cell bindings where semantic truth depends on concrete cell resolution,
4. optional typed host-query capability/profile input,
5. locale and date-system context,
6. deterministic recalc producing candidate, commit, reject, and trace artifacts.

This is a proving-host baseline, not a full host-product definition.

## 4. Direct Cell Binding Rule
DNA OneCalc host policy must preserve direct cell bindings anywhere the exercised semantic lane depends on concrete cell state.

Current explicit cases include:
1. `@` scalarization,
2. `_xlfn.SINGLE`,
3. reference-sensitive `CELL(...)` lanes,
4. any future spill-linked or reference-sensitive host scenario with concrete cell resolution truth.

Defined names alone are insufficient for those lanes.

## 5. Typed Host Responsibilities
The downstream host is expected to supply typed context, not hidden side effects.

Current typed host responsibilities are:
1. defined-name value or reference bindings,
2. direct cell bindings where required,
3. typed host-query profile or provider access,
4. locale profile and date-system context,
5. recalc invocation boundary and requested backend choice.

The host should not leak:
1. raw workbook objects,
2. raw scheduler state,
3. ad hoc capability side channels.

## 6. Current Host Policy Profiles
The current planning profiles for DNA OneCalc host consumption are recorded in:
1. `crates/oxfml_core/tests/fixtures/empirical_pack_planning/dna_onecalc_host_policy_profiles.json`

These profiles are planning artifacts only.
They are not product configuration files and they do not authorize pack-grade promotion.

## 7. Relationship To OxCalc
DNA OneCalc remains a reduced-profile single-node proving host.

It differs from OxCalc-integrated hosting because it does not own:
1. multi-formula dependency coordination,
2. global scheduler policy,
3. multi-session publish arbitration,
4. broader workbook graph lifecycle policy.

That difference is intentional and does not change OxFml semantics.

## 8. Current Explicit Gaps
The following remain open beyond the current baseline:
1. full DNA OneCalc product specification,
2. host policy for broader workbook structure ownership,
3. broader external-provider and async host policy,
4. pack-grade empirical capture and promotion policy,
5. host behavior for later richer callable-value carriers.

## 9. Working Rule
Use this document to keep the current DNA OneCalc-facing host boundary explicit.

Do not use it to overclaim:
1. full host maturity,
2. full OxCalc equivalence,
3. pack-grade scenario promotion.
