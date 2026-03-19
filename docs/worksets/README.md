# OxFml Worksets

Worksets are sequence-based execution packets for OxFml formula-language and evaluator seam work.

## Naming Convention

Sequential numbering: `W001`, `W002`, `W003`, ...

File pattern: `W<NNN>_<SLUG>.md`

Sequential numbering makes dependency ordering visible and avoids ambiguity.

## Status Vocabulary

| Status | Meaning |
|--------|---------|
| `planned` | Accepted into sequence, not yet started |
| `in_progress` | Active work underway |
| `blocked` | In-progress with active blocker (see CURRENT_BLOCKERS.md) |
| `complete` | All gate criteria met, pre-closure checklist passed, three-axis report attached |

## Claim Confidence

| Level | Meaning |
|-------|---------|
| `draft` | Initial structure, known gaps |
| `provisional` | Substantive content, pending final evidence |
| `validated` | All evidence present and verified |

## Workset Template

Each workset file must include:

```markdown
# W<NNN>: <Title>

## Purpose
<Why this workset exists and what it delivers>

## Position and Dependencies
- **Depends on**: <W-NNN references or "none">
- **Blocks**: <W-NNN references or "none">
- **Cross-repo**: <handoff dependencies if any>

## Scope
### In scope
1. <item>

### Out of scope
1. <item>

## Deliverables
1. <deliverable with verifiable criteria>

## Gate Model
### Entry gate
- <precondition>

### Exit gate
- <criteria — binary yes/no>

## Pre-Closure Verification Checklist
(Copy from OPERATIONS.md Section 7, fill in yes/no for each item)

## Status
- execution_state: planned | in_progress | blocked | complete
- scope_completeness: scope_complete | scope_partial
- target_completeness: target_complete | target_partial
- integration_completeness: integrated | partial
- open_lanes: <list or "none">
- claim_confidence: draft | provisional | validated
```

## Rules

1. Worksets are sequence/gate driven, never date driven.
2. Each workset must declare dependencies, deliverables, and gate criteria.
3. Completion requires passing the Pre-Closure Verification Checklist (OPERATIONS.md Section 7).
4. Completion requires a three-axis status report (AGENTS.md Section 3, Rule 3).
5. Completion requires the Completion Claim Self-Audit (OPERATIONS.md Section 9).
6. Claim confidence and status must be stated separately.
7. Workset-local gate closure may rely on local witness evidence when the workset scope explicitly targets an implementation-start baseline, but such closure must not be reported as if pack-grade evidence already exists.
8. If the user explicitly disables checkpointing until an AutoRun gate, do not emit intermediate checkpoint reports before that gate unless blocked.

## Current Planned Sequence

Current baseline sequence after `W001`:
1. `W002` parser, binder, and artifact core baseline
2. `W003` semantic-plan and OxFunc boundary baseline
3. `W004` FEC/F3E runtime and schema fixture baseline
4. `W005` replay and formal kickoff for core surfaces
5. `W006` formatting semantics and host-query follow-through
6. `W007` execution profiles and concurrency contract baseline
7. `W008` single-formula host and empirical oracle bootstrap
8. `W009` replay appliance adapter and witness rollout
9. `W010` witness distillation and retained fixture promotion
10. `W011` reduced witness family breadth
11. `W012` bundle normalization and pack-candidate evidence
12. `W013` parser binder breadth and incremental reuse
13. `W014` semantic breadth and OxFunc catalog expansion
14. `W015` Stage 2 runtime contention and concurrency hardening
15. `W016` checked formal artifacts and model runs
16. `W017` replay promotion toward `cap.C4.distill_valid`
17. `W018` proving host and empirical oracle expansion
18. `W019` reference breadth and formula-language closure
19. `W020` semantic catalog and callable-value breadth
20. `W021` async runtime and scheduler surface baseline
21. `W022` formal family expansion and checked clause mapping
22. `W023` replay promotion and retained witness sets
23. `W024` DNA OneCalc host policy and empirical-pack planning
24. `W025` pack-grade replay promotion baseline
25. `W026` library-context snapshot and availability taxonomy
26. `W027` callable-value and helper-transport narrowing
27. `W028` commit, publication, and topology breadth
28. `W029` runtime async and distributed consequences
29. `W030` semantic formatting and display boundary closure
30. `W031` MS-OE376 formula and formatting rule review
31. `W032` OxFunc catalog, callable transport, and provider closure
32. `W033` replay promotion toward `cap.C4.distill_valid`
33. `W034` distributed runtime and coordinator consequence boundary
34. `W035` broader formal family and concurrency model expansion
35. `W036` structured reference and table formula semantics
36. `W037` R1C1 formula channels and reference translation
37. `W038` name and external-name formula carriers
38. `W039` conditional-formatting and data-validation formula sublanguages
39. `W040` higher-order callable evidence and seam reopen

This sequence is the current planning baseline.
It may be refined, but new worksets should preserve dependency clarity rather than bypass it informally.

Supporting planning artifact:
1. `docs/worksets/NEXT_WAVE_EXECUTION_MAP.md`

Current user-authorized execution override:
1. No AutoRun override is currently active.
2. The repo is back on default checkpoint-at-gates behavior.
3. A new AutoRun override must be explicitly authorized by the user for a bounded scope.
