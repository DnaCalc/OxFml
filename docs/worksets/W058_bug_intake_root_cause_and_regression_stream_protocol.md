# W058: Bug Intake Root-Cause And Regression Stream Protocol

## Purpose
Establish a canonical OxFml bug-tracking and processing protocol so formula-language, evaluator, publication, and replay bugs are recorded against an exact repo ref, triaged by ownership, grouped into canonical bug streams, and carried through fix, root-cause analysis, and similar-risk scanning with a deterministic operating record.

## Position and Dependencies
- **Depends on**: `W054`
- **Blocks**: ad hoc bug handling drift, duplicate bug-report ambiguity, and untracked regression-family reopenings
- **Cross-repo**: only when a bug stream reveals coordinator-facing seam change or sibling ownership handoff

## Scope
### In scope
1. Define the canonical bug intake and lifecycle protocol in `OPERATIONS.md`.
2. Add machine-readable scaffolding for:
   - individual bug reports,
   - canonical bug streams,
   - duplicate/shortcut linkage between reports and streams.
3. Require exact source-ref capture for every bug report using:
   - release/version id when available,
   - otherwise exact git commit.
4. Define mandatory bug-processing stages:
   - intake,
   - reproduction,
   - ownership classification,
   - root-cause analysis,
   - similar-risk scan,
   - fix/evidence,
   - closure or handoff.
5. Define status vocabulary for bug reports and canonical bug streams.
6. Add templates and directory scaffolding so future bug work uses one repo-native shape rather than ad hoc notes.

### Out of scope
1. Solving any specific bug stream beyond the protocol/setup work itself.
2. Replacing `CURRENT_BLOCKERS.md` for validation blockers.
3. Replacing worksets for bounded implementation scope.
4. Cross-repo bug tracker synchronization automation.

## Deliverables
1. An `OPERATIONS.md` bug-protocol section with explicit lifecycle rules.
2. `docs/bugs/README.md`.
3. `docs/bugs/BUG_REPORT_REGISTER.csv`.
4. `docs/bugs/BUG_STREAM_REGISTER.csv`.
5. `docs/bugs/BUG_REPORT_TEMPLATE.md`.
6. `docs/bugs/BUG_STREAM_TEMPLATE.md`.
7. Empty `docs/bugs/reports/` and `docs/bugs/streams/` scaffolding with `.gitkeep`.

## Gate Model
### Entry gate
- OxFml currently handles bug-driven work through worksets, blockers, handoffs, and ad hoc notes, but it does not yet have a single canonical bug-report and canonical-bug-stream protocol.

### Exit gate
- `OPERATIONS.md` defines the required bug intake and lifecycle sequence.
- Exact source-ref capture is mandatory for all new bug reports.
- Duplicate bug reports can be linked into a canonical known-bug stream without losing the original report record.
- Root-cause classification and similar-risk scanning are explicit required steps, not optional follow-up.
- The repo contains templates and registers that future work can populate immediately.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | yes |
| 2 | Conformance matrix rows updated? | n/a |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | n/a |
| 4 | Cross-repo impact assessed and handoff filed if needed? | yes |
| 5 | All required tests pass? | n/a |
| 6 | No known semantic gaps remain in declared scope? | yes |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | n/a |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | yes |

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - future bug streams still need to populate the new registers
  - no automation yet exists for duplicate detection or commit-range inference
- claim_confidence: provisional
