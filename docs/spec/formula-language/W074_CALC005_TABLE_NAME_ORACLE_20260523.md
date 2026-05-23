# W074-CALC005 Table-Name Oracle Evidence 2026-05-23

Scope: retained black-box Excel COM 16.0 evidence for table-name,
structured-reference, table rename, column rename, sheet-defined-name collision,
and UDF/table-name collision rows that affect W074/W056 table support
admission. This evidence narrows the table-adjacent W074 residual; it does not
freeze generic host-name or TreeCalc-specific name/call semantics.

Excel build:
1. `Application.Version = 16.0`
2. `Application.Build = 20026`

## Observations

1. A table name without a defined-name collision is visible in non-call position:
   bare `=TableOnly` evaluated to the first table data value observed (`10`).
2. The same table name is not an ordinary callable surface: `=TableOnly()`
   stored successfully but evaluated to `#REF!`.
3. Structured references for the same table were admitted and evaluated:
   `=SUM(TableOnly[Amount])` returned `60`,
   `=ROWS(TableOnly[Amount])` returned `3`, and
   `=TableOnly[Amount]` returned the first observed value (`10`) in the probe
   cell.
4. Renaming table `TableA` to `TableB` rewrote stored formula
   `=SUM(TableA[Amount])` to `=SUM(TableB[Amount])` and preserved value `60`.
5. Renaming table column `Amount` to `Price` rewrote the stored formula again
   to `=SUM(TableB[Price])` and preserved value `60`.
6. A sheet-scoped defined name created before the table can collide with a
   ListObject renamed to the same display name. In that state bare
   `=TableSheet` evaluated to the sheet-defined name value `88`, while
   `=SUM(TableSheet[Amount])` and `=TableSheet[Amount]` were rejected at formula
   authoring with `0x800A03EC`.
7. When a table already exists, adding a sheet-scoped defined name with the same
   display name was rejected with the same table-name collision message
   observed for workbook names.
8. A VBA UDF control in the same workbook (`=PlainUdf()`) evaluated to `123`.
   With a table and a UDF both named `TableUdf`, `=TableUdf()` evaluated to
   `#REF!`, bare `=TableUdf` evaluated to the first table data value observed
   (`10`), and `=SUM(TableUdf[Amount])` evaluated to `30`. This probe supports
   table-name precedence for that table/UDF collision shape only; it does not
   freeze generic UDF/defined-name precedence.

Observed output:

```json
[
  {
    "scenario": "table_only_bare_and_structured",
    "formula": "=TableOnly",
    "set": "accepted",
    "stored": "=TableOnly",
    "text": "10",
    "value": 10.0
  },
  {
    "scenario": "table_only_bare_and_structured",
    "formula": "=TableOnly()",
    "set": "accepted",
    "stored": "=TableOnly()",
    "text": "#REF!",
    "value": -2146826265
  },
  {
    "scenario": "table_only_bare_and_structured",
    "formula": "=SUM(TableOnly[Amount])",
    "set": "accepted",
    "stored": "=SUM(TableOnly[Amount])",
    "text": "60",
    "value": 60.0
  },
  {
    "scenario": "table_only_bare_and_structured",
    "formula": "=ROWS(TableOnly[Amount])",
    "set": "accepted",
    "stored": "=ROWS(TableOnly[Amount])",
    "text": "3",
    "value": 3.0
  },
  {
    "scenario": "table_only_bare_and_structured",
    "formula": "=TableOnly[Amount]",
    "set": "accepted",
    "stored": "=TableOnly[Amount]",
    "text": "10",
    "value": 10.0
  },
  {
    "scenario": "structured_formula_rename_rewrite",
    "before_formula": "=SUM(TableA[Amount])",
    "before_value": 60.0,
    "after_table_rename_formula": "=SUM(TableB[Amount])",
    "after_table_rename_value": 60.0,
    "after_column_rename_formula": "=SUM(TableB[Price])",
    "after_column_rename_value": 60.0
  },
  {
    "scenario": "sheet_defined_name_first_collision",
    "formula": "=TableSheet",
    "set": "accepted",
    "stored": "=TableSheet",
    "text": "88",
    "value": 88.0
  },
  {
    "scenario": "sheet_defined_name_first_collision",
    "formula": "=SUM(TableSheet[Amount])",
    "set": "rejected",
    "error": "0x800A03EC"
  },
  {
    "scenario": "sheet_defined_name_first_collision",
    "formula": "=TableSheet[Amount]",
    "set": "rejected",
    "error": "0x800A03EC"
  },
  {
    "scenario": "table_first_sheet_name_add",
    "set": "rejected",
    "error": "A table with that name already exists. Select a different name."
  },
  {
    "scenario": "udf_control",
    "formula": "=PlainUdf()",
    "set": "accepted",
    "stored": "=PlainUdf()",
    "text": "123",
    "value": 123.0
  },
  {
    "scenario": "table_udf_collision",
    "formula": "=TableUdf()",
    "set": "accepted",
    "stored": "=TableUdf()",
    "text": "#REF!",
    "value": -2146826265
  },
  {
    "scenario": "table_udf_collision",
    "formula": "=TableUdf",
    "set": "accepted",
    "stored": "=TableUdf",
    "text": "10",
    "value": 10.0
  },
  {
    "scenario": "table_udf_collision",
    "formula": "=SUM(TableUdf[Amount])",
    "set": "accepted",
    "stored": "=SUM(TableUdf[Amount])",
    "text": "30",
    "value": 30.0
  }
]
```

## W074 Interpretation

For W074 table-adjacent closure:

1. Excel table names are not interchangeable with workbook or sheet defined
   names. Creation order matters for collisions, and structured syntax can be
   rejected in defined-name-first collision states.
2. Bare table-name and table-name-call observations are table-specific oracle
   rows, not generic host-name semantics.
3. Table and column rename behavior is structure/table-context mutation
   evidence: Excel rewrites stored structured-reference formulas rather than
   preserving stale table/column tokens.
4. OxFml should keep local runtime/replay invalidation generic: table
   descriptor, column, enclosing-table, and caller-row facts participate in
   prepared identity through the table-context packet; OxCalc remains
   responsible for host-owned table objects and dependency lowering.
