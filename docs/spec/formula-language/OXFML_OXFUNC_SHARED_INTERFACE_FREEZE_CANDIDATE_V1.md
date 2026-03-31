# OXFML_OXFUNC_SHARED_INTERFACE_FREEZE_CANDIDATE_V1.md

## Purpose
Capture the current OxFml-side shared interface freeze candidate for the OxFml <-> OxFunc boundary.

This is not a product API.
It is the current-phase minimum shared packet and carrier set that OxFml reads as aligned with OxFunc's freeze-promotion handoff and acceptable for explicit shared seam-freeze text for the narrowed seam families.

## Current-Phase Scope
This freeze candidate covers only the currently exercised and narrowed OxFml/OxFunc seam families:
1. typed context/query bundle families for the admitted host-observing slice,
2. returned-value surface families for the admitted publication-sensitive slice,
3. runtime library-context consumer shape,
4. minimum callable carrier,
5. registered-external worksheet `CALL` / `REGISTER.ID` packet family.

This freeze candidate does not claim:
1. full product host API closure,
2. full rich-value object-model closure,
3. full grouped-aggregation option-matrix closure,
4. broader UDF product surface closure,
5. final cross-process transport ABI.

## 1. Typed Context Query Bundle
Current first shared typed query family set for the covered seam-heavy slice:
1. `ReferenceResolver`
2. `CellInfo`
3. `Info`
4. `Rtd`
5. `Image`

Current working rules:
1. the bundle remains capability-scoped and typed rather than object-handle based,
2. host/query families outside the current admitted slice remain outside this freeze candidate,
3. `TypedContextQueryFamily::Image` is the first freeze name for the `IMAGE` host-query lane,
4. preserved reference identity remains explicit where these query families depend on it.

## 2. Returned Value Surface
Current first shared returned-value split:
1. `OrdinaryValue`
2. `ValueWithPresentation`
3. `RichValue`
4. `TypedHostProviderOutcome`

Current working rules:
1. `HYPERLINK` preserves `ValueWithPresentation`,
2. `IMAGE` preserves `RichValue` with `rich_value_type_name = "_webimage"`,
3. published worksheet fallback remains separate from the semantic return carrier,
4. richer return-surface factoring remains outside this current-phase freeze candidate.

## 3. Runtime Library Context Consumer Shape
Current shared runtime consumer direction:
1. `LibraryContextProvider`
2. immutable `LibraryContextSnapshot`
3. `LibraryContextSnapshotRef`

Current working rules:
1. the runtime-only consumer shape is preferred over direct CSV/export mirroring,
2. the pinned snapshot export remains a stabilization and test-pinning artifact, not the preferred final runtime interface,
3. snapshot identity and generation remain explicit for bind, semantic-plan, replay, and invalidation correlation,
4. richer export-description fields may remain outside the runtime-only shared minimum where explicit mapping preserves meaning.

## 4. Minimum Callable Carrier
Current minimum shared callable carrier:
1. opaque callable identity or token
2. `origin_kind`
3. `capture_mode`
4. `arity_shape`
5. `invocation_contract_ref`

Current working rules:
1. typed invocation over opaque callable identity is the preferred boundary,
2. no additional explicit invocation-model field is currently required beyond `invocation_contract_ref`,
3. parameter-name, capture-name, and body-kind detail may remain provenance/replay detail rather than minimum shared transport fields for the current phase,
4. any callable identity is too weak if origin, capture mode, arity shape, or invocation-contract meaning become unrecoverable,
5. adopted defined-name callable preservation is part of the current first-pass callable freeze pressure rather than a later extension.

## 5. Registered External Runtime Packet Family
Current direct shared packet set:
1. `RegisterIdRequest { library_name, procedure, declared_type_text }`
2. `RegisteredExternalDescriptor { stable_registration_id, register_id, origin_kind, display_name, library_name, procedure, declared_type_text }`
3. `RegisteredExternalCallRequest { target, invocation_args }`
4. `RegisteredExternalTarget::{ RegisterId, Direct }`
5. `RegisteredExternalProvider`

Current adjacent OxFml funnel packet family:
1. `RegisteredExternalCatalogMutationRequest`
2. `RegisteredExternalCatalogMutationResult`
3. `RegisteredExternalCatalogController`

Current working rules:
1. `RegisteredExternalProvider` remains separate from `HostInfoProvider`,
2. descriptor-driven dereference and general type coercion remain OxFunc-owned,
3. the current seven-field `RegisteredExternalDescriptor` is the shared minimum field set for the current phase,
4. the mutation/controller family remains OxFml-owned for the current phase unless later concrete evidence forces promotion into the shared runtime packet family,
5. bind-visible registration or unregister yields new `LibraryContextSnapshot` generation plus bind invalidation where the visible function or name world changes,
6. `CALL` / `REGISTER.ID`-only descriptor mutation yields targeted reevaluation by default.

## 6. Boundary Non-Claims
This freeze candidate does not claim:
1. that OxFml/OxFunc host security or coordinator policy is frozen,
2. that broader snapshot-acknowledgment or publication consequences from register/unregister are part of the current packet,
3. that richer callable provenance belongs in the minimum shared carrier,
4. that broader typed query families beyond the current covered slice are frozen,
5. that broader admission-matrix characterization is blocked on this current-phase freeze.

## 7. Current Promotion Read
Current OxFml read is:
1. the packet and carrier set above is converged enough to serve as the explicit current-phase shared interface freeze candidate,
2. OxFunc's `docs/handoffs/HANDOFF_SHARED_INTERFACE_FREEZE_PROMOTION_TO_OXFML_V1.md` is acceptable from the OxFml side as the current shared freeze wording for the narrowed seam families,
3. later widening should be concrete-mismatch-driven rather than reopening broad seam theory.
