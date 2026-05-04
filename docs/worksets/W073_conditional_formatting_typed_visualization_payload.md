# W073: Conditional Formatting Typed Visualization Payload

## Purpose

Replace the W072 bounded string conventions for aggregate-context conditional-formatting visualization rules with a typed payload model that can express Excel color-scale stops, data-bar options, icon-set thresholds, rank options, and average-rule flags without overloading `VerificationConditionalFormattingRule.thresholds`.

W072 remains the validated common-case floor. W073 is the follow-up that turns that floor into an explicit typed contract for richer authoring and consumer configuration.

## Position and Dependencies

- **Depends on**: `W071`, `W072`
- **Unblocks**: richer DNA OneCalc conditional-formatting rule configuration without encoding rule metadata as threshold strings.
- **Cross-repo**: DNA OneCalc has acknowledged W072 and can already render the bounded carrier. W073 needs DNA OneCalc alignment before any bridge-breaking payload shape is promoted.

## Source Context

1. DNA OneCalc `docs/HANDOFF_OXFML_CF_AGGREGATE_VISUALIZATION_RULES.md` records W072 as closed and absorbed on 2026-05-04.
2. W072 documented the current bounded conventions:
   - color-scale stops in `thresholds`, e.g. `min:#F8696B`, `percent:50:#FFEB84`, `num:42:#63BE7B`;
   - data-bar options in `thresholds`, e.g. `min:n`, `max:n`, `direction:right`, `showBarOnly`;
   - icon-set kind and thresholds in `thresholds`;
   - rank count/percent and average flags in `thresholds`.
3. W072 already supports several items that the original DNA OneCalc closure note listed as open at the time of writing: explicit data-bar min/max, explicit icon-set numeric thresholds, and average `equal` / `stddev:n`.

## Scope

### In Scope

1. Define a typed conditional-formatting rule metadata model for the W072 aggregate and visualization families.
2. Preserve W072 evaluator behavior while moving rule metadata reads from string parsing to structured fields.
3. Keep JSON/publication output for `ArrayCellFormat.data_bar` and `ArrayCellFormat.icon` stable unless a separate consumer-facing change is explicitly agreed.
4. Add deterministic tests proving typed payloads produce the same outcomes as the W072 bounded convention.
5. File downstream-facing notes if the bridge shape changes.

### Out of Scope

1. DNA OneCalc UI controls for richer per-rule configuration.
2. Pixel-perfect Excel rendering of data bars, icons, and gradients.
3. Worksheet-range conditional-formatting semantics outside formula-result publication.
4. Full Excel formula-based conditional-formatting stop support unless separately scoped.

## Compatibility Decision

W073 first slice lands as an additive typed payload. `VerificationConditionalFormattingRule.typed_rule` is preferred by the evaluator when present; existing W072 bounded string conventions in `thresholds` remain supported as compatibility fallback.

No existing DNA OneCalc W072 bridge/UI consumer is required to change to preserve current behavior. DNA OneCalc can opt into typed request construction when ready.

## Bead Set

### B073-01: Payload Inventory and Compatibility Gate

- **Status**: validated
- **Owner**: OxFml with DNA OneCalc consultation
- **Effect**: inventory all current W072 bounded string conventions and classify each as retained, replaced by typed field, or deferred.
- **Gate**: additive typed fields with W072 compatibility fallback.
- **Evidence**: compatibility decision recorded above; downstream note `HANDOFF-DNAONECALC-012` filed.

### B073-02: Typed Rule Metadata Shape

- **Status**: validated
- **Owner**: OxFml
- **Effect**: define typed metadata for:
  1. `colorScale`: ordered stops with stop kind (`min`, `max`, `percent`, `percentile`, `num`, later `formula`) and color.
  2. `dataBar`: lower/upper bounds, bar color, direction, show-bar-only, and deferred slots for negative-axis and gradient policy.
  3. `iconSet`: set kind and typed threshold sequence.
  4. `top` / `bottom`: count versus percent rank option.
  5. `aboveAverage` / `belowAverage`: equal-average and stddev multiplier options.
- **Evidence**: compile-checked public structs exported from `oxfml_core`, with JSON projection for `typed_rule`.

### B073-03: Evaluator Input Adapter

- **Status**: validated
- **Owner**: OxFml
- **Effect**: route aggregate and visualization evaluation through typed metadata first, with W072 bounded string parsing retained only if the compatibility gate chooses an additive path.
- **Evidence**: old W072 tests remain green; new typed-payload tests cover each rule family.

### B073-04: Typed Color-Scale Stop Semantics

- **Status**: validated
- **Owner**: OxFml
- **Effect**: replace color-stop string parsing with typed stop interpretation for two-stop and three-stop gradients, percent/percentile stops, and absolute numeric stops.
- **Evidence**: `typed_color_scale_payload_matches_bounded_threshold_convention`.

### B073-05: Typed Data-Bar and Icon-Set Semantics

- **Status**: validated
- **Owner**: OxFml
- **Effect**: replace data-bar and icon-set threshold parsing with typed lower/upper bounds, direction, show-bar-only, icon-set kind, and icon threshold definitions.
- **Evidence**: `typed_data_bar_payload_controls_bounds_direction_and_bar_only` and `typed_icon_set_payload_uses_explicit_thresholds`.

### B073-06: Typed Aggregate Predicate Options

- **Status**: validated
- **Owner**: OxFml
- **Effect**: replace rank and average option parsing with typed `RankRuleOptions` and `AverageRuleOptions`.
- **Evidence**: `typed_rank_and_average_payloads_replace_threshold_parsing`.

### B073-07: Surface and Handoff Update

- **Status**: filed
- **Owner**: OxFml
- **Effect**: update publication/spec docs, worklist rows, and handoff register entries for any bridge-visible field changes.
- **Evidence**: `HANDOFF-DNAONECALC-012` records the additive typed input shape and no forced W072 bridge change.

### B073-08: Regression and Compatibility Evidence

- **Status**: validated
- **Owner**: OxFml
- **Effect**: preserve W072 behavior and prove typed payload parity.
- **Evidence**:
  1. focused conditional-formatting tests,
  2. `cargo test -p oxfml_core`,
  3. `cargo fmt --all -- --check`,
  4. `git diff --check`.

Validation commands:

1. `cargo fmt --all -- --check` - passed.
2. `cargo test -p oxfml_core --test conditional_formatting_array_tests` - passed, 19 tests.
3. `cargo test -p oxfml_core` - passed.
4. `git diff --check` - passed with CRLF normalization warnings only.

## Status

- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - DNA OneCalc acknowledgement and optional typed request-construction uptake
  - future decision on whether or when to retire W072 bounded string fallback
