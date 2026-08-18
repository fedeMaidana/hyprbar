use hyprbar::bar::notifications::{HistoryNote, HyprnotifyState, NoteUrgency, age};

#[test]
fn age_formats_compact_spanish_labels() {
    assert_eq!(age(0), "ahora");
    assert_eq!(age(59), "ahora");
    assert_eq!(age(60), "1m");
    assert_eq!(age(3 * 60 + 30), "3m");
    assert_eq!(age(3600), "1h");
    assert_eq!(age(5 * 3600), "5h");
    assert_eq!(age(86400), "1d");
    assert_eq!(age(3 * 86400 + 7200), "3d");
}

#[test]
fn state_contract_deserializes_with_missing_fields() {
    let state: HyprnotifyState = serde_json::from_str("{}").expect("contrato vacío");

    assert!(!state.dnd);
    assert_eq!(state.history_count, 0);

    let state: HyprnotifyState = serde_json::from_str(r#"{"dnd": true, "history_count": 4}"#).expect("contrato completo");

    assert!(state.dnd);
    assert_eq!(state.history_count, 4);
}

#[test]
fn history_note_deserializes_with_defaults_and_urgency() {
    let note: HistoryNote = serde_json::from_str("{}").expect("nota vacía");

    assert_eq!(note.urgency, NoteUrgency::Normal);
    assert!(note.summary.is_empty());

    let json = r#"{
        "app_name": "mail",
        "summary": "Nuevo correo",
        "body": "hola\nsegunda línea",
        "urgency": "Critical",
        "closed_at_unix": 123
    }"#;

    let note: HistoryNote = serde_json::from_str(json).expect("nota completa");

    assert_eq!(note.app_name, "mail");
    assert_eq!(note.urgency, NoteUrgency::Critical);
    assert_eq!(note.closed_at_unix, 123);
}

#[test]
fn malformed_history_is_an_error_not_a_panic() {
    let result: Result<Vec<HistoryNote>, _> = serde_json::from_str("{ esto no es json }");

    assert!(result.is_err());
}
