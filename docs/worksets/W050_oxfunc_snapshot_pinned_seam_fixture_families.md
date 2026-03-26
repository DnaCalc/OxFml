# W050: OxFunc Snapshot-Pinned Seam Fixture Families

## Purpose
Create the first pinned OxFml-owned seam fixture corpus for OxFunc, organized by seam family and tied to the committed OxFunc snapshot/export plus the real OxFml preparation adapter.

## Position and Dependencies
- **Depends on**: `W049`
- **Blocks**: `W032`
- **Cross-repo**: bounded OxFml -> OxFunc integration-artifact lane using the committed OxFunc snapshot/export and the converged first-freeze packets

## Scope
### In scope
1. Define the first pinned cross-seam fixture families OxFunc can consume against the real OxFml adapter.
2. Cover the admitted first seam families:
   - prepared-argument lanes,
   - callable lanes,
   - host/provider lanes,
   - snapshot/catalog lanes.
3. Keep the fixture corpus tied to explicit snapshot provenance and deterministic expectations.
4. Add an explicit matrix for deferred or blocked families that are not yet honest to include.
5. Use the fixture corpus to narrow concrete mismatch classes for later OxFml/OxFunc note rounds.
6. Start from OxFunc's current pinned first-wave table and project it into OxFml-owned harness/fixture form; the older note-only “38-scenario” wording should be read against the currently published consolidated table, which now enumerates 45 scenario ids.

### Out of scope
1. Full broad product regression corpus.
2. Pack-grade replay promotion.
3. Deferred runtime packets such as worksheet `CALL` / `REGISTER.ID`.
4. Rich-value/publication families that remain outside the first freeze.
5. Broad independent scenario proliferation before the pinned first wave is mapped and exercised.

## Deliverables
1. A canonical machine-readable fixture family corpus for OxFunc-facing seam tests.
2. Explicit snapshot provenance and pinning rules for that corpus.
3. A mismatch taxonomy for fixture failures that maps cleanly to:
   - OxFml-owned bug,
   - OxFunc-owned bug,
   - shared seam-freeze gap.
4. A deferred-family register for lanes intentionally excluded from the first fixture wave.
5. A direct mapping between OxFunc's pinned scenario groups and OxFml-owned harness cases.

## Gate Model
### Entry gate
- `W049` has produced a first real OxFml preparation adapter and consumer harness.

### Exit gate
- The first admitted seam families each have deterministic OxFml-owned fixture cases consumable by OxFunc.
- Snapshot pinning rules are explicit and exercised.
- Deferred families are listed explicitly instead of being silently absent.
- The stable first-wave scenario groups from OxFunc are either mapped directly or recorded as concrete mismatches/deferred lanes.

## Pre-Closure Verification Checklist

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

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - a first machine-readable OxFml-owned pinned fixture corpus now exists in consolidated form at `crates/oxfml_core/tests/fixtures/w050_oxfunc_pinned_fixture_corpus.json`, with the admitted subset retained in `crates/oxfml_core/tests/fixtures/w050_oxfunc_admitted_fixture_cases.json` and the explicit deferred register retained in `crates/oxfml_core/tests/fixtures/w050_oxfunc_deferred_fixture_register.json`
  - the current pinned corpus now covers the whole published first-wave table with 45 admitted scenarios and 0 explicit defers
  - mismatch classification is present structurally through the `W049` adapter artifacts, but not yet driven across a broader downstream packet-diff family set
  - the next bounded OxFunc-facing lane is now worksheet `CALL` / `REGISTER.ID` under `W052`, not residual first-wave coverage
- claim_confidence: provisional

## Current Local Floor
1. Deterministic harness coverage exists in `crates/oxfml_core/tests/w050_oxfunc_pinned_fixture_tests.rs`.
2. A consolidated pinned corpus artifact now exists in `crates/oxfml_core/tests/fixtures/w050_oxfunc_pinned_fixture_corpus.json`.
3. The current admitted pinned subset now covers 45 scenarios:
   - all prepared-argument lanes `A01`-`A10`
   - all implicit-intersection lanes `B01`-`B07`
   - all callable lanes `C01`-`C14`
   - all return-surface lanes `D01`-`D06`
   - all provider lanes `E01`-`E03`
   - all cross-seam lanes `F01`-`F05`
4. The earlier callable publication/reject residuals are now pinned as explicit OxFml rules:
   - bare top-level callable publication maps to worksheet `#CALC!`
   - duplicate `LET` binding names surface as bind-time `BindMismatch`
5. The current local floor should therefore be read as full OxFml-side coverage of the published pinned first-wave table rather than a broad partially-mapped wave.
6. OxFunc now explicitly confirms that the published pinned first-wave table is authoritatively 45 ids and that the current 45-admitted floor is acceptable as the current integration baseline.
