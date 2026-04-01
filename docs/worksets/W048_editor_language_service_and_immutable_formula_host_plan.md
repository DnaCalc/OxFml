# W048: Editor Language Service And Immutable Formula Host Plan

## Purpose
Plan the extended-scope formula-editor and language-service surface so OxFml can later serve as:
1. the canonical immutable formula subtree owner inside larger host document trees,
2. the source of live diagnostics and squiggle-ready spans,
3. the source of deterministic completion context,
4. the OxFunc-linked bridge for function help and signature help,
5. the validator for external intelligent completion proposals.

## Position and Dependencies
- **Depends on**: `W032`, `W041`, `W043`, `W045`
- **Blocks**: future implementation work for editor-grade parse/bind services, host editor integration, and OxFunc help-metadata seam narrowing
- **Cross-repo**: future OxFunc seam packet for function-help/signature metadata; future OxCalc/host packet for immutable formula-edit integration

## Scope
### In scope
1. Freeze the intended editor-grade green-tree and trivia model at planning level.
2. Freeze the intended immutable formula-edit packet and host-driven spine update rule at planning level.
3. Freeze the intended live diagnostics packet and severity/stage taxonomy at planning level.
4. Freeze the intended deterministic completion, function-help, signature-help, and external intelligent-completion packet boundaries at planning level.
5. Record the expected OxFunc and OxCalc seam consequences for later bounded rounds.

### Out of scope
1. Implementing the editor-grade substrate.
2. Delivering a real editor UI.
3. Delivering an LLM or external intelligent-completion service.
4. Promoting any new seam packet to shared frozen text.

## Deliverables
1. Canonical planning spec for the extended editor/language-service scope.
2. Explicit work breakdown for future execution slices.
3. Explicit OxFunc- and OxCalc-facing future seam questions.
4. Explicit rule that intelligent completion remains external and non-canonical.

## Closure Plan
The remaining `W048` lanes are no longer one undifferentiated backlog.
They now split cleanly into OxFml-internal execution work versus seam-freeze work.

### A. OxFml-internal execution work
These slices can continue locally without waiting for another repo:
1. trivia-owning green-token realization
   - extend the canonical green token/storage model so trivia is owned directly rather than projected later,
   - keep the current editor snapshot builder as a compatibility projection during transition,
   - add deterministic incremental-edit evidence proving unchanged subtrees survive trivia-preserving edits,
2. deterministic completion breadth
   - widen local completion beyond the current function/name/table/selector slice,
   - cover channel-sensitive assists such as more `R1C1` entry help and restricted-carrier assists,
   - keep all completion proposals replay-stable and deterministic,
3. editor replay evidence
   - add replay-facing or retained local witness artifacts for edit-result packets, diagnostics, and validated completion re-entry,
   - prove that editor packet identity is stable enough for later host/editor integration,
4. local packet hardening
   - keep refining `FormulaEditResult`, `LiveDiagnosticSnapshot`, and completion-validation artifacts until the host-facing seam packet is mostly a projection rather than a reinvention.

### B. Seam-freeze-only work
These lanes are now mostly a cross-repo packet-shape decision rather than an OxFml semantic unknown.

#### B1. OxFunc seam: function help and signature help
What remains here is mainly packet freeze, not local semantic discovery.
OxFml can already:
1. detect the active call site,
2. compute active argument index,
3. publish a deterministic `FunctionHelpPacket` built from pinned call-context resolution.

What now needs freezing with OxFunc:
1. whether help retrieval rides the existing runtime `LibraryContextProvider` or a sibling metadata/help provider,
2. the minimum help/signature response packet,
3. which fields are semantic truth versus presentation-only prose,
4. how runtime-registered extension functions participate under snapshot identity.

#### B2. OxCalc seam: immutable edit and validated intelligent-completion packets
What remains here is also mainly packet freeze, not formula semantics.
OxFml can already:
1. accept immutable formula-edit requests,
2. return new artifact identities, reuse summaries, diagnostics, and change ranges,
3. revalidate intelligent-completion proposals through the normal parse/bind path.

What now needs freezing with OxCalc:
1. the exact host/coordinator-facing edit packet,
2. the exact return packet for editor updates,
3. whether validated completion application is a host-local packet or a coordinator-visible packet,
4. how larger immutable workbook/document spine replacement is keyed and acknowledged outside OxFml.

## Next Execution Order
The recommended next order is:
1. finish OxFml-local trivia-owning green-token design and first exercised slice,
2. widen deterministic completion and editor replay evidence locally,
3. run a bounded `NOTES_FOR_OXFUNC` round on help/signature packet freezing,
4. run a bounded `NOTES_FOR_OXCALC` round on immutable edit and validated-completion packet freezing,
5. only after those packets converge, promote the editor host packet from local OxFml layer to shared seam text.

## Gate Model
### Entry gate
- Current parser/green/red architecture is strong enough to support a narrower extension plan.
- Host/runtime packet direction is converged enough to describe host-driven immutable updates honestly.

### Exit gate
- There is one canonical planning document for the editor-grade extension.
- The immutable update path, diagnostics/help/completion packet families, and seam implications are explicit rather than implied.
- Future execution order is explicit.

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
  - function help and signature help now reach deterministic facade packet publication; OxFunc-backed richer help payload retrieval is not yet integrated
  - OxCalc now reads the immutable edit request / result / validated completion split as the right first packetization, but no shared host/OxCalc immutable edit packet is frozen yet
  - containing-spine replacement and validated-completion acceptance are now converged as host/coordinator-owned, but no shared host-facing packet for validated intelligent-completion results is frozen yet
- current_local_floor:
  - `crates/oxfml_core/src/language_service/mod.rs` now provides a first OxFml-local language-service packet layer
  - live internal packet types now exist for editor syntax snapshots, formula-edit requests/results, explicit text-change ranges, live diagnostics, deterministic completion, completion-validation, signature-help context, function-help lookup requests, and intelligent-completion context
  - `apply_formula_edit(...)` now drives incremental parse/red/bind reuse plus optional semantic-plan follow-on for editor-host flows and reports the smallest local text-change range when a previous green tree is supplied
  - `build_live_diagnostics(...)` now unifies syntax, bind, and semantic-plan diagnostics into one squiggle/list-ready packet family
  - syntax-tree tokens now own canonical leading/trailing trivia directly in the green tree while the raw lexer stream remains available in `full_fidelity_tokens`
  - `collect_completion_proposals(...)`, `signature_help_context_at_cursor(...)`, `build_function_help_lookup_request(...)`, `validate_completion_candidate(...)`, `apply_completion_proposal(...)`, and `build_intelligent_completion_context(...)` now provide the first local deterministic editor-support floor, including `R1C1` syntax assists
  - deterministic local evidence now exists in `crates/oxfml_core/tests/language_service_tests.rs` and `crates/oxfml_core/tests/language_service_fixture_tests.rs`
- claim_confidence: draft
