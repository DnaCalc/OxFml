*Posted by Codex agent on behalf of @govert*

# HANDOFF-CALC-005 W074 Name/Call Freeze Follow-Up

## Purpose

This is the OxFml W074 follow-up to
`HANDOFF_CALC_005_OXFML_RECEIPT.md`. It gives OxCalc the current
evidence-backed name/call rule to consume for W051/W056 TreeCalc host names and
lambda-valued nodes.

This handoff does not add TreeCalc syntax to OxFml. It freezes the current
generic mapping rule only: TreeCalc host names enter OxFml as product-neutral
host-name bind packets mapped to the closest Excel defined-name or
defined-name-`LAMBDA` lane, while explicit TreeCalc references continue to use
the generic explicit host-reference hook.

## Evidence Base

Canonical matrix:

1. `docs/spec/formula-language/W074_CALC005_NAME_CALL_PRECEDENCE_ORACLE_MATRIX.csv`

Retained observation notes:

1. `docs/spec/formula-language/W074_CALC005_TABLE_NAME_COLLISION_ORACLE_20260522.md`
2. `docs/spec/formula-language/W074_CALC005_TABLE_NAME_ORACLE_20260523.md`
3. `docs/spec/formula-language/W074_CALC005_NON_TABLE_NAME_CALL_ORACLE_20260524.md`

Local runtime/replay evidence already carried in W074:

1. explicit host-reference pass-through with replay-visible source span/token,
   opaque selector, resolution layer, diagnostics, and identity inputs;
2. host namespace version participation in prepared identity even without
   explicit host-reference bind results;
3. product-neutral `RuntimeHostNameBinding` mapped to the defined-name /
   defined-name-`LAMBDA` evaluator lane;
4. registry snapshot and capability-overlay runtime formula-call identity
   evidence;
5. no-host LET/LAMBDA lexical guardrail for DnaOneCalc-style execution;
6. generic structured-reference bind packets and table-context identity facts.

## Current Frozen Rule

For the current W051/W056 host-name mapping, OxFml freezes these rules:

1. **Explicit host references bypass ordinary name/call ambiguity.**
   Explicit host-reference syntax is host supplied and resolved through the
   generic host hook. It must emit replay-visible resolution-layer facts and
   typed diagnostics. This lane may select a host object whose display name
   collides with a function, UDF, defined name, or table name.

2. **Built-in functions keep the call-callee frontier.**
   In call position, Excel-observed built-ins such as `SUM` and `N` win over
   same-named workbook or sheet defined names, defined-name `LAMBDA` values,
   and same-named lexical scalar or callable locals. Bare non-call use can
   still resolve to a same-named defined name or lexical local.

3. **Registered UDFs are callable, not bare value names.**
   A registered UDF with no same-named value/name object is callable in call
   position and unresolved in bare non-call position.

4. **Defined names shadow registered UDFs for the observed non-built-in
   collisions.**
   Workbook or sheet defined names win over same-named registered UDFs in both
   call and non-call positions. If the winning defined name is scalar, call
   position produces the observed non-callable error. If the winning defined
   name is a `LAMBDA`, call position invokes it and bare non-call position
   returns the observed `#CALC!` value-reference behavior.

5. **Sheet scope wins over workbook scope where visible.**
   Sheet-scoped defined names, including lambda-valued names, win on their
   sheet. Workbook-scoped names remain visible where the sheet-scoped name is
   not visible. Structure/sheet/workspace context is therefore a prepared
   identity input for host-name mapping.

6. **Lexical LET/LAMBDA names remain OxFml-internal.**
   Lexical locals are not host namespace entries and are not exposed as OxCalc
   references. For non-built-in identifiers, lexical callable locals can win
   call position against external UDF/defined-name candidates, and lexical
   scalar locals can produce the observed non-callable error in call position.
   For built-in names, the built-in call-callee frontier remains intact.

7. **Defined-name kind and presence are invalidation inputs.**
   Mutating a defined name from scalar to `LAMBDA`, or deleting it, changes the
   observed result of existing formulas without changing formula text. Prepared
   identity must account for defined-name namespace version, kind, and scope
   where those facts can change bind or prepared-call shape.

8. **Structured references remain on the table-context lane.**
   Table-name observations constrain table behavior and collision diagnostics,
   but they do not turn TreeCalc host names into table semantics. Node-associated
   TreeCalc tables should continue to enter through generic structured-reference
   packets plus OxCalc-owned table catalog/lowering.

## OxCalc Consumption Rule

OxCalc may now consume this mapping for W056:

1. TreeCalc host value names map to the Excel defined-name value lane.
2. TreeCalc lambda-valued host nodes map to the Excel defined-name-`LAMBDA`
   lane.
3. A TreeCalc host name that collides with a built-in function must not override
   the built-in in ordinary call-callee position. Use explicit host-reference
   syntax or a future explicitly-evidenced extension if the product needs to
   call such a host node.
4. A TreeCalc host name that collides with a registered UDF follows the
   defined-name lane: host value/lambda names can shadow the UDF when supplied
   as visible host-name bind packets.
5. Caller context, host namespace version, structure/workspace context,
   registry snapshot identity, and resolution rule version remain identity and
   invalidation inputs.
6. OxCalc must keep TreeCalc selector parsing, table catalogs, reference
   carriers, dependencies, and invalidation semantics out of OxFml. OxFml only
   sees product-neutral host-name bind packets, explicit host-reference bind
   results, and generic structured-reference packets.

## Remaining Non-Blocking W074 Work

These items remain W074 work, but they no longer block OxCalc's W051/W056
name/call mapping rule:

1. broader bind/editor cache migration beyond the current runtime formula-call
   and host-name evidence;
2. additional Excel oracle rows if future product features admit new host
   name-world classes;
3. full W036 structured-reference grammar/table semantics beyond the W056
   packet and identity slice.

## Status

- execution_state: complete for this W074/CALC-005 freeze handoff
- scope_completeness: scope_complete for the current W051/W056 host-name
  mapping rule
- target_completeness: target_complete for OxCalc consumption of current
  TreeCalc host value names and lambda-valued host nodes
- integration_completeness: partial until OxCalc acknowledges this handoff and
  exercises the W056 host-name/node-as-function paths through the real bridge
- open_lanes:
  - OxCalc W056 consumption and retained evidence,
  - broader OxFml bind/editor cache migration,
  - future product extension if TreeCalc intentionally wants host names to
    override built-in call-callee resolution.
