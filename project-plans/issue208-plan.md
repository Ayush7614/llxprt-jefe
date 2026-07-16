# Issue #208: Actions runs newest-first

## Goal

Workflow runs in Actions mode must appear reverse-chronological by
`created_at` (newest first), independent of GitHub API / pagination order.

## Acceptance matrix

| ID | Actor / path | Input / boundary | Observable success | Observable failure | Evidence |
| --- | --- | --- | --- | --- | --- |
| A1 | Actions list reload | API returns runs out of chronological order | List shows newest `created_at` first; selection lands on newest | N/A (pure reorder) | Reducer `runs_loaded_sorts_newest_created_at_first`; TUI `actions-mode.json` |
| A2 | Actions load-more | Page 2 appends older runs (or disordered page) | After append, full list is still newest-first; selection follows run id | Stale page ignored (existing) | Reducer `page2_append_resorts_newest_first` (+ TUI load-more in `actions-mode.json`) |
| A3 | Equal timestamps | Two runs share `created_at` | Deterministic order by `id` descending | N/A | Pure + reducer equal-timestamp tests |
| A4 | Missing timestamps | Empty `created_at` | Empty timestamps sort after dated runs; id tie-break among empties | N/A | Pure `sort_workflow_runs_newest_first_puts_empty_timestamps_last` |

## Non-goals

- Changing GitHub API query parameters or `per_page`.
- Sorting by `updated_at` or conclusion.
- Re-sorting inside the pure viewport projection on every paint.
- UI chrome / filter / job-detail behavior beyond run-list order.

## Architecture

Sort at commit time in the Actions load reducers (`RunsLoaded` /
`RunsPageLoaded`) via `sort_workflow_runs_newest_first` /
`resort_actions_runs_preserving_selection`. Projection stays order-preserving.
In-place `PaginatedList::sort_by` avoids cloning on page append.

## Vertical slices

1. **Pure comparator + unit tests** — already landed.
2. **Reducer reload + page-append sort with selection-by-id** — already landed.
3. **TUI scenario** — multi-run fixture returned oldest-first; assert newest title
   is selected first; navigate to end to trigger page-2; assert post-append order.
4. **Plan / ledger + main integration + exact-head gates** — this document.

## Scope ledger

| Change | Disposition | Maps to |
| --- | --- | --- |
| `src/actions_view.rs` comparator/sort helpers | In scope | A1–A4 |
| `src/state/actions_load_ops.rs` sort on load/page | In scope | A1–A2 |
| `src/domain/paginated_list.rs` `sort_by` | In scope | A2 (in-place) |
| `src/state/actions_tests.rs` / paginated_list tests | In scope | A1–A4 |
| `scripts/issue194-gh-shim.sh` multi-run + page-2 fixture | In scope | A1–A2 TUI |
| `dev-docs/tmux-scenarios/actions-mode.json` order asserts | In scope | A1–A2 TUI |
| `scripts/issue194-run-scenario.sh` audit expectations | In scope | A1–A2 TUI |
| `project-plans/issue208-plan.md` | In scope | delivery ledger |
| Merge `vybestack/main` | Required readiness | mainline drift |

## Review counters

- CodeRabbit: allocation concern fixed earlier; exact-head re-review still required after this push.
- OCR: two findings (pub→pub(crate); equal-timestamp reducer test) — fixed; threads to resolve after push.
- LLxprt Code review (`4709293462`): blockers addressed by this remediation commit set.

## TUI RED → GREEN (slice 3)

Do not rewrite prior implementation history. For the TUI ordering slice:

1. **RED:** with the multi-run oldest-first fixture and newest-first scenario
   asserts, temporarily disable production sort (`sort_workflow_runs_newest_first`
   no-op / skip resort). Scenario must fail because the first selected title is
   the oldest API row.
2. **GREEN:** restore sort; scenario passes; first selected title is newest;
   after End/load-more the list remains newest-first.
3. Record commands and outcomes under Verification below.

## Verification

```bash
cargo fmt --all --check
cargo test --lib actions_view sort_workflow_runs runs_loaded_sorts page2_append runs_load_paths_break
CARGO_TARGET_DIR=$PWD/target scripts/issue194-run-scenario.sh
make quick-check
# before push / merge readiness:
make ci-check
```

### TUI RED → GREEN evidence (2026-07-16)

Fixture returns page-1 runs oldest-first; scenario step asserts
`> [X] Inspectable Actions fixture` as the initially selected row.

1. **RED:** temporarily no-op'd `sort_workflow_runs_newest_first` and
   `resort_actions_runs_preserving_selection`.
   `scripts/issue194-run-scenario.sh` exited 1 with:
   `step 4 failed: expected screen to contain '> [X] Inspectable Actions fixture'`
2. **GREEN:** restored sort helpers. Same scenario exited 0:
   `ok: 35 steps` / `PASS: issue 194/208 Actions scenario (newest-first + jobs)...`
   Page-2 audit entry `actions/runs?page=2&per_page=30` present after `End`.

Do not rewrite earlier implementation commits to simulate TDD; this TUI slice
was proven with a genuine temporary RED patch that was discarded after GREEN.