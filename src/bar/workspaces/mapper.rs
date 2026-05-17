// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::{Result, anyhow};
use serde_json::Value;

use super::state::WorkspaceData;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn parse_workspace_data(workspaces_json: &str, active_json: &str) -> Result<WorkspaceData> {
    let workspaces: Value = serde_json::from_str(workspaces_json)?;
    let active: Value = serde_json::from_str(active_json)?;

    let mut existing: Vec<i32> = workspaces
        .as_array()
        .ok_or_else(|| anyhow!("workspaces no es array"))?
        .iter()
        .filter_map(|workspace| {
            let id = workspace.get("id")?.as_i64()? as i32;

            if id < 0 {
                return None;
            }

            Some(id)
        })
        .collect();

    existing.sort_unstable();

    let active_id = active
        .get("id")
        .and_then(|value| value.as_i64())
        .ok_or_else(|| anyhow!("activeworkspace sin id"))? as i32;

    Ok(WorkspaceData { existing, active_id })
}
