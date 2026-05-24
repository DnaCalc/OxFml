# W074-CALC005 Non-Table Name/Call Oracle Evidence 2026-05-24

Scope: retained black-box Excel COM 16.0 evidence for non-table name/call
collision rows that affect the W074/CALC-005 freeze gate. This evidence covers
broader workbook/sheet defined-name, UDF, defined-name `LAMBDA`, lexical-local,
and defined-name mutation combinations. It does not introduce TreeCalc-specific
semantics and does not by itself close the final W074 freeze audit.

Excel build:
1. `Application.Version = 16.0`
2. `Application.Build = 20026`

## Setup

The probe created a new workbook with two worksheets, added workbook-scoped and
sheet-scoped defined names, and added a VBA module with deterministic UDF
controls. Formulas were assigned through COM `Formula2`, followed by full
calculation. Values below are black-box observations of stored formula text,
display text, and `Value2`.

## Observations

1. A registered VBA UDF is callable in call position, but its bare non-call
   name is not a value reference: `=UdfOnly(1)` returned `101`, while
   `=UdfOnly` returned `#NAME?`.
2. A workbook defined-name `LAMBDA` with the same name as a registered UDF won
   the observed collision in call position and non-call position:
   `=UdfWithLambda(1)` returned `31`; bare `=UdfWithLambda` returned `#CALC!`.
3. A sheet-scoped scalar defined name with the same name as a registered UDF won
   on the sheet where it was scoped: `=UdfSheet(1)` returned `#VALUE!`, and bare
   `=UdfSheet` returned `55`. On the other sheet, the UDF was callable and the
   bare name returned `#NAME?`.
4. When workbook and sheet defined-name `LAMBDA` values had the same display
   name, sheet scope won on the sheet where it was scoped and workbook scope won
   elsewhere. Both bare references to the lambda-valued name returned `#CALC!`.
5. A sheet-scoped scalar defined name hid a workbook defined-name `LAMBDA` on
   the scoped sheet. The call position returned `#VALUE!`, the bare position
   returned the scalar value, and the workbook `LAMBDA` remained callable from
   the other sheet.
6. Built-in `SUM` won call position even with a workbook defined-name `LAMBDA`
   named `SUM`. Bare `=SUM` resolved to the lambda-valued defined name and
   returned `#CALC!`.
7. Reclassifying a workbook defined name from scalar to `LAMBDA`, then deleting
   it, changed existing formulas without changing their formula text:
   `=Morph(1)` moved from `#VALUE!` to `41` to `#NAME?`; bare `=Morph` moved
   from `5` to `#CALC!` to `#NAME?`.
8. Lexical `LET` locals won against external UDF and workbook defined-name
   `LAMBDA` candidates for the tested identifier. A lexical scalar local
   returned `9` in non-call position and `#VALUE!` in call position; a lexical
   callable local returned `3` in call position.
9. A lexical callable local or `LAMBDA` parameter named `SUM` did not override
   the built-in in call position. `SUM(2)` returned `2` through the built-in,
   while bare references to the lexical callable returned `#CALC!`.

Observed output:

```json
{
  "excel_version": "16.0",
  "excel_build": 20026.0,
  "observations": [
    {
      "scenario": "vba_setup",
      "set": "accepted"
    },
    {
      "scenario": "udf_only_control",
      "address": "A1",
      "formula": "=UdfOnly(1)",
      "set": "accepted",
      "stored": "=UdfOnly(1)",
      "text": "101",
      "value": 101.0
    },
    {
      "scenario": "udf_only_control",
      "address": "A2",
      "formula": "=UdfOnly",
      "set": "accepted",
      "stored": "=UdfOnly",
      "text": "#NAME?",
      "value": -2146826259
    },
    {
      "scenario": "udf_workbook_scalar_name_collision",
      "address": "A3",
      "formula": "=UdfWithName(1)",
      "set": "accepted",
      "stored": "=UdfWithName(1)",
      "text": "#VALUE!",
      "value": -2146826273
    },
    {
      "scenario": "udf_workbook_scalar_name_collision",
      "address": "A4",
      "formula": "=UdfWithName",
      "set": "accepted",
      "stored": "=UdfWithName",
      "text": "77",
      "value": 77.0
    },
    {
      "scenario": "udf_workbook_lambda_name_collision",
      "address": "A5",
      "formula": "=UdfWithLambda(1)",
      "set": "accepted",
      "stored": "=UdfWithLambda(1)",
      "text": "31",
      "value": 31.0
    },
    {
      "scenario": "udf_workbook_lambda_name_collision",
      "address": "A6",
      "formula": "=UdfWithLambda",
      "set": "accepted",
      "stored": "=UdfWithLambda",
      "text": "#CALC!",
      "value": -2146826238
    },
    {
      "scenario": "udf_sheet_scalar_name_collision_same_sheet",
      "address": "A7",
      "formula": "=UdfSheet(1)",
      "set": "accepted",
      "stored": "=UdfSheet(1)",
      "text": "#VALUE!",
      "value": -2146826273
    },
    {
      "scenario": "udf_sheet_scalar_name_collision_same_sheet",
      "address": "A8",
      "formula": "=UdfSheet",
      "set": "accepted",
      "stored": "=UdfSheet",
      "text": "55",
      "value": 55.0
    },
    {
      "scenario": "udf_sheet_scalar_name_collision_other_sheet",
      "address": "A1",
      "formula": "=UdfSheet(1)",
      "set": "accepted",
      "stored": "=UdfSheet(1)",
      "text": "401",
      "value": 401.0
    },
    {
      "scenario": "udf_sheet_scalar_name_collision_other_sheet",
      "address": "A2",
      "formula": "=UdfSheet",
      "set": "accepted",
      "stored": "=UdfSheet",
      "text": "#NAME?",
      "value": -2146826259
    },
    {
      "scenario": "workbook_sheet_lambda_same_name_same_sheet",
      "address": "A9",
      "formula": "=ScopeLambda(1)",
      "set": "accepted",
      "stored": "=ScopeLambda(1)",
      "text": "21",
      "value": 21.0
    },
    {
      "scenario": "workbook_sheet_lambda_same_name_same_sheet",
      "address": "A10",
      "formula": "=ScopeLambda",
      "set": "accepted",
      "stored": "=ScopeLambda",
      "text": "#CALC!",
      "value": -2146826238
    },
    {
      "scenario": "workbook_sheet_lambda_same_name_other_sheet",
      "address": "A3",
      "formula": "=ScopeLambda(1)",
      "set": "accepted",
      "stored": "=ScopeLambda(1)",
      "text": "2",
      "value": 2.0
    },
    {
      "scenario": "workbook_sheet_lambda_same_name_other_sheet",
      "address": "A4",
      "formula": "=ScopeLambda",
      "set": "accepted",
      "stored": "=ScopeLambda",
      "text": "#CALC!",
      "value": -2146826238
    },
    {
      "scenario": "sheet_scalar_over_workbook_lambda_same_sheet",
      "address": "A11",
      "formula": "=MixedName(1)",
      "set": "accepted",
      "stored": "=MixedName(1)",
      "text": "#VALUE!",
      "value": -2146826273
    },
    {
      "scenario": "sheet_scalar_over_workbook_lambda_same_sheet",
      "address": "A12",
      "formula": "=MixedName",
      "set": "accepted",
      "stored": "=MixedName",
      "text": "7",
      "value": 7.0
    },
    {
      "scenario": "sheet_scalar_over_workbook_lambda_other_sheet",
      "address": "A5",
      "formula": "=MixedName(1)",
      "set": "accepted",
      "stored": "=MixedName(1)",
      "text": "4",
      "value": 4.0
    },
    {
      "scenario": "sheet_scalar_over_workbook_lambda_other_sheet",
      "address": "A6",
      "formula": "=MixedName",
      "set": "accepted",
      "stored": "=MixedName",
      "text": "#CALC!",
      "value": -2146826238
    },
    {
      "scenario": "builtin_sum_workbook_lambda_collision",
      "address": "B1",
      "formula": "=SUM(1,2)",
      "set": "accepted",
      "stored": "=SUM(1,2)",
      "text": "3",
      "value": 3.0
    },
    {
      "scenario": "builtin_sum_workbook_lambda_collision",
      "address": "B2",
      "formula": "=SUM",
      "set": "accepted",
      "stored": "=SUM",
      "text": "#CALC!",
      "value": -2146826238
    },
    {
      "scenario": "defined_name_kind_mutation_before_scalar",
      "address": "B3",
      "formula": "=Morph(1)",
      "set": "accepted",
      "stored": "=Morph(1)",
      "text": "#VALUE!",
      "value": -2146826273
    },
    {
      "scenario": "defined_name_kind_mutation_before_scalar",
      "address": "B4",
      "formula": "=Morph",
      "set": "accepted",
      "stored": "=Morph",
      "text": "5",
      "value": 5.0
    },
    {
      "scenario": "defined_name_kind_mutation_after_lambda",
      "address": "B3",
      "stored": "=Morph(1)",
      "text": "41",
      "value": 41.0
    },
    {
      "scenario": "defined_name_kind_mutation_after_lambda",
      "address": "B4",
      "stored": "=Morph",
      "text": "#CALC!",
      "value": -2146826238
    },
    {
      "scenario": "defined_name_kind_mutation_after_delete",
      "address": "B3",
      "stored": "=Morph(1)",
      "text": "#NAME?",
      "value": -2146826259
    },
    {
      "scenario": "defined_name_kind_mutation_after_delete",
      "address": "B4",
      "stored": "=Morph",
      "text": "#NAME?",
      "value": -2146826259
    },
    {
      "scenario": "lexical_scalar_over_external_candidates",
      "address": "B5",
      "formula": "=LET(LexScalar,9,LexScalar)",
      "set": "accepted",
      "stored": "=LET(LexScalar,9,LexScalar)",
      "text": "9",
      "value": 9.0
    },
    {
      "scenario": "lexical_scalar_call_against_external_candidates",
      "address": "B6",
      "formula": "=LET(LexScalar,9,LexScalar(1))",
      "set": "accepted",
      "stored": "=LET(LexScalar,9,LexScalar(1))",
      "text": "#VALUE!",
      "value": -2146826273
    },
    {
      "scenario": "lexical_callable_over_external_candidates",
      "address": "B7",
      "formula": "=LET(LexScalar,LAMBDA(x,x+2),LexScalar(1))",
      "set": "accepted",
      "stored": "=LET(LexScalar,LAMBDA(x,x+2),LexScalar(1))",
      "text": "3",
      "value": 3.0
    },
    {
      "scenario": "lexical_callable_named_sum_vs_builtin",
      "address": "A1",
      "formula": "=LET(SUM,LAMBDA(x,x+10),SUM(2))",
      "set": "accepted",
      "stored": "=LET(SUM,LAMBDA(x,x+10),SUM(2))",
      "text": "2",
      "value": 2.0
    },
    {
      "scenario": "lexical_callable_named_sum_bare",
      "address": "A2",
      "formula": "=LET(SUM,LAMBDA(x,x+10),SUM)",
      "set": "accepted",
      "stored": "=LET(SUM,LAMBDA(x,x+10),SUM)",
      "text": "#CALC!",
      "value": -2146826238
    },
    {
      "scenario": "lambda_param_callable_named_sum_vs_builtin",
      "address": "A3",
      "formula": "=LAMBDA(SUM,SUM(2))(LAMBDA(x,x+10))",
      "set": "accepted",
      "stored": "=LAMBDA(SUM,SUM(2))(LAMBDA(x,x+10))",
      "text": "2",
      "value": 2.0
    },
    {
      "scenario": "lambda_param_callable_named_sum_bare",
      "address": "A4",
      "formula": "=LAMBDA(SUM,SUM)(LAMBDA(x,x+10))",
      "set": "accepted",
      "stored": "=LAMBDA(SUM,SUM)(LAMBDA(x,x+10))",
      "text": "#CALC!",
      "value": -2146826238
    }
  ]
}
```

## W074 Interpretation

For the final W074 freeze audit:

1. call and non-call positions must remain distinct;
2. built-ins can win call position while a same-named defined name wins
   non-call position;
3. registered UDFs are callable through the registry, but bare UDF names are
   not value references unless another name object wins the bare position;
4. workbook/sheet scope rules apply to defined-name `LAMBDA` values in both
   call and non-call positions;
5. a non-call bare reference to a lambda-valued defined name produces `#CALC!`
   in the observed rows, while call position invokes it;
6. defined-name kind changes and deletes are name-world mutation inputs for
   prepared/cache identity;
7. lexical locals remain OxFml-owned and are not host namespace entries; a
   lexical callable local can win call position against external candidates,
   while a lexical scalar local in call position produces the observed scalar
   non-callable error;
8. built-in function names keep their call-callee frontier even when a lexical
   callable local with the same display name is visible.

TreeCalc host names and lambda-valued host nodes may continue to map to the
closest defined-name and defined-name-`LAMBDA` lanes, but product admission
still requires the final W074 freeze audit and OxCalc handoff.
