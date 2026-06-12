use hyprbar::bar::date::{GRID_CELLS, days_in_month, monday_offset, month_grid, shift_month};

#[test]
fn shifts_month_forward_across_year_boundary() {
    assert_eq!(shift_month(2026, 11, 3), (2027, 2));
}

#[test]
fn shifts_month_backward_across_year_boundary() {
    assert_eq!(shift_month(2026, 2, -3), (2025, 11));
}

#[test]
fn shift_month_with_zero_offset_is_identity() {
    assert_eq!(shift_month(2026, 6, 0), (2026, 6));
}

#[test]
fn computes_days_in_month() {
    assert_eq!(days_in_month(2026, 6), 30);
    assert_eq!(days_in_month(2026, 7), 31);
    assert_eq!(days_in_month(2026, 2), 28);
    assert_eq!(days_in_month(2024, 2), 29);
}

#[test]
fn computes_monday_offset() {
    assert_eq!(monday_offset(2026, 6), 0);
    assert_eq!(monday_offset(2026, 1), 3);
}

#[test]
fn builds_month_grid_for_june_2026() {
    let grid = month_grid(2026, 6);

    assert_eq!(grid.len(), GRID_CELLS);
    assert_eq!(grid[0], Some(1));
    assert_eq!(grid[29], Some(30));
    assert_eq!(grid[30], None);
}

#[test]
fn builds_month_grid_with_leading_blanks() {
    let grid = month_grid(2026, 1);

    assert_eq!(grid[0], None);
    assert_eq!(grid[2], None);
    assert_eq!(grid[3], Some(1));
    assert_eq!(grid[33], Some(31));
}
