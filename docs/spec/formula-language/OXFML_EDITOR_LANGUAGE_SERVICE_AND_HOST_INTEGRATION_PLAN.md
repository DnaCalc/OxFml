# OxFml Editor Language Service And Host Integration Plan

## 1. Purpose
Define the extended-scope plan for turning OxFml's formula parser and binder into an editor-grade language substrate suitable for:
1. larger immutable host document trees,
2. live diagnostics and squiggle surfaces,
3. function help and signature help,
4. completion and intelligent-completion integration,
5. edit-driven incremental syntax/bind updates without hidden semantic mutation.

This is primarily a planning document for future scope, but it now also records the first OxFml-local execution slice for editor-facing packets.
It is not a claim that the current OxFml local floor already provides a full editor-grade service surface.

Read together with:
1. `OXFML_FORMULA_ENGINE_ARCHITECTURE.md`
2. `OXFML_PARSER_AND_BINDER_REALIZATION.md`
3. `OXFML_OXFUNC_LIBRARY_CONTEXT_RUNTIME_INTERFACE.md`
4. `../OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md`
5. `../OXFML_PUBLIC_API_AND_RUNTIME_SERVICE_SKETCH.md`

## 2. Current Read
Current OxFml already has the right broad architectural direction:
1. immutable green tree,
2. contextual red view,
3. explicit bind artifacts,
4. explicit runtime library-context provider direction.

But the current local floor is still narrower than a true editor-grade language-service substrate.

Main current gaps:
1. trivia is preserved only as plain tokens, not as carefully owned leading/trailing trivia on syntax elements,
2. no canonical edit/change packet exists for formula-in-document spine updates,
3. diagnostics are parser/bind/runtime artifacts, not yet a unified live language-service stream,
4. function help/signature help are not surfaced as live editor packets,
5. no canonical completion or intelligent-completion request/response packet exists.

## 2.1 Current first local floor
OxFml now has a first internal language-service packet layer in `crates/oxfml_core/src/language_service/mod.rs`.

That local floor currently includes:
1. canonical syntax-tree tokens with leading/trailing trivia owned directly in the green tree, plus `EditorSyntaxSnapshot` built from that owned trivia while the retained full token stream stays available for correlation and round-trip text recovery,
2. `FormulaEditRequest` / `FormulaEditResult` plus explicit text-change ranges and subtree-reuse summaries over incremental parse/red/bind and optional semantic-plan follow-on,
3. `LiveDiagnosticSnapshot` unifying syntax, bind, and semantic-plan diagnostics for squiggle/list use,
4. deterministic completion packets over visible functions, names, tables, table columns, structured selectors, and first `R1C1` syntax assists,
5. cursor-sensitive `SignatureHelpContext`,
6. deterministic function-help subject construction so OxFml can publish a canonical help packet without guessing at call context,
7. `IntelligentCompletionContext` so an external intelligent completer can work from one normalized context packet,
8. deterministic completion-candidate validation and proposal application that re-enter the normal parse/bind/plan pipeline rather than bypassing it.

This is still narrower than the full target outcome:
1. no OxFunc-backed help payload retrieval exists yet,
2. no shared host/OxCalc immutable formula-edit packet is frozen yet,
3. no shared host-facing packet for validated intelligent-completion results is frozen yet,
4. editor packet evidence is deterministic local evidence, not replay-appliance projection.

## 3. Target Outcome
The target outcome is not "an editor inside OxFml".
The target outcome is a clean immutable language-service substrate that external hosts can use.

That substrate should allow:
1. OxCalc-integrated hosts and direct hosts to embed formula green trees as canonical immutable children in larger immutable workbook/document trees,
2. formula edits to rebuild only the edited leaf payloads plus ancestor spine,
3. live parse/bind/semantic diagnostics to be surfaced continuously,
4. OxFunc-backed function help to appear in the editor stream,
5. deterministic local completion plus optional external intelligent completion to operate over the same immutable context packet.

## 4. Green Tree And Trivia Plan
### 4.1 Required extension
Green trees should move from "token-retaining" to "carefully trivia-owning".

Required direction:
1. every syntax token should preserve exact source text,
2. trivia should be preserved explicitly and stably enough for round-trip and formatting-neutral editing,
3. malformed fragments should remain representable in-tree rather than only in diagnostics,
4. incremental edits should be able to reuse unchanged green subtrees without reparsing the whole formula.

### 4.2 Intended token/trivia model
The preferred editor-grade model is:
1. `GreenToken`
   - `kind`
   - `text`
   - `leading_trivia`
   - `trailing_trivia`
   - `span` or span-derivable width
2. `GreenTrivia`
   - `kind`
   - `text`
3. `GreenNode`
   - immutable
   - parentless
   - context-free
   - children = nodes or tokens

The exact storage layout remains open.
The semantic requirement is not.

### 4.3 Formula-text update rule
Canonical update direction should be host-driven:
1. host owns the larger immutable workbook/document tree,
2. host submits a formula-text edit request against one formula-bearing slot,
3. OxFml returns:
   - new green root,
   - subtree-reuse metadata,
   - updated diagnostics,
   - optional updated bind and semantic-plan artifacts when requested,
4. host then rebuilds only the containing immutable document spine.

Working rule:
1. OxFml should not own the whole workbook tree,
2. OxFml should own the immutable formula artifact transforms,
3. larger document-spine replacement remains host/coordinator work.

### 4.4 Change packet
The first future edit packet should include:
1. `formula_stable_id`
2. `previous_formula_token`
3. `previous_green_tree_key`
4. `new_formula_text`
5. optional textual change ranges
6. `structure_context_version`
7. requested follow-on stages:
   - parse only
   - parse + bind
   - parse + bind + semantic-plan

Expected return packet:
1. `new_formula_token`
2. `green_tree_key`
3. subtree reuse summary
4. diagnostics stream snapshot
5. optional `bind_hash`
6. optional `semantic_plan_key`

## 5. Live Diagnostics Plan
### 5.1 Unified language-service diagnostics
OxFml should expose one live diagnostics family with typed origin/stage rather than separate host-specific ad hoc lists.

Minimum diagnostics classes:
1. `syntax_error`
2. `syntax_recovery_info`
3. `bind_error`
4. `bind_warning`
5. `semantic_plan_warning`
6. `capability_info`
7. `host-service-unavailable_info`

### 5.2 Display model
The language-service packet should support:
1. list views,
2. squiggle spans,
3. hover detail,
4. quick navigation to the span,
5. change-stable diagnostic identity where possible.

Minimum fields:
1. `diagnostic_id`
2. `severity`
3. `stage`
4. `message`
5. `primary_span`
6. optional `related_spans`
7. optional `code`
8. optional `suggested_fix_kind`

### 5.3 Suggestions and fix-its
OxFml may eventually provide bounded structured suggestions where semantics are local and deterministic.

Examples:
1. missing closing delimiter,
2. malformed structured-reference qualifier combination,
3. omitted-table-name without enclosing table context,
4. unsupported function/query family in current host profile.

Working rule:
1. suggestions are advisory,
2. they must never silently mutate canonical formula text,
3. they must stay deterministic and replay-stable.

## 6. Function Help And Signature Help Plan
### 6.1 Source of truth
Function help should come from OxFunc, not from duplicated prose inside OxFml.

Preferred source:
1. OxFunc function catalog/runtime snapshot metadata,
2. optionally paired help/signature documentation packet from OxFunc,
3. versioned by the same library-context snapshot identity already used for semantic planning.

### 6.2 Why OxFunc is the right source
OxFunc already owns:
1. function identity,
2. semantic traits,
3. arity and argument-shape truth,
4. deferred-function classification,
5. profile/gating truth for built-ins and registered extensions.

So the editor/help layer should not invent a second function-definition source of truth in OxFml.

### 6.3 First function-help packet
The first future packet should support:
1. `lookup_key`
   - typed function id or surface token
2. `library_context_snapshot_ref`
3. `display_name`
4. `signature_forms`
5. `argument_help`
6. `short_description`
7. `availability/gating_summary`
8. `deferred_or_profile_limited` flags where applicable

### 6.4 Signature-help trigger model
Signature help should be driven by:
1. current cursor position,
2. current bound/red syntax position,
3. current active argument index,
4. current library-context snapshot.

OxFml should compute:
1. whether the cursor is inside a call,
2. active callee syntax,
3. active argument ordinal,
4. parse/bind ambiguity notes if the call is currently malformed.

OxFunc should supply:
1. the function signatures and argument help payloads for the identified function.

## 7. Completion And Intelligent Completion Plan
### 7.1 Deterministic local completion
OxFml should first expose deterministic local completion categories:
1. function names from the current library-context snapshot,
2. defined names visible in bind context,
3. table names and column names where structured-reference context is active,
4. syntax keywords/selector families such as `#Headers`, `#Data`, `#Totals`, `#All`, `#This Row`,
5. channel-specific syntax assists such as `R1C1` forms where applicable.

### 7.2 Intelligent completion boundary
External intelligent completion is allowed, but it must remain non-canonical and host-owned.

Working rule:
1. OxFml provides the structured context packet,
2. an external intelligent completer may propose candidate edits or insertions,
3. OxFml remains the canonical validator through parse/bind/semantic diagnostics,
4. no intelligent suggestion becomes semantic truth until it re-enters OxFml through the ordinary edit path.

### 7.3 First intelligent-completion context packet
The minimum packet should include:
1. `formula_text`
2. `formula_channel_kind`
3. `cursor_span_or_offset`
4. `green_tree_key`
5. `red_context_summary`
6. visible name/table scope summaries
7. `library_context_snapshot_ref`
8. active diagnostics near cursor
9. active call/signature-help context if present

Optional richer fields later:
1. nearby formula snippets,
2. surrounding host object kind:
   - cell
   - defined name
   - external name
   - conditional-formatting rule
   - data-validation rule
3. target profile/capability summary

### 7.4 Completion result packet
Deterministic and intelligent completion results should normalize to one insertion-oriented shape:
1. `proposal_id`
2. `proposal_kind`
3. `display_text`
4. `insert_text`
5. optional `replacement_span`
6. optional `documentation_ref`
7. optional `requires_revalidation` flag

## 8. Host And OxCalc Integration
### 8.1 Direct host
A direct single-formula host should be able to use the same language-service packet family without OxCalc.

That means:
1. formula edit packets are independent of coordinator scheduling,
2. diagnostics/help/completion are derived from immutable formula artifacts plus explicit context,
3. live editing does not require a multi-node engine.

### 8.2 OxCalc-integrated host
In OxCalc-integrated mode:
1. the larger immutable workbook/document tree stays host/coordinator-owned,
2. OxFml remains the canonical formula-language service for formula-bearing nodes,
3. OxCalc may add cross-cell orchestration, but should not redefine formula syntax or local editor semantics.

### 8.3 Name/table/object ownership
This plan does not change the existing ownership split:
1. host/coordinator owns workbook objects,
2. OxFml owns formula-language meaning,
3. OxFunc owns function help/semantic catalog truth for functions.

### 8.4 OxFml best-effort proposal for OxCalc/direct host
Current OxFml best-effort proposal is that the first shared editor packet should be split into:
1. immutable edit request,
2. immutable edit result,
3. validated completion application result.

Proposed immutable edit request:
1. `formula_stable_id`
2. `previous_formula_token`
3. `previous_green_tree_key`
4. `new_formula_text`
5. optional `text_change_range`
6. `formula_channel_kind`
7. `structure_context_version`
8. explicit bind-visible context summary:
   - visible names
   - visible tables
   - caller anchor when already part of the formula slot
9. requested follow-on stage

Proposed immutable edit result:
1. `new_formula_token`
2. `green_tree_key`
3. `text_change_range`
4. subtree reuse summary
5. diagnostics snapshot
6. optional `bind_hash`
7. optional `semantic_plan_key`

Proposed validated-completion application result:
1. `proposal_id`
2. applied replacement span
3. updated immutable edit result
4. explicit rule that host/coordinator still owns the containing document-spine replacement

Working rule:
1. OxFml should not mutate the workbook/document tree,
2. OxFml should only return replacement-ready immutable formula artifacts,
3. host or coordinator remains responsible for accepting the result and rebuilding the containing immutable spine.

## 9. OxFunc Seam Implications
This extended scope creates one likely future seam packet with OxFunc:
1. editor/help-facing function-definition packet or provider surface.

Expected future questions for OxFunc:
1. what is the smallest help/signature packet derivable from runtime library-context truth,
2. which fields are semantic truth versus prose/help presentation,
3. how registered runtime extensions participate,
4. whether function-help retrieval rides the existing runtime library-context provider or a sibling metadata provider.

Working recommendation:
1. keep semantic-planning truth and help/signature truth related by shared stable ids,
2. avoid making OxFml scrape or duplicate OxFunc docs,
3. keep editor/help metadata versioned by library-context snapshot identity where practical.

### 9.1 OxFml best-effort proposal for OxFunc
Current OxFml best-effort proposal is:
1. keep semantic planning on the existing runtime `LibraryContextSnapshot`,
2. expose help/signature metadata through a sibling help provider keyed by the same snapshot identity rather than overloading the hot-path semantic snapshot,
3. let OxFml compute call-site context locally and ask OxFunc only for help payloads.

Proposed request:
1. `lookup_key`
2. `library_context_snapshot_ref`

Proposed response:
1. `stable_function_id`
2. `display_name`
3. `signature_forms`
   - parameter display labels
   - minimum arity
   - maximum arity or open-ended marker
4. `short_description`
5. `availability_summary`
6. `deferred_or_profile_limited`
7. optional `documentation_ref`

Working rule:
1. OxFunc remains the source of truth for signatures/help text,
2. OxFml remains the source of truth for cursor position, active argument index, parse ambiguity, and whether the user is inside a call at all.

## 10. First Work Breakdown
The likely execution order for this editor-grade extension is:
1. green-tree trivia and token ownership freeze,
2. immutable formula-edit and subtree-reuse packet,
3. live diagnostics packet and stage taxonomy,
4. deterministic completion packet,
5. OxFunc-backed function-help/signature-help seam,
6. external intelligent-completion context packet,
7. host/OxCalc integration packet.

## 10A. Open-Lane Closure Strategy
The remaining `W048` lanes should be handled in this order:
1. OxFml-only execution:
   - trivia-owning green-token realization,
   - deterministic completion breadth,
   - editor replay/evidence widening,
2. OxFunc seam freeze:
   - help/signature provider shape,
   - minimal help/signature payload,
3. OxCalc seam freeze:
   - immutable edit request/result packet,
   - validated intelligent-completion result packet.

This matters because two visible open lanes are now mainly packet-shape freeze rather than formula-semantics uncertainty:
1. OxFunc help/signature payloads,
2. OxCalc immutable-edit and validated-completion integration.

## 11. Non-Goals
Out of scope for the first extension wave:
1. a full IDE/workspace implementation inside OxFml,
2. auto-fix mutation applied without host approval,
3. AI/LLM completion becoming a semantic authority,
4. storing the whole workbook object model inside OxFml,
5. conflating editor services with FEC/F3E runtime session semantics.

## 12. Integration Readiness Classification For Downstream Hosts

### 12.1 Integration-Ready Packet Surfaces
The following language-service packet surfaces are currently good enough for downstream host integration. Hosts such as DNA OneCalc should consume these rather than inventing local equivalents:

1. `FormulaEditRequest` / `FormulaEditResult` — immutable edit request/result with text-change ranges, incremental parse/red/bind reuse, optional semantic-plan follow-on.
2. `LiveDiagnosticSnapshot` — unified syntax/bind/semantic-plan diagnostics for squiggle and list use.
3. Deterministic completion proposals — over visible functions, names, tables, table columns, structured selectors, and R1C1 syntax assists.
4. Completion-candidate validation and proposal application — re-enters the normal parse/bind pipeline.
5. `SignatureHelpContext` — cursor-sensitive call and argument context.
6. function-help packet publication — deterministic subject resolution keyed to the current library-context snapshot.
7. `IntelligentCompletionContext` — normalized context packet for external non-canonical completion.
8. `EditorSyntaxSnapshot` — owned-trivia token view for editor rendering.

### 12.2 Local-Only Evidence Surfaces (Not Yet Integration-Ready)
The following exist as deterministic local evidence but are not yet frozen for host integration:

1. OxFunc-backed help/signature payload retrieval — depends on OxFunc help-metadata freeze.
2. Shared host/OxCalc immutable formula-edit packet — depends on OxCalc immutable-edit seam round.
3. Shared host-facing validated intelligent-completion result packet — depends on OxCalc validated-completion seam round.
4. Editor packet replay-appliance projection — depends on replay adapter promotion.

### 12.3 Downstream Integration Working Rule
1. downstream hosts should consume the integration-ready surfaces in Section 12.1 as their canonical formula-edit, diagnostic, completion, and help-lookup substrate,
2. for OxFunc-backed help payloads, downstream hosts should start from the library-context snapshot export metadata fields while keeping the host ready for the later provider-backed snapshot model,
3. intelligent completion remains host-owned and non-canonical until re-validated through OxFml's ordinary edit path,
4. hosts may add presentation, interaction, and command affordances but must not locally own canonical parse, bind, diagnostic, completion validity, or function/signature help payload truth.

For the full downstream clarification including field-level obligations and not-authorized surfaces, see `../OXFML_DNA_ONECALC_DOWNSTREAM_CONSUMER_CONTRACT.md`.

## 13. Current Recommendation
The next honest planning owner should:
1. freeze the editor-grade green-tree/trivia model first,
2. keep the update path host-driven and immutable-spine-friendly,
3. treat diagnostics/help/completion as explicit typed packets,
4. source function help from OxFunc through stable runtime/catalog truth,
5. keep intelligent completion external and non-canonical.
