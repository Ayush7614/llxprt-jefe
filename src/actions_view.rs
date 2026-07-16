//! Pure GitHub Actions run-list viewport projection.
//!
//! This iocraft-free module maps loaded runs and list geometry into a stable
//! selection-following window. Job-detail projection lives in
//! [`crate::actions_detail_view`].
//!
//! Run ordering (issue #208): callers should keep the loaded list sorted with
//! [`sort_workflow_runs_newest_first`] so navigation indices match the
//! reverse-chronological display order. The projection itself does not re-sort;
//! sorting lives next to load/page accept so paginated appends stay coherent.

use std::cmp::Ordering;

use crate::domain::{WorkflowRun, WorkflowRunConclusion, WorkflowRunStatus};
use crate::list_viewport::{ContentRows, ListViewport, RowsPerItem};

/// Newest-first ordering for workflow runs (issue #208).
///
/// Primary key is `created_at` descending. GitHub returns RFC 3339 / ISO-8601
/// timestamps, so lexicographic string compare matches chronological order for
/// the same format. Equal or missing timestamps break ties by `id` descending
/// for a deterministic stable fallback.
#[must_use]
pub(crate) fn cmp_workflow_runs_newest_first(a: &WorkflowRun, b: &WorkflowRun) -> Ordering {
    b.created_at
        .cmp(&a.created_at)
        .then_with(|| b.id.cmp(&a.id))
}

/// Sort `runs` reverse-chronologically in place (most recent `created_at` first).
pub(crate) fn sort_workflow_runs_newest_first(runs: &mut [WorkflowRun]) {
    runs.sort_by(cmp_workflow_runs_newest_first);
}

/// A single run in the projected runs list view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedRun {
    /// Absolute workflow-run index represented by this visible row.
    pub source_index: usize,
    pub id: u64,
    pub name: String,
    pub head_branch: String,
    pub head_sha: String,
    pub run_number: u32,
    pub event: String,
    pub workflow_name: String,
    pub created_at: String,
    pub updated_at: String,
    pub status: WorkflowRunStatus,
    pub conclusion: Option<WorkflowRunConclusion>,
    pub is_selected: bool,
}

/// The projected window of workflow runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionsRunListView {
    pub visible_runs: Vec<ProjectedRun>,
    pub first_visible_run_index: usize,
    pub total_runs_count: usize,
}

/// Project the full list of runs into a scroll-windowed view.
#[must_use]
pub fn project_runs_list(
    runs: &[WorkflowRun],
    selected_run_index: Option<usize>,
    list_viewport_height: usize,
) -> ActionsRunListView {
    let viewport = ListViewport::uniform(
        runs.len(),
        selected_run_index,
        ContentRows::new(list_viewport_height),
        RowsPerItem::new(1),
    );
    let first_visible_run = viewport.first_visible_item();
    let visible_slice = &runs[viewport.visible_range()];

    let visible_runs = visible_slice
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let actual_idx = first_visible_run + i;
            ProjectedRun {
                source_index: actual_idx,
                id: r.id,
                name: r.name.clone(),
                head_branch: r.head_branch.clone(),
                head_sha: r.head_sha.clone(),
                run_number: r.run_number,
                event: r.event.clone(),
                workflow_name: r.workflow_name.clone(),
                created_at: r.created_at.clone(),
                updated_at: r.updated_at.clone(),
                status: r.status,
                conclusion: r.conclusion,
                is_selected: selected_run_index == Some(actual_idx),
            }
        })
        .collect();
    ActionsRunListView {
        visible_runs,
        first_visible_run_index: first_visible_run,
        total_runs_count: runs.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(id: u64) -> WorkflowRun {
        WorkflowRun {
            id,
            name: format!("run {id}"),
            head_branch: "main".to_string(),
            head_sha: format!("sha{id}"),
            run_number: u32::try_from(id).unwrap_or_default(),
            event: "push".to_string(),
            status: WorkflowRunStatus::Completed,
            conclusion: Some(WorkflowRunConclusion::Success),
            workflow_name: "CI".to_string(),
            created_at: "time".to_string(),
            updated_at: "time".to_string(),
        }
    }

    #[test]
    fn selection_is_centered_when_possible() {
        let runs: Vec<_> = (0..10).map(run).collect();
        // Trailing-edge follow keeps the selected run at the bottom edge.
        let projection = project_runs_list(&runs, Some(5), 3);
        assert_eq!(projection.first_visible_run_index, 3);
        assert_eq!(projection.visible_runs.len(), 3);
        assert_eq!(projection.visible_runs[0].id, 3);
        assert_eq!(projection.visible_runs[1].id, 4);
        assert_eq!(projection.visible_runs[2].id, 5);
        assert!(projection.visible_runs[2].is_selected);
    }

    #[test]
    fn final_page_stays_full() {
        let runs: Vec<_> = (0..7).map(run).collect();
        let view = project_runs_list(&runs, Some(6), 5);

        assert_eq!(view.first_visible_run_index, 2);
        assert_eq!(view.visible_runs.len(), 5);
        assert!(view.visible_runs.last().is_some_and(|run| run.id == 6));
    }

    #[test]
    fn empty_and_zero_height_are_total() {
        assert!(project_runs_list(&[], None, 5).visible_runs.is_empty());
        let runs: Vec<_> = (0..3).map(run).collect();
        assert!(project_runs_list(&runs, Some(2), 0).visible_runs.is_empty());
    }

    #[test]
    fn short_list_stays_visible_from_the_top() {
        let runs: Vec<_> = (0..2).map(run).collect();
        let view = project_runs_list(&runs, Some(1), 5);

        assert_eq!(view.first_visible_run_index, 0);
        assert_eq!(view.visible_runs.len(), 2);
        assert!(view.visible_runs[1].is_selected);
    }

    #[test]
    fn stale_selection_clamps_window_without_marking_invalid_row() {
        let runs: Vec<_> = (0..3).map(run).collect();
        let view = project_runs_list(&runs, Some(99), 2);

        assert_eq!(view.first_visible_run_index, 1);
        assert!(view.visible_runs.iter().all(|run| !run.is_selected));
    }

    fn run_at(id: u64, created_at: &str) -> WorkflowRun {
        let mut r = run(id);
        r.created_at = created_at.to_string();
        r
    }

    #[test]
    fn sort_workflow_runs_newest_first_orders_by_created_at_desc() {
        let mut runs = vec![
            run_at(1, "2026-07-01T10:00:00Z"),
            run_at(2, "2026-07-03T10:00:00Z"),
            run_at(3, "2026-07-02T10:00:00Z"),
        ];
        sort_workflow_runs_newest_first(&mut runs);
        assert_eq!(runs.iter().map(|r| r.id).collect::<Vec<_>>(), vec![2, 3, 1]);
    }

    #[test]
    fn sort_workflow_runs_newest_first_breaks_ties_by_id_desc() {
        let mut runs = vec![
            run_at(1, "2026-07-01T10:00:00Z"),
            run_at(3, "2026-07-01T10:00:00Z"),
            run_at(2, "2026-07-01T10:00:00Z"),
        ];
        sort_workflow_runs_newest_first(&mut runs);
        assert_eq!(runs.iter().map(|r| r.id).collect::<Vec<_>>(), vec![3, 2, 1]);
    }

    #[test]
    fn sort_workflow_runs_newest_first_puts_empty_timestamps_last() {
        let mut runs = vec![
            run_at(1, ""),
            run_at(2, "2026-07-02T10:00:00Z"),
            run_at(3, "2026-07-01T10:00:00Z"),
        ];
        sort_workflow_runs_newest_first(&mut runs);
        assert_eq!(runs.iter().map(|r| r.id).collect::<Vec<_>>(), vec![2, 3, 1]);
    }

    #[test]
    fn projected_list_preserves_pre_sorted_newest_first_order() {
        let runs = vec![
            run_at(2, "2026-07-03T10:00:00Z"),
            run_at(3, "2026-07-02T10:00:00Z"),
            run_at(1, "2026-07-01T10:00:00Z"),
        ];
        let view = project_runs_list(&runs, Some(0), 5);
        assert_eq!(
            view.visible_runs.iter().map(|r| r.id).collect::<Vec<_>>(),
            vec![2, 3, 1]
        );
        assert!(view.visible_runs[0].is_selected);
    }
}
