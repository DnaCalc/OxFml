*Posted by Codex agent on behalf of @govert*

# HANDOFF-OXFUNC-006: W070 Locale Profile Expansion Request

Status: `consumed`
Source repo/workset: `OxFml/W070`
Target repo/workset: `OxFunc/TBD`
Filed date: `2026-05-04`
Related inbound: `../DnaOneCalc/docs/HANDOFF_OXFML_LOCALE_EXPANSION.md`

## Purpose

Escalate the locale-profile dependency found while processing DNA OneCalc's locale expansion handoff. OxFml should not create a second comprehensive locale registry; locale profile identity, Excel locale-code mapping, locale defaults, and format-code token policy need to stay canonical in OxFunc's locale-format seam.

## Current OxFunc State Observed

OxFunc exposed a first locale-profile breadth slice:

1. expanded `LocaleProfileId` variants for the DNA OneCalc locale set,
2. `LocaleProfileId::stable_name()`,
3. `LocaleProfileId::from_bcp47_language_tag(...)`,
4. `CANONICAL_LOCALE_PROFILE_IDS` / `LOCALE_PROFILE_IDS`,
5. `format_profile(id)` carrying decimal, thousands, list, currency, date, time, and currency-decimal defaults.

OxFml can consume that slice for locale-keyed month/weekday rendering and separator-aware General numeric rendering.

OxFunc W094 later exposed the final profile semantics requested below. OxFml consumed that final surface on 2026-05-06.

## Requested OxFunc Surface

OxFml requests this ideal final-state `FormatProfile` surface:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateComponentOrder {
    Mdy,
    Dmy,
    Ymd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrencyPlacement {
    Before,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrencySpacing {
    None,
    Space,
    NarrowNoBreakSpace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrencyNegativePattern {
    LeadingMinus,
    TrailingMinus,
    Parentheses,
    LeadingMinusBeforeSymbol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatCodeTokenPolicy {
    /// Stored OOXML format code tokens are invariant Excel tokens:
    /// "." is the decimal token and "," is the grouping/scaling token.
    InvariantExcel,
    /// Optional future mode for locale-authored format strings where decimal
    /// and grouping tokens are localized before semantic parsing.
    LocalizedExcel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatProfile {
    pub id: LocaleProfileId,
    pub stable_name: &'static str,
    pub excel_lcid: Option<u16>,

    pub decimal_separator: &'static str,
    pub thousands_separator: &'static str,
    pub list_separator: &'static str,
    pub date_separator: &'static str,
    pub time_separator: &'static str,

    pub short_date_order: DateComponentOrder,
    pub short_date_pattern: &'static str,
    pub two_digit_year_pivot: Option<i64>,

    pub currency_symbol: &'static str,
    pub currency_decimals: i32,
    pub currency_placement: CurrencyPlacement,
    pub currency_spacing: CurrencySpacing,
    pub negative_currency_pattern: CurrencyNegativePattern,

    pub format_code_decimal_token: &'static str,
    pub format_code_group_token: &'static str,
    pub format_code_token_policy: FormatCodeTokenPolicy,
}
```

OxFml also requests:

1. `LocaleProfileId::from_excel_lcid(lcid: u16) -> Option<Self>` for custom format locale prefixes such as `[$-0409]`, `[$-0407]`, `[$-040C]`, `[$-0411]`, `[$-0809]`, and aliases used by the supported locale set.
2. `format_profile(id)` remains the single canonical profile constructor.
3. Existing fields remain source-compatible if feasible; otherwise coordinate a single breaking migration so consumers do not add transitional shims.
4. Deterministic OxFunc tests covering the profile matrix, LCID mapping, and at least one non-US profile for date order, currency placement, and format-code token policy.
5. No requirement for OxFunc to parse OxFml custom format grammar or render dates/numbers; OxFunc owns locale facts, OxFml consumes them.

## Why OxFml Is Blocked

OxFunc's first locale-profile breadth slice unblocks locale-keyed name-table consumption, but the remaining grammar and parser lanes still need canonical profile facts:

1. date parsing cannot safely interpret ambiguous local short dates without `short_date_order`,
2. currency parsing/rendering cannot safely match local Excel defaults without placement, spacing, and negative-pattern facts,
3. custom number-format parsing must distinguish invariant OOXML format-code tokens from localized display separators,
4. locale-prefix grammar for `[$-040C]...` needs an OxFunc-owned LCID-to-profile map.

Adding those facts directly in OxFml would make OxFml the owner of a duplicate locale registry. That conflicts with the ownership direction established by the OxFunc locale-format seam.

OxFml recorded this as `BLK-FML-005`; it is now resolved after OxFunc W094 and OxFml consumption.

## OxFml Follow-Up After OxFunc Lands

After OxFunc exposed the ideal profile API, OxFml added:

1. locale-aware short-date parser branches driven by `short_date_order`,
2. currency parser/rendering branches driven by placement, spacing, and negative-pattern fields,
3. custom numeric format parser behavior driven by explicit format-code token policy,
4. locale-prefix custom-format grammar coverage driven by `from_excel_lcid(...)`,
5. publication-surface locale profile evidence that does not depend on OxFml-local profile facts.

## OxFml Consumption Evidence

OxFml consumes the final W094 fields for:

1. `DateComponentOrder`-driven short-date parsing,
2. currency placement, spacing, and negative-pattern rendering/parsing,
3. invariant format-code decimal/group token parsing for canonical profiles,
4. `LocaleProfileId::from_excel_lcid(...)` locale-prefix custom-format rendering,
5. explicit localized token-policy fixture profiles for existing `TEXT(...)` separator-context evidence.

Validation:

1. `cargo test -p oxfml_core --test locale_format_expansion_tests` - passed, 6 tests.
2. `cargo test -p oxfml_core --test ftc_0288_separator_context_tests --test ftc_0288_trailing_comma_separator_context_tests --test ftc_0288_adjacent_matrix_tests --test ftc_0288_rule_edge_tests` - passed, 9 tests.

`BLK-FML-005` is resolved in OxFml.

## Evidence

OxFml tracking files:

1. `CURRENT_BLOCKERS.md` - `BLK-FML-005` resolved 2026-05-06
2. `docs/worksets/W070_dnaonecalc_formatting_handoff_processing.md`

## Non-Claims

This handoff does not ask OxFunc to implement OxFml formatting grammar behavior or DNA OneCalc UI cleanup. It asks only for canonical locale identity/profile breadth so OxFml can consume it without creating a competing source of truth.
