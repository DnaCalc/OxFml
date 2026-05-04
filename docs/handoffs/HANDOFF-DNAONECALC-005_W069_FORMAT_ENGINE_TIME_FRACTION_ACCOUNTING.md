*Posted by Codex agent on behalf of @govert*

# HANDOFF-DNAONECALC-005: W069 Format Engine Time, Fraction, And Accounting Landing

Status: `filed`
Source repo/workset: `OxFml/W069`
Target repo/workset: `DnaOneCalc/TBD`
Filed date: `2026-05-04`
Related inbound: `../DnaOneCalc/docs/HANDOFF_OXFML_FORMAT_ENGINE_TIME_FRACTION_ACCOUNTING.md`

## Purpose

Record OxFml's response to the DnaOneCalc observation that user-supplied time format codes such as `HH:mm:ss` were rejected by `render_with_code(...)`, causing publication to fall back to `DateLike` date-only rendering for `NOW()`.

## OxFml Landing Summary

OxFml now has bounded support for the day-to-day format families called out by the handoff:

1. time tokens `h`, `hh`, `m`, `mm`, `s`, `ss`,
2. `AM/PM` and `am/pm` 12-hour rendering,
3. date/time composites such as `m/d/yyyy h:mm` and `yyyy-mm-dd hh:mm:ss`,
4. elapsed time `[h]`, `[m]`, `[s]`,
5. simple fractions such as `?/?`, `??/??`, `# ?/?`, `# ??/??`, and `0/0`,
6. common accounting parentheses patterns.

The publication surface now honors a recognized user-supplied time format code before consulting presentation-hint fallback behavior.

## Downstream Cleanup Now Unblocked

After consuming this OxFml revision, DNA OneCalc can remove `<NOT IMPLEMENTED>` markers for the bounded live families that are just custom-code presets over:

1. time,
2. datetime,
3. simple fraction,
4. common accounting parentheses.

DNA OneCalc should keep any markers for format families beyond this bounded OxFml landing, including full custom-format grammar, text sections, and UI-specific accounting alignment.

## Evidence

Relevant OxFml files:

1. `crates/oxfml_core/src/format/datetime.rs`
2. `crates/oxfml_core/src/format/number.rs`
3. `crates/oxfml_core/src/publication/mod.rs`
4. `crates/oxfml_core/tests/format_time_fraction_accounting_tests.rs`
5. `crates/oxfml_core/tests/ftc_0654_fraction_format_engine_tests.rs`

Focused evidence covers:

1. `HH:mm:ss` rendering,
2. AM/PM noon and midnight transitions,
3. datetime composites,
4. elapsed-time tokens,
5. simple fractions through formatter/evaluator/runtime/adapter paths,
6. explicit publication formatting for a `DateLike` value with `HH:mm:ss`.

## Non-Claims

This handoff does not claim:

1. full Excel custom-format grammar parity,
2. DnaOneCalc has removed all UI markers,
3. exact UI column alignment for accounting display,
4. shared coordinator-facing seam changes.
