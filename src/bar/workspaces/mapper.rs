// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use super::state::WorkspaceData;

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct WorkspaceResponseItem {
    id: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ActiveWorkspaceResponse {
    id: Option<i64>,
}

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn parse_workspace_data(workspaces_json: &str, active_json: &str) -> Result<WorkspaceData> {
    let workspaces: Vec<WorkspaceResponseItem> = serde_json::from_str(workspaces_json).context("workspaces no es array")?;

    let active: ActiveWorkspaceResponse = serde_json::from_str(active_json).context("activeworkspace inválido")?;

    let mut existing: Vec<i32> = workspaces
        .into_iter()
        .filter_map(|workspace| workspace_id_to_i32(workspace.id))
        .filter(|id| *id >= 0)
        .collect();

    existing.sort_unstable();

    let active_id = workspace_id_to_i32(active.id).ok_or_else(|| anyhow!("activeworkspace sin id"))?;

    Ok(WorkspaceData { existing, active_id })
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn workspace_id_to_i32(id: Option<i64>) -> Option<i32> {
    let id = id?;

    i32::try_from(id).ok()
}
