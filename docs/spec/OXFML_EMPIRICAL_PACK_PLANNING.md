# OxFml Empirical Pack Planning

## 1. Purpose
This document defines the current OxFml empirical-pack planning surface for host and oracle scenarios.

It exists to:
1. group exercised proving-host and oracle scenarios into future promotion families,
2. keep promotion blockers explicit,
3. avoid mistaking planning artifacts for pack-grade promotion.

Read together with:
1. `OXFML_TEST_LADDER_AND_PROVING_HOSTS.md`
2. `OXFML_DNA_ONECALC_HOST_POLICY_BASELINE.md`
3. `fec-f3e/FEC_F3E_TESTING_AND_REPLAY.md`

## 2. Current Planning Artifacts
The current machine-readable planning artifacts are:
1. `crates/oxfml_core/tests/fixtures/empirical_pack_planning/dna_onecalc_host_policy_profiles.json`
2. `crates/oxfml_core/tests/fixtures/empirical_pack_planning/empirical_pack_candidate_groups.json`

These artifacts are planning inputs only.
They do not authorize pack-grade promotion and they do not replace OxFml replay governance.

## 3. Scenario Grouping Rule
Future empirical-pack promotion should group scenarios by semantic pressure, not just by function name.

Current planning groups are:
1. scalarization and reference resolution,
2. helper invocation and callable-value lanes,
3. spill-shaped publication,
4. semantic formatting,
5. host-query environment lanes,
6. reference-sensitive host-query lanes.

## 4. Preservation Rules
Any future empirical-pack capture must preserve:
1. entered formula text,
2. stored formula text when different,
3. defined-name inputs,
4. direct cell bindings where semantic truth depends on concrete resolution,
5. host-query profile or capability assumptions,
6. locale and date-system context,
7. typed expected result summary and replay-facing consequence facts.

Direct cell bindings may not be collapsed into prose-only notes where they matter semantically.

## 5. Non-Goals In This Pass
This planning pass does not:
1. create pack-grade artifacts,
2. declare any empirical group promotion-ready,
3. authorize replay-safe rewrites,
4. define a new scenario DSL.

## 6. Promotion Blockers
Current promotion blockers remain:
1. local witness tier only,
2. missing pack-grade capture governance,
3. missing broader Excel-oracle pack authoring,
4. missing broader retained-witness promotion beyond the current local floor,
5. open runtime and semantic breadth outside the current exercised scenarios.

## 7. Relationship To Replay Governance
Empirical-pack planning is subordinate to OxFml replay governance.

That means:
1. retained-local witness rules still apply,
2. quarantined or explanatory-only witnesses remain non-pack-eligible,
3. empirical pack planning does not weaken replay-safe transform constraints,
4. pack planning remains planning-only until replay governance and evidence both permit promotion.

## 8. Working Rule
Use this document to organize future empirical scenario promotion without overstating current maturity.

Current empirical-pack planning is:
1. machine-readable,
2. tied to exercised host/oracle scenarios,
3. intentionally non-pack-grade.
