# OPERATIONS.md — OxFml Operations

## 1. Purpose
Define daily execution rules for formula-language and evaluator seam delivery.

## 2. Operating Principles
1. Seam determinism before optimization.
2. No hidden protocol semantics: contract text and traces must match.
3. Reject semantics must be typed and replay-stable.
4. Formula-semantic formatting behavior must be evaluated in the seam path, not deferred to display-only layers.

## 3. Execution Lanes
1. Formula-language lane: grammar, parsing, bind/reference normalization.
2. Evaluator lane: `prepare -> open_session/capability_view -> execute -> commit` lifecycle.
3. Seam contract lane: commit deltas, reject taxonomy, trace schema.
4. Conformance lane: matrix updates, scenario packs, replay evidence.

## 4. Required Packs (baseline)
1. `PACK.fec.commit_atomicity`
2. `PACK.fec.reject_detail_replay`
3. `PACK.fec.overlay_lifecycle`
4. `PACK.fec.format_dependency_tokens`
5. `PACK.format.semantic_vs_display_boundary`

## 5. Cross-Repo Handoff Rule
When OxFml changes coordinator-facing semantics:
1. file a handoff packet to OxCalc,
2. include exact proposed normative text,
3. include replay/evidence links,
4. include migration impact and fallback policy.

## 6. Promotion Gate
No seam change is promoted without:
1. updated spec text,
2. updated matrix rows,
3. at least one deterministic replay artifact proving intended behavior,
4. explicit statement of impact on OxCalc coordinator logic.
