use hyprbar::bar::date::{GRID_CELLS, days_in_month, month_grid, shift_month, sunday_offset, week_number};

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
fn computes_sunday_offset() {
    // Junio 2026 empieza lunes; enero 2026, jueves.
    assert_eq!(sunday_offset(2026, 6), 1);
    assert_eq!(sunday_offset(2026, 1), 4);
    // Agosto 2026 empieza sábado: la primera fila tiene 6 huecos.
    assert_eq!(sunday_offset(2026, 8), 6);
}

#[test]
fn builds_month_grid_for_june_2026() {
    let grid = month_grid(2026, 6);

    assert_eq!(grid.len(), GRID_CELLS);
    assert_eq!(grid[0], None);
    assert_eq!(grid[1], Some(1));
    assert_eq!(grid[30], Some(30));
    assert_eq!(grid[31], None);
}

#[test]
fn builds_month_grid_with_leading_blanks() {
    let grid = month_grid(2026, 1);

    assert_eq!(grid[0], None);
    assert_eq!(grid[3], None);
    assert_eq!(grid[4], Some(1));
    assert_eq!(grid[34], Some(31));
}

#[test]
fn computes_iso_week_numbers_per_row() {
    // Agosto 2026, como en el mockup: filas 31..=36.
    assert_eq!(week_number(2026, 8, 0), Some(31));
    assert_eq!(week_number(2026, 8, 3), Some(34));
    assert_eq!(week_number(2026, 8, 5), Some(36));
}

#[test]
fn week_number_anchors_on_the_closing_saturday() {
    // Enero 2026 arranca jueves: la primera fila cierra el sábado 3,
    // que es semana ISO 1 (el domingo 28/12 pertenece a la 52 de 2025).
    assert_eq!(week_number(2026, 1, 0), Some(1));
}
