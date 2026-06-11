use hyprbar::bar::system::count_update_lines;

#[test]
fn counts_pending_update_lines() {
    let stdout = "linux 6.12.3 -> 6.12.4\nfirefox 133.0 -> 134.0\n";

    assert_eq!(count_update_lines(stdout), 2);
}

#[test]
fn ignores_blank_lines() {
    assert_eq!(count_update_lines("\n   \n"), 0);
    assert_eq!(count_update_lines(""), 0);
}
