*Posted by Codex agent on behalf of @govert*

# HANDOFF-OXFUNC-006: W070 Locale Profile Expansion Request

Status: `filed`
Source repo/workset: `OxFml/W070`
Target repo/workset: `OxFunc/TBD`
Filed date: `2026-05-04`
Related inbound: `../DnaOneCalc/docs/HANDOFF_OXFML_LOCALE_EXPANSION.md`

## Purpose

Escalate the locale-profile dependency found while processing DNA OneCalc's locale expansion handoff. OxFml should not create a second comprehensive locale registry; locale profile identity and constants need to stay canonical in OxFunc's locale-format seam.

## Requested OxFunc Surface

OxFml needs canonical locale profile identities and profile constants beyond the current two-profile surface:

1. `LocaleProfileId` variants for the requested DNA OneCalc locale set.
2. `FormatProfile` constants or constructors carrying decimal separator, thousands separator, list separator, currency symbol, date separator, time separator, and currency decimals.
3. Clear compatibility expectations for workbook date systems and profile ids.
4. Stable names that OxFml can use in publication surfaces and locale-keyed formatter/parser tests.

## Why OxFml Is Blocked

OxFml can currently construct only:

1. `LocaleProfileId::EnUs`
2. `LocaleProfileId::CurrentExcelHost`

Adding locale-keyed month/weekday names, parser branches, General rendering expectations, and optional locale-prefix custom-format parsing directly in OxFml would make OxFml the owner of a duplicate locale registry. That conflicts with the ownership direction established by the OxFunc locale-format seam.

OxFml has recorded this as `BLK-FML-005`.

## OxFml Follow-Up After OxFunc Lands

Once OxFunc exposes the expanded canonical locale profile API, OxFml can add:

1. locale-keyed month and weekday render tables,
2. locale-aware date/number parser branches,
3. General rendering tests keyed by profile,
4. publication-surface locale profile evidence,
5. optional locale-prefix custom-format grammar coverage.

## Evidence

OxFml tracking files:

1. `CURRENT_BLOCKERS.md` - `BLK-FML-005`
2. `docs/worksets/W070_dnaonecalc_formatting_handoff_processing.md`

## Non-Claims

This handoff does not ask OxFunc to implement OxFml formatting grammar behavior or DNA OneCalc UI cleanup. It asks only for canonical locale identity/profile breadth so OxFml can consume it without creating a competing source of truth.
