# OxFml Name-World And Runtime Registration Invalidation

## Purpose
Define the current OxFml reading of how name-bearing worlds affect bind, semantic planning, reevaluation, and invalidation.

This note exists to make one distinction explicit:
1. some changes alter the bind-visible name world and must be treated like structural changes,
2. some changes alter only runtime descriptor or provider truth and should trigger narrower reevaluation.

This document also makes the relationship between:
1. built-in and runtime-registered function names,
2. host-managed defined names,
explicit enough for host and coordinator implementation planning.

## Core Rule
For invalidation purposes, OxFml treats function names and defined names as two instances of the same broad category:
1. bind-visible name worlds.

The common rule is:
1. if the set of names visible to bind changes in a way that can alter name resolution, that is a bind/structure change,
2. if only runtime descriptor truth changes for a lane that is already syntactically and semantically admitted, that is not automatically a bind change.

## Name Worlds
The current first invalidation model distinguishes three canonical name-bearing worlds plus one planned host namespace extension for `W051`.

### 1. Function Catalog World
Owned by OxFunc through:
1. built-in function and operator catalog truth,
2. runtime library-context snapshots,
3. runtime registration and unregister of name-bindable functions.

Examples:
1. built-in `SUM`
2. a host-registered function later callable as `=MYFUNC(A1)`
3. a VBA shim function later callable as `=BookMacro(A1)`

### 2. Defined-Name World
Owned by host/coordinator workbook structure truth.

Examples:
1. workbook-defined name formulas
2. worksheet-scoped names
3. externally supplied name bindings used during bind/evaluation

### 3. Host Namespace World (`W051` planned extension)
Owned by the consuming host or OxCalc when a formula channel has host-specific reference and namespace facts outside native worksheet A1/R1C1 syntax.

Examples:
1. TreeCalc node names supplied by OxCalc,
2. explicit host paths or selectors,
3. host lambda-valued nodes,
4. host reference collections such as a child/member set.

Current rule:
1. host namespace names should map to the closest Excel defined-name lane unless an explicit TreeCalc extension is later documented,
2. lambda-valued host nodes should map to the closest Excel defined-name `LAMBDA` invocation lane unless evidence forces a separate carrier,
3. explicit host-reference syntax and explicit paths may bind through the host namespace resolver and bypass ordinary function-name ambiguity,
4. bare names and bare callees must wait for the Excel oracle matrix before OxFml freezes a host-name precedence rule.
5. `LET` / `LAMBDA` lexical variables, callable locals, captures, and returned lambdas are not a host namespace. They remain OxFml-internal bind/evaluation facts even when the oracle matrix observes their precedence against built-ins, UDFs, or defined names.

### 4. Registered-External Descriptor World
Owned by OxFunc runtime registered-external catalog truth for worksheet `CALL` / `REGISTER.ID`.

Examples:
1. `REGISTER.ID("Kernel32", ...)`
2. host API registration of a registered external target used only through `CALL`
3. VBA shim registration that is used only through the registered-external lane

This registered-external world is name-bearing in a broad product sense, but it is not automatically bind-visible as an ordinary function-name world.

## Excel Oracle Matrix Before Precedence Freeze
`W074` must settle the Excel-visible precedence rule before OxFml promotes any generic host namespace shadowing rule.

Required matrix families:
1. built-in function name in call position and non-call bare-name position,
2. registered UDF name in call position and non-call bare-name position,
3. workbook-defined name and sheet-defined name collisions with built-ins,
4. workbook-defined name and sheet-defined name collisions with registered UDFs,
5. defined-name `LAMBDA` invocation by bare call and behavior when referenced in non-call position,
6. value-like, reference-like, and lambda-valued defined names with the same identifier across workbook and sheet scopes,
7. lexical `LET` / `LAMBDA` bindings colliding with built-ins, UDFs, and defined names,
8. late UDF registration changing an unresolved call into a bindable call,
9. UDF unregister and capability-denial changing a previously bindable call,
10. defined-name add/remove/reclassification changing non-call and call classification,
11. explicit host-reference syntax selecting a host object whose display name collides with a function, UDF, or defined name.

Required row fields:
1. source position: `call_callee`, `non_call_bare_name`, `let_lambda_lexical`, or `explicit_host_reference`,
2. visible candidate set: built-in function, registered UDF, workbook-defined name, sheet-defined name, defined-name `LAMBDA`, lexical local, host namespace name,
3. observed winner and observable result class,
4. whether the result is callable, value-like, reference-like, or unresolved,
5. mutation inputs that invalidate the prepared identity or semantic-plan cache,
6. replay-visible resolution layer and diagnostics.

Until that evidence exists, the active rule is:
1. do not freeze built-in/UDF/defined-name/host-name shadowing order,
2. preserve the candidate resolution layers and diagnostics explicitly,
3. treat TreeCalc host names as defined-name-like only as a planning mapping, not as final product semantics.

Current W074 evidence split:
1. explicit host-reference bypass now has deterministic non-Excel evidence from
   OxCalc host-resolver/replay artifacts and OxFml runtime/replay facade
   projection; this supports generic host-reference pass-through, not a bare
   host-name precedence freeze,
2. capability-overlay denial now has an OxFml/OxFunc registry/editor probe;
   formula-call binding and invalidation under denied registry views remain
   open,
3. structured-reference syntax now has local collision and prepared-identity
   mutation evidence; bare table-name precedence, stable row membership/order,
   and exact header/totals packet facts remain open.

## Shared Invalidation Principle
The function catalog world and the defined-name world should be treated the same way for invalidation when they are bind-visible:
1. a newly added visible name can change binding,
2. a removed visible name can change binding,
3. a changed visible name kind can change binding,
4. a scope or precedence change can change binding,
5. an existing unresolved token may become resolvable or cease to be resolvable.

Therefore:
1. function-name registration that creates or removes ordinary formula-callable surfaces is bind-affecting,
2. defined-name add/remove/rename/scope-change is bind-affecting.

## Structural-Change Rule
The current OxFml rule is:
1. bind-visible name-world change should be treated like a structural change.

That means:
1. the relevant version key must change,
2. rebind must be required for affected formulas,
3. semantic plans pinned to the earlier bind world are stale for those formulas.

Current affected version contexts:
1. `structure_context_version`
   - for workbook-structure-owned name worlds such as defined names,
2. `LibraryContextSnapshotRef`
   - for OxFunc-owned function catalog truth and name-bindable runtime function registration,
3. host namespace version and `resolution_rule_version`
   - for generic host-reference/name hooks admitted by `W051`,
4. caller context identity
   - where host names, relative references, or explicit host references are caller-sensitive,
5. table-context identity
   - where structured-reference binding depends on `table_catalog`, `enclosing_table_ref`, or `caller_table_region`,
6. both or more than one context, where a host change affects several name worlds at once.

## Runtime-Descriptor Rule
Not every runtime registration is a bind-visible name-world change.

Current narrower rule:
1. if a change affects only the registered-external descriptor/runtime world for `CALL` / `REGISTER.ID`,
2. and does not add or remove an ordinary formula-callable function surface by name,
3. then that change is not automatically a broad bind invalidation event.

Instead:
1. formulas already using `CALL` / `REGISTER.ID` may need reevaluation,
2. formulas that do not use that lane do not need rebinding merely because the registered-external descriptor catalog changed.

## Examples

### Bind-affecting examples
1. OxFunc snapshot gains ordinary callable surface `MYFUNC`
   - formulas containing `MYFUNC(...)` may newly bind
   - formulas previously reporting unknown-function diagnostics for `MYFUNC` may become valid
2. a workbook defined name `SalesTax` is added or removed
   - formulas referencing `SalesTax` may newly bind or become unresolved
3. a defined name changes from value-like to reference-like
   - bind classification may change
4. a function or defined name changes visible scope or precedence
   - resolution may change

### Runtime-descriptor-only examples
1. a `REGISTER.ID` result is added for `Kernel32!MulDiv`
   - formulas using `REGISTER.ID` / `CALL` may change evaluation outcome
   - ordinary formulas without that lane do not need broad rebind
2. a registered external descriptor is removed
   - descriptor-dependent `CALL` evaluations may fail or change
   - ordinary formula name resolution is unchanged unless that registration also supplied a bind-visible callable surface

## Snapshot And Version Guidance
The current intended split is:

### Function catalog changes
If the change creates or removes an ordinary formula-callable surface, OxFunc should:
1. publish a new `LibraryContextSnapshot`,
2. give it a new snapshot generation/ref,
3. let OxFml or the host treat formulas pinned to the old snapshot as stale for bind/semantic-plan purposes where affected.

### Defined-name changes
If the change creates, removes, renames, or reclassifies a visible defined name, the host/coordinator should:
1. change `structure_context_version`,
2. treat formulas pinned to the old structure context as stale for bind where affected.

### Host namespace changes
If a host namespace change creates, removes, renames, or reclassifies a visible host name/reference in a formula channel that admits host-context names, the host/coordinator should:
1. change the host namespace version or structure-context version used by the `HostFormulaContext`,
2. treat formulas pinned to the old host context as stale for bind where affected,
3. preserve whether the change was caused by function registry mutation, workbook/defined-name mutation, or host namespace/model mutation in replay-visible invalidation facts.

### Structured-reference context changes
If a table-context change creates, removes, renames, or reclassifies table or column meaning visible to structured-reference binding, the host/coordinator should:
1. change the table-context identity or structure-context version used by the prepared formula,
2. rebind formulas whose structured-reference syntax mentions the changed table/column or depends on the enclosing table/current-row context,
3. preserve table-name-versus-defined-name disambiguation as an OxFml bind consequence over host-owned table packet truth.

### Registered-external descriptor changes
If the change only affects `CALL` / `REGISTER.ID` descriptor truth, the host/OxFunc runtime should:
1. preserve the mutation through the registered-external packet lane,
2. generate any required new runtime snapshot generation if the OxFunc catalog model requires it,
3. avoid treating the change as universal bind invalidation unless a bind-visible function-name world also changed.

## Affected-Formula Discovery
The best current implementation strategy is to maintain explicit usage indexes rather than relying only on coarse full-workbook invalidation.

### For bind-visible function catalog changes
Index formulas by:
1. referenced function surface names,
2. canonical function ids when known,
3. unresolved function identifiers from bind or semantic diagnostics.

On snapshot change:
1. directly rebind formulas that mention changed surface names or ids,
2. also rebind formulas with unresolved function identifiers that may now resolve,
3. conservatively rebind more broadly if the changed set is unknown.

### For defined-name changes
Index formulas by:
1. referenced defined-name identifiers,
2. unresolved name identifiers,
3. scope-sensitive name usage where workbook/sheet visibility matters.

On defined-name change:
1. rebind formulas that mention the changed identifier,
2. also rebind formulas with unresolved identifiers that may now resolve under the changed scope,
3. conservatively rebind more broadly if scope or precedence effects are not cheaply indexable.

### For host namespace changes
Index formulas by:
1. host reference handles where already known,
2. host namespace identifiers or explicit path/source-token identities,
3. unresolved host identifiers,
4. caller-context-dependent usages.

On host namespace change:
1. rebind formulas that mention changed host identifiers or handles,
2. also rebind formulas with unresolved identifiers that may now resolve under the changed host context,
3. conservatively rebind more broadly if caller-context or precedence effects are not cheaply indexable.

### For structured-reference context changes
Index formulas by:
1. structured-reference table names and table ids,
2. structured-reference column names and column ids,
3. omitted-table-name/current-row-sensitive usages,
4. unresolved table or column identifiers.

On table-context change:
1. rebind formulas that mention changed table or column identifiers,
2. also rebind formulas with omitted-table-name forms when the enclosing table or caller region changed,
3. conservatively rebind more broadly if table-name-versus-defined-name precedence or caller-region effects are not cheaply indexable.

### For registered-external descriptor changes
Index formulas by:
1. presence of worksheet `CALL`,
2. presence of worksheet `REGISTER.ID`,
3. stable registration id when already known,
4. direct `{ library, procedure, type_text }` triples when statically visible.

On registered-external mutation:
1. reevaluate formulas using the affected descriptor or target,
2. rebind only if the mutation also changed the bind-visible function-name world.

## Host And Coordinator Rule
Hosts and OxCalc should not maintain a competing semantic function catalog locally.

The intended rule is:
1. OxFunc populates the initial built-in function catalog,
2. OxFunc owns runtime function-catalog mutation,
3. host/OxCalc own workbook structure and defined-name worlds,
4. OxFml normalizes formula/bind/runtime packets across those worlds,
5. invalidation should follow the world that actually changed:
   - function catalog snapshot
   - structure context
   - registered-external runtime descriptor set.

## Non-Claims
This note does not claim:
1. a final global invalidation algorithm,
2. a final packet ABI for snapshot-change notifications,
3. that every registration must always force both snapshot and structure invalidation,
4. that registered externals used only through `CALL` are bind-visible ordinary function names,
5. that host/coordinator can skip conservative fallback rebinding when dependency/index truth is incomplete.
6. that TreeCalc host names have product-specific precedence in OxFml before the `W074` Excel oracle matrix justifies an extension,
7. that OxFml exposes lexical `LET` / `LAMBDA` internals as host-visible namespace bindings.

## Current Recommendation
The current recommended operational model is:
1. treat function-name registration and defined-name mutation as the same class of bind-visible structural invalidation,
2. treat `CALL` / `REGISTER.ID` descriptor mutation as a narrower runtime reevaluation lane unless it also changes ordinary formula-callable surfaces,
3. keep usage indexes for:
   - function-name references,
   - defined-name references,
   - unresolved identifiers,
   - registered-external descriptor usage,
4. use new snapshot generations and structure-context versions explicitly rather than hiding these changes in mutable globals.
