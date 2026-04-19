# OxFml/OxFunc Evaluation Adapter And Test Artifacts

## Purpose
Define the first OxFml-side proposal for the concrete integration artifacts OxFunc can use to test the real OxFml preparation pipeline rather than only OxFunc-local mock resolvers.

This document does not freeze the final cross-process ABI.
It defines the first bounded implementation-facing artifact family for seam closure and mismatch discovery.

## Why This Exists
The current OxFml/OxFunc note exchange is converged enough that the next useful work is not more abstract seam discussion.

The next useful work is:
1. a real OxFml-backed evaluation adapter,
2. pinned cross-seam fixture families,
3. mismatch-driven refinement of the remaining frozen packets under:
   - `W041`
   - `W042`
   - `W043`

## Boundary Position
The evaluation adapter should:
1. drive the real OxFml parse -> bind -> semantic-plan -> prepare path,
2. preserve the currently frozen or freeze-candidate seam packets,
3. avoid inventing a workbook-object contract for OxFunc,
4. use the pinned OxFunc library-context snapshot/export for test pinning while still allowing the runtime `LibraryContextProvider` model to be exercised in parallel.

The adapter should not:
1. become the normative production host API,
2. force OxFunc to consume raw OxFml parser internals,
3. bypass the current `TypedContextQueryBundle`, `ReturnedValueSurface`, or runtime snapshot/provider lanes.

## Current Local OxFml Floor
The first local `W049` floor now exists in:
1. `crates/oxfml_core/src/oxfunc_adapter/mod.rs`
2. `crates/oxfml_core/tests/w049_oxfunc_adapter_tests.rs`

Current exercised local behavior:
1. canonical adapter request packet carrying:
   - formula text
   - formula channel
   - caller anchor
   - optional active selection anchor
   - direct cell fixture
   - optional defined-name bindings
   - optional table packet
   - typed context/query bundle
   - optional runtime library-context provider plus optional pinned snapshot ref
   - optional host-query capability profile
2. preparation artifact projection carrying:
   - syntax/bind/semantic diagnostics
   - typed query bundle spec
   - pinned library-context snapshot ref
   - execution profile and availability summaries
   - prepared-call frames from the real `EvaluationTrace`
3. end-to-end evaluation artifact projection carrying:
   - prepared result
   - worksheet-visible value
   - `ReturnedValueSurface`
   - explicit typed `execution_outcome_surface` for executed-result versus bind/commit-boundary rejection classification
   - candidate/commit identity and trace-event kinds
4. structured mismatch artifact projection carrying:
   - fixture case id
   - snapshot ref
   - failing seam family
   - packet family
   - narrowed owner guess
5. first local deterministic harness coverage for:
   - direct-scalar vs array-like preparation
   - caller-anchor carriage
   - pinned snapshot selection over a runtime provider
   - typed `RTD` host/provider outcome projection
   - structured mismatch packet construction

Current non-claims:
1. this is not yet the final OxFml/OxFunc seam closure,
2. this is not yet the worksheet `CALL` / `REGISTER.ID` closure lane,
3. this does not yet exercise deferred `CALL` / `REGISTER.ID` lanes.

## Current Local `W050` Floor
The first local pinned seam-fixture family corpus now exists in:
1. `crates/oxfml_core/tests/fixtures/w050_oxfunc_admitted_fixture_cases.json`
2. `crates/oxfml_core/tests/fixtures/w050_oxfunc_deferred_fixture_register.json`
3. `crates/oxfml_core/tests/fixtures/w050_oxfunc_pinned_fixture_corpus.json`
4. `crates/oxfml_core/tests/w050_oxfunc_pinned_fixture_tests.rs`

Current exercised local behavior:
1. the admitted subset is machine-readable and adapter-driven rather than prose-only,
2. the deferred subset is now empty rather than silently absent,
3. a consolidated pinned corpus artifact now exists for downstream consumption rather than forcing consumers to stitch the split registers together,
4. the current admitted floor now covers all 45 scenarios across the published pinned first-wave table:
   - all prepared-argument lanes `A01`-`A10`
   - all implicit-intersection lanes `B01`-`B07`
   - all callable lanes `C01`-`C14`
   - return-surface lanes `D01`-`D06`
   - provider lanes `E01`-`E03`
   - cross-seam lanes `F01`-`F05`
5. the earlier `C12` and `C14` callable publication/reject residuals are now exercised as explicit OxFml rules rather than remaining in the deferred register.

## Current OxFunc Acceptance Read
OxFunc now accepts this adapter lane in principle and narrows the first exercised wave.

Current converged first-wave reading:
1. the adapter is a bounded test/integration artifact, not the normative production API,
2. the committed `W044` export remains the first pinning artifact,
3. the first wave should be driven by OxFunc's currently published pinned scenario table, and OxFunc now explicitly confirms that the authoritative published table enumerates 45 ids,
4. `formula_channel` may remain present but is not first-wave significant,
5. `active_selection_anchor` may remain present but is not populated by the first OxFunc scenario wave,
6. defined-name and table inputs may remain empty unless OxFml deliberately exercises those lanes,
7. the typed capability bundle remains the right shape, but only `LocaleFormatContext` is materially required in the first wave,
8. the remaining practical seam questions should be answered by the adapter artifact itself rather than another broad note round.

## First Adapter Request Packet
The first bounded adapter request should be able to express:
1. `formula_text`
2. `formula_channel`
3. `caller_anchor`
4. optional `active_selection_anchor`
5. `cell_fixture`
6. optional `defined_name_bindings`
7. optional `table_catalog`
8. optional `enclosing_table_ref`
9. optional `typed_context_query_bundle`
10. optional `library_context_snapshot_ref`
11. optional `host_query_capability_profile`

Current first-wave narrowing:
1. `formula_channel` should default to the ordinary worksheet cell-formula lane unless a concrete mismatch requires another channel,
2. `active_selection_anchor` should remain optional and currently unevidenced,
3. `defined_name_bindings` and table packet inputs should remain optional and empty-by-default in the first OxFunc-driven wave,
4. `typed_context_query_bundle` should remain the canonical packet, but first-wave exercised dependence is expected to be narrow.

### `cell_fixture`
The minimum fixture world should allow:
1. direct cell values by address,
2. error cells,
3. blank cells,
4. array-like or spill-relevant cells where needed,
5. enough sheet/workbook identity to preserve caller-sensitive and reference-sensitive lanes honestly.

## First Adapter Output Families
The first bounded adapter output should expose three artifact families.

### 1. Preparation artifact
Enough structured output to prove that OxFml prepared the call honestly.

Minimum intended content:
1. formula identity and caller anchor,
2. pinned `library_context_snapshot_ref`,
3. syntax/bind/semantic diagnostics,
4. prepared-call frames or equivalent function-invocation records,
5. prepared arguments with preserved source, structure, reference, and caller facts,
6. callable registration or invocation facts where a higher-order lane is involved.

### 2. End-to-end evaluation artifact
Enough output to prove that OxFunc consumed the prepared inputs correctly through the real seam.

Minimum intended content:
1. final value or error result,
2. `ReturnedValueSurface`,
3. typed host/provider outcome projection where applicable,
4. explicit `execution_outcome_surface` carrying adapter-owned execution classification without asking downstream tools to infer bind-boundary rejection from ad hoc error payloads,
5. selected trace facts needed to explain stage-sensitive or callable-sensitive behavior.

### 3. Mismatch report artifact
Enough structured output to drive bounded seam closure when the adapter reveals divergence.

Minimum intended content:
1. fixture case id,
2. snapshot provenance,
3. expected versus actual seam family,
4. failing packet or field family,
5. narrowed owner guess:
   - OxFml
   - OxFunc
   - shared freeze needed

## First Fixture Families
The first OxFml-side fixture families should be organized by seam pressure, not only by function name.

### 1. Prepared-argument families
1. direct scalar versus array-like lanes,
2. omitted versus blank versus empty string,
3. reference-visible versus dereferenced lanes,
4. caller-context-sensitive scalarization including explicit `@`,
5. structured-reference/table lanes once they are locally honest enough.

### 2. Callable families
1. lexical capture versus shadowing,
2. arity mismatch,
3. helper-bound callable values,
4. adopted defined-name callable lanes,
5. higher-order invoker lanes already exercised locally.

### 3. Host/provider families
1. `INFO`,
2. `CELL`,
3. `RTD`,
4. success versus typed denial/failure projection.

### 4. Snapshot/catalog families
1. pinned `W044` row consumption,
2. alias/name-resolution cases,
3. stage-aware availability/gating cases,
4. runtime provider/snapshot packet cases.

### 5. Deferred families
These should remain explicitly deferred until their owning worksets narrow them enough:
1. worksheet `CALL` / `REGISTER.ID`,
2. richer publication/rich-value packets,
3. broader interoperable callable transport.

## Current Pinned OxFunc Packet Inputs
The first bounded OxFml adapter wave should read the following OxFunc-side artifacts as pinned downstream inputs:
1. `../OxFunc/docs/upstream/OXFUNC_OXFML_SEAM_REQUIREMENTS_CONSOLIDATED.md`
2. `../OxFunc/docs/function-lane/FUNCTION_SLICE_TYPED_CONTEXT_AND_QUERY_BUNDLE_CONTRACT_PRELIM.md`
3. `../OxFunc/docs/function-lane/W47_TYPED_CONTEXT_QUERY_DEPENDENCY_MAP.csv`
4. `../OxFunc/docs/function-lane/FUNCTION_SLICE_RETURN_SURFACE_AND_PUBLICATION_HINT_CONTRACT_PRELIM.md`
5. `../OxFunc/docs/function-lane/W48_RETURN_SURFACE_CLASS_MAP.csv`
6. `../OxFunc/docs/function-lane/FUNCTION_SLICE_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL_PRELIM.md`
7. `../OxFunc/docs/function-lane/W49_RUNTIME_LIBRARY_CONTEXT_CSV_TO_RUNTIME_MAPPING.csv`
8. `../OxFunc/docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`

Working rule:
1. treat these as pinned first-wave downstream artifacts,
2. report only concrete mismatches against them,
3. let the adapter implementation itself answer the remaining practical questions around prepared operands, `@`, and callable invocation where possible.

## Working Rule
The adapter and fixture corpus should be used to close the remaining seam lanes by evidence.

That means:
1. use the current freeze candidates from `W041`, `W042`, and `W043`,
2. report only concrete packet or field mismatches back to OxFunc,
3. do not reopen already-converged broad callable or catalog theory unless the adapter exposes a real contradiction.

## Current Validation Read
OxFunc now explicitly reads the admitted `W049` / `W050` floor as functional validation of the first adapter wave:
1. all 45 of the authoritative 45 first-wave scenarios now pass through the real OxFml adapter and the real OxFunc surface-dispatch path or OxFml-side publication/reject boundary as appropriate,
2. the earlier `C12` and `C14` callable residuals are now handled locally and pinned in the admitted corpus,
3. OxFunc does not request another broad note round for this admitted adapter wave.

Current working implication:
1. keep `W049` / `W050` mismatch-driven from here,
2. move the next new OxFunc-facing note work onto the separate worksheet `CALL` / `REGISTER.ID` lane under `W052`.

## Current Next Owners
The next local owners for this packet are:
1. `W049` OxFunc preparation adapter and consumer harness
2. `W050` OxFunc snapshot-pinned seam fixture families
3. `W052` registered external provider and worksheet `CALL` / `REGISTER.ID` boundary
