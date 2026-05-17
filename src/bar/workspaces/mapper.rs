use anyhow::{Result, anyhow};
use serde_json::Value;

use super::state::WorkspaceData;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_existing_and_active_workspaces() {
        let workspaces_json = r#"
            [
                { "id": 3, "name": "3" },
                { "id": 1, "name": "1" },
                { "id": 2, "name": "2" }
            ]
        "#;

        let active_json = r#"{ "id": 2, "name": "2" }"#;

        let data = parse_workspace_data(workspaces_json, active_json).unwrap();

        assert_eq!(data.existing, vec![1, 2, 3]);
        assert_eq!(data.active_id, 2);
    }

    #[test]
    fn ignores_negative_workspace_ids() {
        let workspaces_json = r#"
            [
                { "id": -99, "name": "special" },
                { "id": 1, "name": "1" }
            ]
        "#;

        let active_json = r#"{ "id": 1, "name": "1" }"#;

        let data = parse_workspace_data(workspaces_json, active_json).unwrap();

        assert_eq!(data.existing, vec![1]);
        assert_eq!(data.active_id, 1);
    }

    #[test]
    fn fails_when_workspaces_payload_is_not_an_array() {
        let workspaces_json = r#"{ "id": 1 }"#;
        let active_json = r#"{ "id": 1 }"#;

        let error = parse_workspace_data(workspaces_json, active_json).unwrap_err();

        assert!(error.to_string().contains("workspaces no es array"));
    }

    #[test]
    fn fails_when_active_workspace_has_no_id() {
        let workspaces_json = r#"[{ "id": 1 }]"#;
        let active_json = r#"{ "name": "1" }"#;

        let error = parse_workspace_data(workspaces_json, active_json).unwrap_err();

        assert!(error.to_string().contains("activeworkspace sin id"));
    }
}
