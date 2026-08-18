use hyprbar::bar::weather::{Coordinates, parse_detected_location};

#[test]
fn parses_detected_location() {
    let body = r#"
        {
            "latitude": -38.0023,
            "longitude": -57.5575,
            "city": "Mar del Plata",
            "country_name": "Argentina"
        }
    "#;

    let location = parse_detected_location(body).unwrap();

    assert_eq!(location.coordinates, Coordinates::new(-38.0023, -57.5575));
    assert_eq!(location.label, Some("Mar del Plata, Argentina".to_string()));
}

#[test]
fn parses_detected_location_without_label() {
    let body = r#"
        {
            "latitude": -38.0023,
            "longitude": -57.5575
        }
    "#;

    let location = parse_detected_location(body).unwrap();

    assert_eq!(location.coordinates, Coordinates::new(-38.0023, -57.5575));
    assert_eq!(location.label, None);
}

#[test]
fn fails_when_detected_location_has_no_latitude() {
    let body = r#"
        {
            "longitude": -57.5575
        }
    "#;

    let error = parse_detected_location(body).unwrap_err();

    assert!(error.to_string().contains("no trae latitude"));
}

#[test]
fn fails_when_detected_location_has_no_longitude() {
    let body = r#"
        {
            "latitude": -38.0023
        }
    "#;

    let error = parse_detected_location(body).unwrap_err();

    assert!(error.to_string().contains("no trae longitude"));
}
