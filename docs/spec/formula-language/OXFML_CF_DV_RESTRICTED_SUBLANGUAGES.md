# OxFml Conditional-Formatting and Data-Validation Restricted Sublanguages

## Purpose
This document defines the first honest OxFml-local floor for conditional-formatting (`CF`) and
data-validation (`DV`) formulas as restricted, host-managed formula carriers.

These are not treated as ordinary worksheet-cell carriers.

## Carrier Ownership
Hosts own the surrounding carrier records and lifecycle for:
1. conditional-formatting rules,
2. data-validation rules,
3. target-range attachment and rule-field management.

OxFml owns:
1. formula admission for the exercised restricted floor,
2. restriction classification,
3. the formula-semantic meaning of the currently modeled host fields,
4. the distinction between `CF` and `DV` restriction profiles.

## Current Local Floor
For the current local floor:
1. `CF` formulas use carrier kind `ConditionalFormatting`,
2. `DV` formulas use carrier kind `DataValidation`,
3. both carriers currently reuse the ordinary worksheet parser/binder for the admitted syntax
   subset,
4. both carriers are validated through an explicit restricted-carrier validation step.

The current local restriction floor rejects these reference/operator families for both carriers:
1. union reference operator,
2. intersection reference operator,
3. spill-reference operator,
4. external references.

The current local floor does not claim broader parity for:
1. structured references,
2. table-context-sensitive formulas,
3. array-constant or 3-D reference policy,
4. UI/rendering policy.

## Formula-Semantic Host Fields
The current local floor models the following host fields explicitly:

### Conditional Formatting
1. `target_ranges`
2. `rule_kind`
3. optional `operator`
4. threshold-bearing fields such as `cfvo@val`

### Data Validation
1. `target_ranges`
2. `validation_kind`
3. optional `operator`
4. formula slot identity such as `formula1` or `formula2`

These are treated as formula-semantic carrier facts, not generic styling noise.

## Distinct Restriction Profiles
The current local floor keeps the carrier profiles distinct:
1. `CF` uses `cf_restricted_not_equal_to_dv`
2. `DV` uses `dv_restricted_not_equal_to_cf`

Similarity is not treated as license to collapse them into one identical profile.

## Explicit Residuals
The following remain outside the current local floor:
1. broader carrier-specific admissibility rules from the full `MS-OE376` family,
2. rendering or UI policy,
3. broader runtime/coordinator consequence handling for non-cell carriers,
4. structured-reference/table-aware `CF`/`DV` semantics.

## Current Deterministic Evidence
The current local evidence lives in:
1. `crates/oxfml_core/tests/w047_host_readiness_tests.rs`
2. `crates/oxfml_core/src/carrier.rs`
