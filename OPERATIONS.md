# OPERATIONS.md — OxFml Operations

## 1. Purpose
Define daily execution rules for formula-language and evaluator seam delivery.

## 2. Operating Principles
1. Seam determinism before optimization.
2. No hidden protocol semantics: contract text and traces must match.
3. Reject semantics must be typed and replay-stable.
4. Formula-semantic formatting behavior must be evaluated in the seam path, not deferred to display-only layers.

## 3. Execution Lanes
1. Formula-language lane: grammar, parsing, bind/reference normalization.
2. Evaluator lane: `prepare -> open_session/capability_view -> execute -> commit` lifecycle.
3. Seam contract lane: commit deltas, reject taxonomy, trace schema.
4. Conformance lane: matrix updates, scenario packs, replay evidence.

## 4. Required Packs (baseline)
1. `PACK.fec.commit_atomicity`
2. `PACK.fec.reject_detail_replay`
3. `PACK.fec.overlay_lifecycle`
4. `PACK.fec.format_dependency_tokens`
5. `PACK.format.semantic_vs_display_boundary`

## 5. Cross-Repo Handoff Rule
When OxFml changes coordinator-facing semantics:
1. file a handoff packet to OxCalc,
2. include exact proposed normative text,
3. include replay/evidence links,
4. include migration impact and fallback policy.

## 6. Promotion Gate
No seam change is promoted without:
1. updated spec text,
2. updated matrix rows,
3. at least one deterministic replay artifact proving intended behavior,
4. explicit statement of impact on OxCalc coordinator logic.

## 7. Pre-Closure Verification Checklist

Before claiming any workset or feature item as complete, answer each item yes or no.
All items must be "yes" for a completion claim. Any "no" means the item is `in_progress`.

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | |
| 2 | Conformance matrix rows updated? | |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | |
| 4 | Cross-repo impact assessed and handoff filed if needed? | |
| 5 | All required tests pass? | |
| 6 | No known semantic gaps remain in declared scope? | |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | |

## 8. Expanded Definition of Done

A workset or feature item is done for its declared scope only when all of the following hold:

1. **Seam contract text**: all in-scope FEC/F3E contract text is updated and internally consistent.
2. **Conformance matrix**: all affected matrix rows are updated with evidence links.
3. **Replay artifact**: at least one deterministic replay artifact per in-scope behavior proves intended semantics.
4. **Cross-repo impact**: impact on OxCalc coordinator logic is assessed; handoff packet filed if coordinator-facing clauses changed.
5. **Trace schema**: trace emission for in-scope behaviors conforms to the declared schema.
6. **Reject taxonomy**: reject codes for in-scope failure modes are typed and replay-stable.
7. **No semantic gaps**: no known semantic gap remains between spec and exercised behavior for declared scope.
8. **Three-axis report**: completion report includes `scope_completeness`, `target_completeness`, `integration_completeness`, and `open_lanes` per AGENTS.md Section 3.
9. **Checklist attached**: Pre-Closure Verification Checklist (Section 7) is filled in and all items are "yes".

## 9. Completion Claim Self-Audit

Before submitting a completion claim, the agent must perform this self-audit and include the results.

### Step 1: Scope Re-Read
Re-read the workset scope declaration. For each in-scope item, verify that exercised implementation (not scaffolding) matches. Any missing item = `in_progress`.

### Step 2: Gate Criteria Re-Read
Re-read the workset gate criteria. All pass criteria must be met. Any unmet criterion = gate open.

### Step 3: Silent Scope Reduction Check
Compare the original scope declaration with what was actually delivered. Any unreported narrowing of scope is a doctrine violation. If scope was intentionally narrowed, it must be explicitly documented with rationale.

### Step 4: "Looks Done But Is Not" Pattern Check
Check for these patterns:
- Stubs or placeholder implementations reported as real.
- Insufficient test coverage masking untested paths.
- Spec text that does not match exercised implementation.
- Handoffs filed but not acknowledged by receiving repo.

### Step 5: Include Result
Include the self-audit result in the completion report with explicit pass/fail for each step.

## 10. Report-Back Completeness Contract

Every completion report (status updates, workset closure notes, handoff summaries) must include:

1. `execution_state`: `planned` | `in_progress` | `blocked` | `complete`
2. `scope_completeness`: `scope_complete` | `scope_partial`
3. `target_completeness`: `target_complete` | `target_partial`
4. `integration_completeness`: `integrated` | `partial`
5. `open_lanes`: explicit list when any completeness axis is partial

Normative wording rules:
1. Use `complete for declared scope` only when the declared scope already represents full known semantics and only integration or external limits remain partial.
2. Do not use `complete for declared scope` for semantically bounded subsets that still carry known gaps; report those as `scope_partial`.
3. Do not claim `fully complete` unless all three completeness axes are complete and evidence links are present.

## 11. Carried-Forward Operating Lessons

These five lessons are derived from observed execution failures in OxVba (86+ worksets) and OxFunc (13 worksets). They are not speculative — each addresses a real failure mode.

A separate `docs/LOCAL_EXECUTION_DOCTRINE.md` will be created when locally-observed lessons emerge from actual OxFml execution.

### Lesson 1: Scaffold Determinism Is a Gate
Scaffolding (stubs, empty traits, compile-only code) must produce deterministic outputs or be explicitly marked non-functional. Non-deterministic scaffolding that silently passes tests is a gate failure.
*Source: OxVba Lesson 1.*

### Lesson 2: Spec Drift Checks Run Alongside Implementation
Do not defer spec-vs-implementation consistency checks to a separate phase. Run them as part of each workset execution. Spec drift discovered late is expensive to reconcile.
*Source: OxVba Lesson 3.*

### Lesson 3: Final Validation Must Not Mutate Tracked Evidence
Validation runs must not modify the artifacts they are validating. Evidence mutation during validation invalidates the evidence chain.
*Source: OxVba Lesson 9.*

### Lesson 4: Guard Artifact Scope Before Commit
Before committing, verify that only intended artifacts are staged. Accidental inclusion of generated files, temporary outputs, or out-of-scope changes pollutes the evidence record.
*Source: OxVba Lesson 12.*

### Lesson 5: Partial Semantics Are Not Implementation
A function, protocol, or contract that covers a subset of its declared semantic space is work-in-progress, not an implementation. This applies even if the subset compiles, passes tests, and looks correct for the covered cases.
*Source: OxFunc doctrine decision.*

### Lesson 6: Repeated Replay Facts Must Be Promoted Early
If the same semantic fact is needed in multiple replay families or multiple downstream-facing interpretations, it should be promoted into a canonical artifact rather than remaining trace-only or test-only detail.
Typical examples include helper-environment shape, lexical-capture need, and other formula-level facts that affect both replay and boundary coordination.
*Source: OxFml W003 helper-form execution.*

### Lesson 7: Canonical Artifact Tests Should Prefer Builders Over Manual Struct Literals
When canonical artifacts evolve, hand-written struct literals create noisy, low-signal breakage that obscures the real semantic change. Core artifact tests should prefer fixture/builders or factory helpers wherever practical.
This is especially important for `SemanticPlan`, commit artifacts, and other schema-bearing types expected to grow over time.
*Source: OxFml W003 helper-profile promotion.*

## 11A. Artifact Construction Discipline

For tests and local proving artifacts:
1. prefer fixture/builders or narrow constructor helpers for canonical artifact families,
2. avoid manual full-struct literals for evolving artifacts unless the test is explicitly validating field-by-field schema shape,
3. when a manual literal is necessary, keep it local to schema-focused tests rather than scattering it across unrelated tests.

Rationale:
1. this keeps schema evolution noise small,
2. it makes semantic regressions easier to identify,
3. it reduces accidental coupling between unrelated tests and incidental artifact fields.

## 12. Upstream Observation Ledger Protocol

### 12.1 Purpose
Repos that depend on OxFml discover interface and design constraints through their own implementation work. Those observations must flow back to OxFml through a structured channel so they inform design before contracts solidify.

This is distinct from handoff packets (Section 5), which propose specific normative text changes. Observation ledgers are standing documents that accumulate design feedback over time.

### 12.2 Inbound Observation Sources
OxFml must check for inbound observation ledgers from consumer repos at the start of any design or interface workset. Known source locations:

| Source repo | Ledger location | Relationship |
|-------------|----------------|--------------|
| OxFunc | `../OxFunc/docs/upstream/NOTES_FOR_OXFML.md` | Function-semantic interface constraints |
| OxCalc | `../OxCalc/docs/upstream/NOTES_FOR_OXFML.md` | Coordinator-facing contract constraints |

### 12.3 Outbound Observations
When OxFml implementation work reveals design constraints that affect a downstream or sibling repo, write them to `docs/upstream/NOTES_FOR_<REPO>.md` following this structure:

1. **Purpose**: what the consuming repo needs to know and why.
2. **Core message**: the essential design constraint in 2-3 sentences.
3. **Current evidence**: specific examples with concrete scenarios.
4. **Interface implications**: what the receiving repo must preserve, avoid, or expose.
5. **Minimum invariants**: binary testable statements.
6. **Open questions**: explicit questions the receiving repo should answer.

### 12.4 Lifecycle
1. Observation ledgers are living documents — updated as new evidence accumulates.
2. Entries are never silently removed; outdated observations are marked superseded with rationale.
3. When an observation is addressed by the receiving repo (through spec changes, interface decisions, or handoff packets), the originating entry is updated with a resolution reference.
4. Observation ledgers are not completion artifacts — they do not close worksets or satisfy gate criteria. They are design inputs.

### 12.5 Agent Obligation
Agents starting work on OxFml interface or contract design must:
1. Check all listed inbound observation sources (Section 12.2).
2. Note any unresolved observations that are relevant to current scope.
3. Include a "reviewed inbound observations" line in the workset status report.
4. When a design decision addresses an inbound observation, reference the observation entry explicitly.
