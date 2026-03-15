# W002: Parser, Binder, and Artifact Core Baseline

## Purpose
Establish the first implementation-start baseline for OxFml syntax and binding.

This workset is the bridge from the current spec reset into executable parser and binder work.

## Position and Dependencies
- **Depends on**: `W001`
- **Blocks**: `W003`, `W004`, `W005`, `W006`
- **Cross-repo**: none required for startup; may emit observations to OxCalc if immutable artifact ownership assumptions need narrowing

## Scope
### In scope
1. Finalize the canonical parser and binder realization baseline against the current spec set.
2. Define the first implementation-facing ADTs for formula source records, green roots, red projections, normalized references, and bound formulas.
3. Define the initial incremental reparse and rebind keying rules for `formula_token` and `structure_context_version`.
4. Define the first replay fixture families for parse and bind artifacts.
5. Define the single-formula host artifact model needed by later proving-host work.
6. Narrow the normalized reference ADT baseline enough that implementation can start without reopening reference shape categories.

### Out of scope
1. FEC/F3E runtime services.
2. OxFunc prepared-call/result implementation.
3. Formal proof or model artifacts beyond planning/register updates.

## Deliverables
1. Parser/binder realization docs tightened enough that code can start without reopening architectural ownership questions.
2. Canonical ADT and artifact-shape decisions for syntax/bind surfaces.
3. A first implementation plan for incremental reparse and rebind reuse keys.
4. Updated workset/status tracking reflecting the handoff from W001 planning to parser/binder execution.
5. Host-facing artifact assumptions explicit enough for a single-formula proving host.
6. A normalized reference ADT baseline explicit enough for implementation-start and replay-fixture authoring.

## Gate Model
### Entry gate
- `W001` bootstrap/spec reset remains the active baseline.
- Parser/binder realization and implementation baseline docs exist in the live bootstrap set.
- The normalized reference ADT note is in the live bootstrap set.

### Exit gate
- Parser and binder artifact surfaces are explicit enough for implementation-start.
- Formula-text change versus structure-context change handling is explicit enough for reparse/rebind work.
- Initial parse/bind replay fixture families are declared.
- Any new artifact ownership assumptions are reflected in the canonical docs.
- Normalized reference atom and reference-expression families are explicit enough for implementation-start.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | no |
| 2 | Conformance matrix rows updated? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Cross-repo impact assessed and handoff filed if needed? | no |
| 5 | All required tests pass? | no |
| 6 | No known semantic gaps remain in declared scope? | no |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | no |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | no |

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes: initial parser/binder subset exists but grammar coverage is still narrow; parse/bind replay fixtures are scaffolded but not yet replay-grade artifacts; normalized reference internal encoding and repository shape still open
- claim_confidence: draft
