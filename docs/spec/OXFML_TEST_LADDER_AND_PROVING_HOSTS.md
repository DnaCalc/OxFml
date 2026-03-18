# OxFml Test Ladder and Proving Hosts

## 1. Purpose
This document defines the canonical OxFml test ladder and the proving-host model used to exercise the lane before full multi-node integration.

It exists to make explicit:
1. the minimal local bootstrap evaluator surface inside OxFml,
2. the boundary between OxFml-local testing and OxFunc-backed semantic testing,
3. the single-formula host model that precedes broader DNA OneCalc specification work,
4. the role of Excel empirical runs as behavior oracle for whole-formula semantics.

Read together with:
1. `OXFML_IMPLEMENTATION_BASELINE.md`
2. `OXFML_HIGH_RISK_AND_EARLY_ATTENTION_AREAS.md`
3. `OXFML_DNA_ONECALC_HOST_POLICY_BASELINE.md`
4. `OXFML_EMPIRICAL_PACK_PLANNING.md`
5. `fec-f3e/FEC_F3E_TESTING_AND_REPLAY.md`
6. `fec-f3e/FEC_F3E_SCHEMA_REPLAY_FIXTURE_PLAN.md`

## 2. Working Rule
OxFml should not wait for a full downstream function universe before it can test its own parser, binder, evaluator, and seam surfaces.

At the same time, OxFml should not re-implement OxFunc.

The test ladder therefore separates:
1. a very small local bootstrap semantic kernel,
2. OxFunc-backed downstream semantic execution,
3. host-level formula proving,
4. empirical Excel-oracle verification.

## 3. Canonical Test Ladder
The current canonical ladder is:

### 3.1 Layer 1: Local Unit and Artifact Fixtures
Purpose:
1. validate parser, green/red, binder, normalized references, schema objects, and trace shapes.

Typical coverage:
1. syntax fidelity,
2. bind normalization,
3. artifact identity/version behavior,
4. minimum schema objects,
5. candidate/commit/reject/trace structural fixtures.

### 3.2 Layer 2: Minimal Local Bootstrap Evaluator
Purpose:
1. let OxFml exercise evaluator-owned behavior quickly without depending on the full OxFunc function surface.

Boundary rule:
1. this layer is intentionally tiny,
2. it exists only to bootstrap OxFml-owned testing and benchmark loops,
3. it must not become a shadow OxFunc.

Current intended scope:
1. literals,
2. basic operators,
3. a tiny fixture function set or probe/test-only functions,
4. defined-name lookup with mutable supplied values,
5. enough execution to exercise parse -> bind -> evaluate -> candidate/commit/reject/trace paths.

### 3.3 Layer 3: OxFunc-Backed Semantic Execution
Purpose:
1. test OxFml prepared-call/result behavior against the real downstream function-semantic lane.

Boundary rule:
1. beyond the minimal bootstrap kernel, OxFml should use OxFunc outputs rather than re-implementing function semantics locally.

Expected inputs from OxFunc:
1. function definitions and traits,
2. prepared-call/result expectations,
3. semantic baselines or fixture outputs where available.

### 3.4 Layer 4: Single-Formula Recalc Host
Purpose:
1. exercise hosting, update, recompute, and FEC/F3E behavior in a controlled proving environment.

The current intended scope is:
1. one formula under test,
2. no upstream formula dependency graph,
3. defined names, direct cell bindings, or host-supplied bindings as mutable inputs,
4. full recompute or full update semantics,
5. candidate/commit/reject/trace behavior,
6. enough host structure to model caller context, profile, locale, date-system, host-query capabilities, and artifact reuse where needed.

Current exercised local floor:
1. defined-name update and reuse-sensitive recalc,
2. reference-sensitive scalarization through `@` and `_xlfn.SINGLE`,
3. helper-form evaluation through `LET` and callable `LAMBDA`,
4. spill-shaped publication through `SEQUENCE`,
5. formatting-sensitive host runs through `TEXT`,
6. host-query-sensitive host runs through `INFO` and `CELL("filename", ...)`.

This is the direct precursor to the later DNA OneCalc host specification.
It is an OxFml proving-host model first, not a full host/product definition.
The current DNA OneCalc-facing host policy baseline is recorded in:
1. `OXFML_DNA_ONECALC_HOST_POLICY_BASELINE.md`

### 3.5 Layer 5: Excel Empirical Oracle Runs
Purpose:
1. use Excel behavior as the executable oracle for formula-level semantics.

Current target use:
1. parse/normalization behavior,
2. formula evaluation behavior in the single-formula host model,
3. `@`, `#`, `SINGLE`, `LET`, `LAMBDA`, host-query, spill, and formatting-sensitive lanes,
4. update/recalc behavior when defined-name inputs change.

Current exercised local floor:
1. `TEXT` locale-sensitive formatting,
2. `INFO("directory")` host-query semantics,
3. `CELL("filename", ref)` host-query semantics with typed reference input,
4. `@` and `_xlfn.SINGLE` scalarization over reference-like inputs,
5. helper-form invocation through `LET` and `LAMBDA`,
6. spill-shaped array publication through `SEQUENCE(2)`.

Rule:
1. empirical Excel runs are not implementation substitutes,
2. they are the behavior oracle for disputed or under-specified formula semantics.

Current machine-readable empirical-pack planning artifacts are:
1. `crates/oxfml_core/tests/fixtures/empirical_pack_planning/dna_onecalc_host_policy_profiles.json`
2. `crates/oxfml_core/tests/fixtures/empirical_pack_planning/empirical_pack_candidate_groups.json`

### 3.6 Layer 6: Replay and Formal Witnesses
Purpose:
1. turn tested behavior into durable replay, Lean, and TLA+ witness artifacts.

This layer closes the loop between:
1. local fixtures,
2. bootstrap evaluator runs,
3. OxFunc-backed runs,
4. single-formula host runs,
5. Excel oracle comparisons.

## 4. Minimal Local Bootstrap Evaluator Rule
The minimal local bootstrap evaluator must stay intentionally constrained.

Allowed goals:
1. quick OxFml-owned regression checks,
2. parser/binder/evaluator path bring-up,
3. seam payload and trace bring-up,
4. benchmark and profiling harnesses for OxFml-owned code paths.

Disallowed drift:
1. broad function-family reimplementation,
2. independent semantic ownership for real Excel functions,
3. divergence from OxFunc function semantics for non-fixture lanes.

## 5. Single-Formula Proving Host Model
The current proving-host model should support:
1. one formula source record,
2. one green/root and bind/semantic-plan path,
3. mutable defined-name inputs supplied by the host,
4. mutable direct cell bindings where a reference-sensitive formula needs concrete cell resolution,
5. explicit recalc trigger and full recompute semantics,
6. replay-stable candidate, commit, reject, and trace outputs.

Working rule:
1. direct cell bindings are not an optional convenience where reference-sensitive truth depends on concrete cell state,
2. host models that omit them should not claim coverage of scalarization, spill-linked, or host-query lanes that require real cell resolution.

It should not require:
1. a workbook-wide formula graph,
2. dependency closure across multiple formulas,
3. OxCalc scheduler policy.

## 6. Empirical Oracle Scaffolding Rule
OxFml should have empirical scaffolding similar in spirit to OxFunc's Excel-compat runs, but formula-oriented.

The scaffolding should make it easy to capture:
1. entered formula,
2. stored formula if different,
3. bound or normalized context summary,
4. input binding set for defined names and any required direct cell bindings,
5. observed Excel result class and value,
6. any relevant host/query/format context,
7. reproducible scenario ids for replay comparison.

Working rule:
1. empirical-oracle scenarios should not hide direct cell state inside ad hoc prose when that state is semantically required,
2. if a scenario depends on concrete cell resolution, the cell bindings belong in the scenario artifact.

## 7. Workset Implications
Current expected primary owners:
1. `W002`: local unit and artifact fixtures; minimal bootstrap evaluator framing; single-formula host artifact model
2. `W003`: OxFunc-backed semantic execution boundary and fixture planning
3. `W004`: single-formula recalc host behavior through FEC/F3E and schema fixtures
4. `W005`: replay/formal witnesses for the ladder outputs
5. `W006`: formatting and host-query proving scenarios within the same ladder

## 8. Working Rule
Before implementation broadens:
1. make the ladder explicit,
2. keep the local bootstrap evaluator minimal,
3. use OxFunc for real downstream semantic breadth,
4. build the single-formula host model before broader host assumptions,
5. keep Excel empirical runs as the behavior oracle for whole-formula semantics.

## 9. Current Local Witness Floor
The current local witness floor for the ladder is:
1. Layer 1 parse/bind fixtures: `crates/oxfml_core/tests/fixtures/parse_bind_cases.json`
2. Layer 3 OxFunc-backed prepared-call/result fixtures: `crates/oxfml_core/tests/fixtures/prepared_call_replay_cases.json`
3. Layer 4 single-formula host fixtures: `crates/oxfml_core/tests/fixtures/single_formula_host_replay_cases.json`
   Current exercised lanes: reuse-sensitive recalc, `@`, `_xlfn.SINGLE`, `LET`, `LAMBDA`, `SEQUENCE`, `TEXT`, `INFO`, and `CELL("filename", ...)`
4. Layer 5 empirical-oracle scenario shapes: `crates/oxfml_core/tests/fixtures/empirical_oracle_scenarios.json`
   Current exercised lanes: formatting, host-query, scalarization, helper forms, spill publication, and seam-significant `format_delta` / `display_delta`
5. Layer 6 execution/replay fixtures: `crates/oxfml_core/tests/fixtures/semantic_plan_replay_cases.json`, `crates/oxfml_core/tests/fixtures/fec_commit_replay_cases.json`, and `crates/oxfml_core/tests/fixtures/execution_contract_replay_cases.json`

Current proving-host discipline:
1. direct cell bindings are preserved where scalarization, host-query, or reference-sensitive replay depends on them,
2. host and empirical fixtures should expand in the same wave when new seam-significant format, display, or topology facts are added,
3. promotion-readiness planning for host and empirical families remains local-only until broader replay promotion work authorizes more.
