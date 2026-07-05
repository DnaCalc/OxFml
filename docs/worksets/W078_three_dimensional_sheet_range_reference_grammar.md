# W078: Three-Dimensional Sheet-Range Reference Grammar

## Purpose

Give OxFml a first-class grammar and bind lowering for 3D sheet-range references of the
form `Sheet1:Sheet3!A1` (and `Sheet1:Sheet3!A1:B2`), so a reference that spans a
contiguous run of sheets is parsed as a single 3D construct rather than being mangled by
the ordinary `:` range loop. This unblocks the W062 R3 3D-slice consumer in OxCalc.

This workset is CREATED, not built. It records the documented grammar collision, a
proposed fix direction, and the OxFunc-boundary semantics. No parser or binder change is
made here.

Tracking bead: `fml-k9s`.

## Position and Dependencies

- **Depends on**: `W037` (R1C1 formula channel and reference translation), `W060`
  (reference detection/validity/resolution boundary), `W077` (BindProfile and reference
  identity seam).
- **Blocks**: OxCalc W062 R3's 3D slice — the R3 3D-slice consumer gates on this grammar
  existing at the OxFml seam.
- **Cross-repo**: semantics touch the OxFunc boundary. Per
  `docs/upstream/NOTES_FOR_OXFUNC.md` (section 7.2, items 4-5 and the classification list),
  a 3D sheet span must remain a construct **distinct** from same-sheet multi-area and must
  not be silently flattened into one reference string. OxFunc owns the resolver-side
  materialization semantics of a 3D span; OxFml owns the grammar and bind lifecycle.

## The documented collision

`Sheet1:Sheet3!A1` does not parse as a 3D reference today. The range loop in
`crates/oxfml_core/src/syntax/parser.rs` (`parse_range` ~line 513 and
`parse_range_from_primary` ~line 543) binds the `:` operator **before** any sheet
qualifier is considered: after parsing the `Sheet1` primary it sees `:` and immediately
forms a `RangeExpr`, whose right operand is then parsed as a fresh primary — which
consumes `Sheet3!A1` as a `QualifiedReferenceExpr` (the Identifier-Bang arm in
`parse_primary` at ~line 806, producing `SyntaxKind::QualifiedReferenceExpr`). The result
is `RangeExpr(Sheet1, Sheet3!A1)` — a range whose left side is a bare sheet name and whose
right side is a qualified cell. That is structurally wrong for a 3D span and cannot be
lowered to a sheet-spanning reference.

## Proposed fix direction (not implemented here)

1. **Parser — 3-token lookahead.** In `parse_primary`'s Identifier-Bang arm (the arm that
   currently turns `Ident !` into `QualifiedReferenceExpr`), add a lookahead for the
   `Colon Identifier Bang` token sequence *before* committing to the ordinary range/qualifier
   path. When an identifier is followed by `: Identifier !`, produce a new
   `SyntaxKind::SheetSpan3DReferenceExpr` node capturing the two sheet identifiers and the
   trailing cell/area target, rather than letting the outer range loop split on the first
   `:`. The lookahead must be bounded (3 tokens, whitespace-skipping) and must not disturb
   ordinary `Sheet1!A1:B2` (single-sheet qualified range) or `A1:B2` (bare range) parsing.
2. **Qualifier model — two-sheet span.** Extend `ParsedQualifier`
   (`crates/oxfml_core/src/binding/mod.rs` ~line 1814) to carry an optional two-sheet
   (start-sheet, end-sheet) span so the bind layer can represent the sheet span alongside
   the existing single-sheet and external-target qualifier fields.
3. **Binder — 3D NormalizedReference arm.** Add a `NormalizedReference`
   (`crates/oxfml_core/src/binding/reference.rs` ~line 176) 3D arm that records
   the sheet span (start sheet, end sheet) plus the resolved cell/area target, keeping the
   span distinct from a same-sheet area and from a multi-area union. Preserve
   caller-independent identity behavior consistent with W077 (the sheet span is part of the
   normal-form key).

## OxFunc boundary

- A 3D sheet span stays a distinct construct from same-sheet multi-area at every layer;
  it must not be flattened into a single reference string (NOTES_FOR_OXFUNC.md 7.2 items
  4-5).
- OxFml preserves the span shape and hands a distinct reference kind to the OxFunc-owned
  resolver; OxFunc owns the actual across-sheet materialization/combination semantics.
- Any new reference kind must be classified alongside the existing single-area / same-sheet
  multi-area / 3D sheet span taxonomy rather than overloading an existing kind.

## Scope

### In scope
1. Grammar and green-tree node for `Sheet1:Sheet3!A1` and `Sheet1:Sheet3!A1:B2`.
2. Qualifier and binder lowering for the sheet-span construct with W077-consistent identity.
3. Keeping the 3D span distinct from same-sheet multi-area across parse, bind, and the
   OxFunc seam.

### Out of scope
1. OxFunc-side across-sheet materialization/combination semantics (OxFunc-owned).
2. OxCalc grid storage of 3D spans and any grid-bounds `#REF!` semantics (consumer-owned).
3. Non-contiguous or reordered sheet spans beyond the contiguous `start:end` form.

## Deliverables
1. A `SheetSpan3DReferenceExpr` grammar production with focused parser fixtures proving
   `Sheet1:Sheet3!A1` no longer mis-binds as `RangeExpr(Sheet1, Sheet3!A1)`.
2. A two-sheet `ParsedQualifier` variant and a 3D `NormalizedReference` arm with bind
   fixtures.
3. Regression fixtures proving single-sheet `Sheet1!A1:B2` and bare `A1:B2` parsing are
   unchanged.

## Gate Model
### Entry gate
- W077 reference identity seam is frozen (met: see W077 workset).
- The collision is documented against exact parser anchors (met: this doc).

### Exit gate
- `Sheet1:Sheet3!A1` parses and binds as a single distinct 3D construct; single-sheet and
  bare-range parsing are byte-for-byte unchanged; the 3D kind is distinct from multi-area
  at the OxFunc seam; OxCalc W062 R3 can consume the 3D slice.

## Implementation (landed)

The fix direction above is now BUILT. Decisions taken where the doc left the grammar
internals open:

1. **Lexer-token vs post-parse question — resolved as parse-time, not a new lexer token.**
   The span is detected in the *parser* (`parse_primary`'s Identifier arm), not the lexer.
   A bounded, whitespace-skipping 3-token lookahead `at_sheet_span_tail()` /
   `peek_significant_kinds(3)` checks for the exact sequence `Colon Identifier Bang`
   immediately after the already-bumped leading identifier. Only on a match does the arm
   consume `: end_sheet !` and emit `SyntaxKind::SheetSpan3DReferenceExpr`. The mandatory
   trailing `!` is the ambiguity guard: `A1:A3`, `name:name`, `Sheet1:Sheet3` (no bang),
   and `Sheet1!A1:B2` (single-sheet qualified range) are byte-for-byte unchanged because
   none carries the `!` in that position.

2. **Grammar production shape.** `SheetSpan3DReferenceExpr` children are
   `[start_sheet, colon, end_sheet, bang, target]`. The target is a single primary (`A1`);
   a trailing `:B2` (`Sheet1:Sheet3!A1:B2`) is left for the outer range loop, yielding
   `RangeExpr(SheetSpan3DReferenceExpr, B2)` — the exact parallel of the qualified-range
   `Sheet1!A1:B2` shape. **Grammar is unconditional (parse always, bind gated)** per the
   W062 D2 stance: the shared grammar always produces the span node; only the binder gates.

3. **Binder / normalized form.** `NormalizedReference::SheetSpan3D(SheetSpan3DRef)` carries
   `{workbook_id, start_sheet, end_sheet, target}`; both sheet ids plus the target are part
   of the normal-form key (`Display` → `sheet-span-3d:<wb>:<start>:<end>!<target>`), kept
   **distinct** from `Name`, same-sheet multi-area, and range — never lowered to a range or
   an area union (NOTES_FOR_OXFUNC.md 7.2 items 4-5). Evaluation defers to a `#REF!` value
   (across-sheet materialization is OxCalc-owned, W062 R3.9/R4.12).

4. **Flag default decision (RECORDED).** A new `ReferenceSyntaxCapabilities`
   `sheet_span_3d_references` flag gates binder routing, following the R3.8 pattern
   (commit 1ef6d19). Defaults: `all()` = **true** (the no-gating trait default —
   `syntax_capabilities()` admits every parsed shape, so the no-profile path binds the
   span, default-preserving); `worksheet_legacy()` = **false** (3D is opt-in per W062 D2 —
   only a profile that explicitly supports across-sheet spans, the strict-Excel style,
   turns it on). Flag OFF → typed `#REF!` capability rejection with a bind diagnostic
   containing "3D sheet-span", never a silently-wrong range.

### Typed limitations (in this landing)
- **External-target start sheet** (`[wb]Sheet1:Sheet3!A1`) is NOT recognized as a 3D span:
  its leading token lexes as a single `BracketedQualifier`, which the span gate (Identifier
  / QuotedIdentifier start only) excludes, so it retains the pre-existing range shape. The
  in-scope form is the unquoted-or-quoted contiguous `start:end` per Scope item 3
  ("Non-contiguous or reordered sheet spans" are explicitly out of scope). A quoted *start*
  sheet (`'My Sheet':Sheet3!A1`) IS handled. A quoted *end* sheet (`Sheet1:'Q2 Data'!A1`)
  is NOT: the lookahead restricts the end sheet to a plain `Identifier` on purpose, because
  admitting `QuotedIdentifier` there collides with the tested whole-column qualified-range
  form `'Sheet'!A:'Sheet'!C` (the inner target `A` would eat the `:'Sheet'!` tail as a
  bogus span). The quoted-end form fails safe to the pre-W078 range shape with no
  diagnostic; broadening it needs a target-aware disambiguation and is left as a follow-up.
- A degenerate `Sheet1:Sheet3!A1:Sheet5!B2` (a second `!`-bearing span in the target)
  recursively nests span nodes; this is malformed input outside the contiguous
  `start:end!target` grammar and is left as-is (no valid input reaches it).

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated (oxfml_core suite green)
- open_lanes: none (OxCalc W062 R3.9 across-sheet materialization is consumer-owned)
- claim_confidence: verified (grammar, bind gating, distinctness, round-trip, and full
  oxfml_core suite proven green; fresh-eyes reviewed)
