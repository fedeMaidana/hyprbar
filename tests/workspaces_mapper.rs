use hyprbar::bar::workspaces::parse_workspace_data;

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

#[test]
fn ignores_workspace_ids_outside_i32_range() {
    let workspaces_json = r#"
        [
            { "id": 1 },
            { "id": 999999999999 }
        ]
    "#;

    let active_json = r#"{ "id": 1 }"#;

    let data = parse_workspace_data(workspaces_json, active_json).unwrap();

    assert_eq!(data.existing, vec![1]);
    assert_eq!(data.active_id, 1);
}
