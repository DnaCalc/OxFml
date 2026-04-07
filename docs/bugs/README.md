# OxFml Bug Tracking

This directory holds the canonical local bug-tracking scaffolding for OxFml.

It separates:
1. **bug reports**: every incoming report or observed defect record,
2. **bug streams**: the canonical known-bug lane that owns investigation, fix, root-cause analysis, and closure.

## Identity Model

### Bug reports
Individual reports use:
- `BUGREP-FML-NNN`

A report captures:
1. who reported the problem,
2. what exact repo ref it was reported against,
3. reproduction information,
4. the initial observed symptom,
5. the canonical bug stream linkage once triaged.

### Bug streams
Canonical known-bug lanes use:
- `BUG-FML-NNN`

A stream captures:
1. the canonical problem statement,
2. exact affected refs,
3. reproduction status,
4. ownership classification,
5. root-cause analysis,
6. introduced/fixed refs where known,
7. similar-risk families and follow-up checks,
8. links to all known reports about the same defect family.

## Files

1. `BUG_REPORT_REGISTER.csv`
   - one row per incoming report
2. `BUG_STREAM_REGISTER.csv`
   - one row per canonical bug stream
3. `BUG_REPORT_TEMPLATE.md`
   - report note template
4. `BUG_STREAM_TEMPLATE.md`
   - canonical stream template
5. `reports/`
   - individual bug report notes
6. `streams/`
   - canonical bug stream notes

## Working Rule

1. Every incoming bug gets a `BUGREP-FML-*` record even if it is immediately recognized as a duplicate.
2. Every non-trivial unique bug gets or links to a canonical `BUG-FML-*` stream.
3. Duplicate reports are not discarded; they are linked to the canonical bug stream through the report register and report note.
4. A bug stream is not closed until:
   - the fix landed,
   - local validation passed,
   - root-cause analysis was recorded,
   - similar-risk scanning was recorded,
   - any required cross-repo handoff was filed.

## Source-Ref Rule

Every report and stream must record the exact source ref against which the defect was observed:
1. preferred: released version/tag,
2. fallback: exact git commit SHA,
3. if neither is known, record `unknown` and say why.

Current bootstrap ref:
- `2dd48c72412797f01e34d4e4b9a1146cbddcf3cd`
