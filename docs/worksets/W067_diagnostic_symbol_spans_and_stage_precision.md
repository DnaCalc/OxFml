# W067: Diagnostic Symbol Spans And Stage Precision

## Purpose
Narrow OxFml editor/runtime diagnostics so host callers can render Roslyn-like, squiggle-ready diagnostics for unresolved symbols without flattening useful formula-stage distinctions.

The motivating example is:

```text
=YYYY(1,2)+ABS(-12)+QQQQ
```

Current local behavior preserves correct worksheet outcome (`#NAME?`) and correctly distinguishes an unknown function surface from an unresolved bare identifier, but the live diagnostic packet does not yet expose enough precise symbol-span and stable diagnostic-code information for callers to squiggle both `YYYY` and `QQQQ` consistently.

## Position and Dependencies
- **Depends on**: `W032`, `W038`, `W048`, and planned `W060` review inputs
- **Blocks**: stronger DNA OneCalc editor squiggle fidelity, future editor replay evidence for diagnostics, and any shared immutable-edit diagnostic packet freeze that needs stable diagnostic identities
- **Cross-repo**:
  - DNA OneCalc consumes the editor diagnostic packet and should not infer missing spans locally
  - OxFunc remains owner of function catalog truth and function metadata availability
  - OxCalc may later consume the same diagnostic packet in immutable formula-edit flows, but no coordinator-facing handoff is required unless packet shape becomes shared seam text

## Scope
### In scope
1. Review current syntax, bind, semantic-plan, and runtime diagnostic surfaces for unresolved symbol and callability cases.
2. Define a span-bearing diagnostic model that preserves:
   - diagnostic stage (`Syntax`, `Bind`, `SemanticPlan`, `Runtime` where applicable),
   - stable diagnostic code,
   - primary source span,
   - optional related spans,
   - worksheet-visible error class when known (for example `#NAME?`).
3. Add span carriage for semantic-plan diagnostics, especially unknown function calls.
4. Preserve call shape for unknown functions instead of reclassifying them as parse errors.
5. Keep unresolved bare identifiers as bind-stage facts while making their packet shape align with unknown-function diagnostics for caller rendering.
6. Add deterministic tests for a first matrix of unknown functions, unresolved identifiers, known functions, arity mismatches, known-noncallable symbols used as calls, gated/unavailable functions, structured-reference unresolved cases, and deferred reference-validity cases where current infrastructure admits them.
7. Identify which cases must remain deferred to `W038` host-managed name/external-name or `W060` reference-validity work instead of being patched locally.

### Out of scope
1. Changing final worksheet error semantics for unknown functions or unknown names.
2. Treating unknown functions as parse failures.
3. Full OxFunc catalog expansion beyond the function metadata needed to classify diagnostics.
4. Product UI rendering in DNA OneCalc.
5. Full reference-validity ownership correction before `W060` completes its boundary review.

## Deliverables
1. Updated spec text for editor/live diagnostics describing stage, code, span, related-span, and worksheet-error-class expectations.
2. Implementation changes to carry precise primary spans on semantic-plan diagnostics and live diagnostic projection.
3. A stable first diagnostic code taxonomy covering at least:
   - `unknown_function`,
   - `unknown_name`,
   - `known_symbol_not_callable`,
   - `function_arity_mismatch`,
   - `function_gated_or_unavailable`,
   - `structured_reference_unresolved`,
   - `reference_invalid_or_deferred` where currently representable.
4. Deterministic local tests proving `=YYYY(1,2)+ABS(-12)+QQQQ` surfaces squiggle-ready diagnostics for `YYYY` and `QQQQ` while leaving `ABS` clean.
5. A defer/ownership matrix for cases that belong to `W038`, `W060`, or OxFunc catalog work rather than the local editor diagnostic pass.
6. Bead-level implementation plan tied to this workset.

## Candidate Diagnostic Matrix
1. Unknown function call: `=YYYY(1)`.
2. Unknown function plus known function plus unresolved identifier: `=YYYY(1,2)+ABS(-12)+QQQQ`.
3. Unresolved bare identifier: `=QQQQ`.
4. Known function with invalid arity: representative existing catalog-known arity failure.
5. Known value-like name invoked as function: `=SomeValueName(1)` with host-provided value-like name.
6. Helper/local or defined-name collision cases that currently have exercised boundary tests.
7. Known but unavailable/gated function through library-context snapshot state where current local fixtures can express it.
8. Structured-reference unresolved cases already admitted by `W036` / current table-context tests.
9. Reference-validity cases explicitly deferred to `W060` if current ownership would force local invention of workbook truth.

## Implementation Evidence And Deferrals
Current local evidence under `crates/oxfml_core/tests/language_service_tests.rs` exercises:
1. exact semantic-plan callee span for `YYYY` and exact bind-stage name span for `QQQQ` in `=YYYY(1,2)+ABS(-12)+QQQQ`, while `ABS` remains clean,
2. exact bind-stage callee span and `function_arity_mismatch` code for a catalog-known invalid-arity function call,
3. exact bind-stage callee span and `known_symbol_not_callable` code for a reference-like host name invoked as a call,
4. exact semantic-plan callee span and `function_gated_or_unavailable` code for a library-context gated function surface,
5. structured-reference unresolved classification for the existing missing enclosing-table context case.

Deferrals remain intentional:
1. value-like defined names that may carry host-managed callable payloads are not broadly rejected locally until `W038` narrows name carrier callability truth,
2. reference grid validity and host/workbook-profile truth remain deferred to `W060`,
3. product UI squiggle rendering remains DNA OneCalc-owned after consuming the packet.

## Design Notes
1. This workset follows a Roslyn-like model: syntax succeeds where the token stream has valid call/name shape, binding/semantic diagnostics attach to unresolved symbols, and error-shaped symbols allow later stages to continue enough to avoid cascading loss.
2. Unknown function calls and unresolved identifiers should not be made identical internally, but callers should receive enough common diagnostic shape to render both precisely.
3. Span data should be carried from parse/bind artifacts rather than rediscovered from formula text in late projection wherever practical.
4. The final worksheet-visible result may remain `#NAME?` while multiple diagnostics identify separate source causes.

## Gate Model
### Entry gate
- Current editor language-service packet exists and can carry live diagnostics.
- Current parser/binder exposes source spans for syntax and bind diagnostics.
- Current semantic-plan diagnostics can identify function names but need span/code enrichment.

### Exit gate
- Exact-span live diagnostics exist for the motivating formula.
- Unknown function, unresolved bare identifier, known arity mismatch, and known-noncallable invocation have deterministic local evidence or explicit deferral records.
- Diagnostic stage distinctions remain visible in the packet.
- DNA OneCalc can render squiggles without host-side symbol inference for the exercised diagnostic families.

## Pre-Closure Verification Checklist

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text updated for all in-scope items? | yes |
| 2 | Conformance matrix rows updated? | n/a |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes — local deterministic editor tests |
| 4 | Cross-repo impact assessed and handoff filed if needed? | yes — DNA OneCalc note updated; no shared coordinator-facing packet change filed |
| 5 | All required tests pass? | yes — `cargo test -p oxfml_core` |
| 6 | No known semantic gaps remain in declared scope? | yes, with W038/W060 deferrals recorded |
| 7 | Completion language audit passed (no premature completion wording per AGENTS.md Section 3)? | yes |
| 8 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 9 | CURRENT_BLOCKERS.md updated (new/resolved)? | n/a |

## Status
- execution_state: exercised_local
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: partial
- open_lanes:
  - DNA OneCalc still needs to consume the updated OxFml packet and add/re-enable exact-span browser assertions
  - W038 still owns broader host-managed value-like/callable name truth
  - W060 still owns broader reference-validity truth
- claim_confidence: evidence_backed_local
