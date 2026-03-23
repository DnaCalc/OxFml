# OxFml Host-Managed Name and External-Name Boundary

## Purpose
Define the first implementation-facing seam for host-managed name and external-name evaluation.

This document keeps workbook object ownership in the host while making the OxFml evaluation boundary precise enough for implementation and later seam freeze work.

It is a draft boundary packet, not a final workbook-management specification.

## Core Rule
1. the host owns name objects, external-name objects, scope, storage, add/remove/rename, and workbook-link policy,
2. OxFml does not own workbook name-carrier management,
3. OxFml evaluates only the formula artifact and FEC facts the host presents,
4. once the host presents that artifact, OxFml owns parse, bind, semantic-plan, evaluator, and FEC consequence meaning.

## Authority Split

### Host-owned
1. workbook object identity for defined names and external names,
2. scope selection and workbook/sheet ownership,
3. deciding when a managed formula is presented for evaluation,
4. deciding whether an external-name lane is:
   - already resolved,
   - unresolved,
   - deferred,
   - provider-backed,
   - provider-failed,
5. broader workbook-management semantics outside the presented evaluation request.

### OxFml-owned
1. parse, bind, and semantic-plan meaning once the managed formula request is presented,
2. unresolved-name and bind-diagnostic classification inside the presented request,
3. runtime/evaluator meaning for resolved, unresolved, deferred, and provider-stage lanes,
4. FEC/F3E candidate, commit, reject, trace, and returned-value-surface meaning,
5. preservation of direct-binding-sensitive truth where cell identity still matters semantically.

### OxFunc-owned
1. built-in function catalog truth through the runtime library-context interface,
2. semantic value behavior for built-ins and callable values that cross the OxFml/OxFunc seam,
3. provider/runtime value-universe behavior where the surface is catalog-known and OxFunc-backed.

## Managed Formula Kinds
The first boundary distinguishes:
1. `defined_name_formula`
2. `external_name_formula`

This draft does not require OxFml to distinguish every workbook-host subtype beyond those two kinds.

## First Request Shape
The host-managed evaluation request should be representable with the following fields.

### Identity and placement
1. `managed_formula_kind`
2. `managed_formula_id`
3. `scope_ref`
4. `caller_anchor`
5. `formula_channel`
6. `formula_text`

### FEC inputs
1. `defined_name_bindings`
2. `direct_cell_bindings`
3. `typed_context_query_bundle`
4. `library_context_snapshot_ref`
5. `structure_context_version`

### Name/external-name resolution state
1. `external_resolution_state`
2. optional `resolved_value`
3. optional `resolved_reference`
4. optional `provider_requirement_ref`
5. optional `provider_failure_detail`

## Field Meaning

### `managed_formula_kind`
Allowed first values:
1. `defined_name_formula`
2. `external_name_formula`

### `managed_formula_id`
A host-stable identifier for the managed formula object being evaluated.

OxFml treats this as identity/provenance only.
It does not infer workbook management rules from the identifier.

### `scope_ref`
A host-stable scope token.
It may represent workbook scope, worksheet scope, or a narrower host-defined scope class.

OxFml consumes it only as scope provenance unless a later freeze packet gives it stronger semantic meaning.

### `caller_anchor`
The evaluation anchor to use for relative-reference-sensitive behavior.

This remains aligned with the broader host/runtime packet and should not be redefined differently for managed formulas.

### `formula_channel`
The same channel vocabulary already used for ordinary formula sources.

This draft does not create a special “name-only parser”.
Managed formulas are still ordinary OxFml formula artifacts once presented.

### `formula_text`
The formula body to parse/bind/evaluate.

The host may preserve its own stored/raw form separately.
OxFml only requires the presented evaluation form.

### `defined_name_bindings`
The visible name bindings for this evaluation.

This is how host-managed name resolution crosses into OxFml.
OxFml should not be asked to discover workbook name objects on its own.

### `direct_cell_bindings`
Concrete cell bindings that must be preserved where semantic truth depends on direct identity.

Managed-formula evaluation does not relax the direct-binding rule.

### `typed_context_query_bundle`
The same typed host/query bundle already being narrowed under `W041`.

Managed formulas must not receive a separate ad hoc host-query interface.

### `library_context_snapshot_ref`
The runtime library-context snapshot reference active for this evaluation.

Managed formulas use the same OxFml/OxFunc runtime seam as ordinary cell formulas.

## External Resolution State
The first boundary should treat external-name status explicitly rather than collapsing it into one generic failure.

Allowed first state families:
1. `resolved_value`
2. `resolved_reference`
3. `unresolved_name`
4. `deferred_external`
5. `provider_required`
6. `provider_failure`

## Operational Rules

### Defined-name formulas
1. the host selects the defined-name object,
2. the host presents its formula text and visible bindings,
3. OxFml evaluates it through the normal parse/bind/evaluate path,
4. OxFml returns ordinary candidate/commit/reject/trace outputs,
5. no special workbook-management side channel is required.

### External-name formulas
1. the host selects the external-name object,
2. the host decides whether the request enters OxFml as resolved, unresolved, deferred, provider-required, or provider-failed,
3. OxFml preserves that stage distinction through bind/runtime/FEC consequences,
4. OxFml must not collapse external-name provider failure into generic unknown-name behavior.

## OxFml Output Rule
Managed-formula evaluation should reuse the existing host/runtime output packet.

The intended rule is:
1. do not invent a special name-only result object,
2. reuse candidate / commit / reject / trace / returned-value-surface carriers,
3. preserve managed-formula identity and scope as provenance if replay or host diagnostics need it.

## Explicit Non-Goals
This draft does not authorize:
1. full workbook name-object lifecycle management in OxFml,
2. external workbook link policy in OxFml,
3. cross-process ABI design,
4. a new parser mode just for managed formulas,
5. collapse of direct-cell-binding-sensitive truth into name-only bindings.

## Current Open Questions
1. exact canonical spelling of the request packet fields,
2. whether `scope_ref` needs stronger first-pass semantics or should remain provenance-only,
3. whether `external_resolution_state` should be a tagged union in canonical docs or a smaller enum plus side fields,
4. how much managed-formula identity should be preserved into replay-facing artifacts in the first freeze.

## Current Draft Recommendation
For `W038`, freeze the seam as:
1. host-managed object ownership,
2. OxFml-managed formula/FEC meaning,
3. one explicit request packet for managed formulas,
4. reuse of ordinary OxFml host/runtime outputs,
5. explicit external-resolution-state classification.

That is the smallest honest boundary likely to be useful for both:
1. a direct single-host implementation,
2. an OxCalc-integrated host implementation.
