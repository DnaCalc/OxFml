# W074-CALC005 Table-Name Collision Oracle Evidence 2026-05-22

Scope: retained black-box Excel COM 16.0 evidence for the W074-CALC005-014
table-name versus workbook defined-name collision row. This evidence narrows
the table-name residual; it does not freeze the full W074 name/call precedence
rule.

## Observations

1. If a table named `Table1` already exists, adding a workbook defined name
   `Table1` is rejected by Excel with: `A table with that name already exists.
   Select a different name.`
2. If a workbook defined name `Table1 = 99` exists first, Excel accepts a
   ListObject renamed to `Table1`.
3. In that collision state, bare `=Table1` stores as `=Table1` and evaluates to
   `99`; the workbook defined name wins the bare non-call position.
4. In that same collision state, authoring `=Table1[Amount]`,
   `=SUM(Table1[Amount])`, or `=ROWS(Table1[Amount])` through COM is rejected
   with `0x800A03EC`.

Observed output from the collision-authoring probe:

```json
[
  {
    "address": "C1",
    "formula": "=Table1",
    "set": "accepted",
    "stored": "=Table1",
    "text": "99",
    "value": 99.0
  },
  {
    "address": "C2",
    "formula": "=SUM(Table1[Amount])",
    "set": "rejected",
    "error": "0x800A03EC"
  },
  {
    "address": "C3",
    "formula": "=ROWS(Table1[Amount])",
    "set": "rejected",
    "error": "0x800A03EC"
  },
  {
    "address": "C4",
    "formula": "=Table1[Amount]",
    "set": "rejected",
    "error": "0x800A03EC"
  }
]
```

## W074 Interpretation

For W074-CALC005-014:

1. bare non-call `=Table1` resolves to the workbook defined name in the observed
   defined-name-first collision;
2. table-created-first prevents the defined-name collision from being created;
3. structured-reference syntax is not admitted in the observed collision state,
   because Excel rejects the formula at authoring time;
4. non-collision structured-reference handling remains governed by the generic
   table-context packet and structured-reference bind-record evidence.

Remaining W074 blockers are broader than this row: host namespace mutation
invalidation beyond explicit host-reference pass-through, remaining workbook /
sheet / UDF / defined-name combinations, and broader full table/name closure.
