# W013: Parser Binder Breadth and Incremental Reuse

## Purpose
Broaden the current parser/binder floor beyond the initial subset and add the first exercised incremental reuse path for green/red and bind artifacts.

## Position and Dependencies
- **Depends on**: `W002`
- **Blocks**: `W014`, `W016`, `W018`
- **Cross-repo**: OxFunc inbound observations remain active semantic pressure for provenance-preserving parse and bind output shape

## Scope
### In scope
1. Broaden parser coverage for additional reference and formula-language surface forms.
2. Broaden binder coverage for qualified references, richer unresolved cases, and reference normalization breadth.
3. Add the first incremental reparse and incremental rebind reuse surfaces over immutable artifacts.
4. Extend local replay fixtures and parser/binder tests for the widened surface.

### Out of scope
1. Full Excel grammar closure.
2. Pack-grade replay promotion.
3. Full semantic-function coverage.

## Deliverables
1. Broader exercised parser/binder implementation slices.
2. First incremental reuse fixtures and tests.
3. Updated formula-language and implementation planning docs reflecting the widened artifact floor.

## Gate Model
### Entry gate
- `W002` baseline parser, binder, and artifact-core slices are exercised locally.

### Exit gate
- The parser/binder floor is materially broader than the initial subset.
- Incremental reuse is exercised by deterministic local fixtures.
- The widened parser/binder floor is reflected in the live planning/spec docs.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | yes |
| 2 | Conformance matrix rows updated? | yes |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes |
| 4 | Cross-repo impact assessed and handoff filed if needed? | yes |
| 5 | All required tests pass? | yes |
| 6 | No known semantic gaps remain in declared scope? | yes |
| 7 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | yes |

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: partial
- open_lanes: full Excel grammar closure, structured-reference breadth, and external-reference breadth remain outside this workset scope
- claim_confidence: validated
