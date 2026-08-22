// ─── < Imports > ────────────────────────────────────────────────────

use chrono::{Datelike, Duration, NaiveDate};

// ─── < Constants > ────────────────────────────────────────────────────

pub const GRID_ROWS: usize = 6;
pub const GRID_COLUMNS: usize = 7;
pub const GRID_CELLS: usize = GRID_ROWS * GRID_COLUMNS;

// ─── < Public Functions > ────────────────────────────────────────────────────

pub fn shift_month(year: i32, month: u32, offset: i32) -> (i32, u32) {
    let index = year * 12 + month as i32 - 1 + offset;

    (index.div_euclid(12), index.rem_euclid(12) as u32 + 1)
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };

    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|date| date.pred_opt())
        .map(|date| date.day())
        .unwrap_or(30)
}

/// Celdas vacías antes del día 1: la semana arranca en domingo.
pub fn sunday_offset(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(year, month, 1)
        .map(|date| date.weekday().num_days_from_sunday())
        .unwrap_or(0)
}

pub fn rows_in_month(year: i32, month: u32) -> usize {
    let cells = sunday_offset(year, month) as usize + days_in_month(year, month) as usize;

    cells.div_ceil(GRID_COLUMNS).clamp(1, GRID_ROWS)
}

/// Número de semana ISO de una fila del grid. El ancla es el sábado que
/// cierra la fila: así el domingo suelto no arrastra la semana anterior.
pub fn week_number(year: i32, month: u32, row: usize) -> Option<u32> {
    let first = NaiveDate::from_ymd_opt(year, month, 1)?;
    let days_to_saturday = row as i64 * 7 + 6 - sunday_offset(year, month) as i64;
    let saturday = first.checked_add_signed(Duration::days(days_to_saturday))?;

    Some(saturday.iso_week().week())
}

pub fn month_grid(year: i32, month: u32) -> [Option<u8>; GRID_CELLS] {
    let mut grid = [None; GRID_CELLS];

    let start = sunday_offset(year, month) as usize;
    let days = days_in_month(year, month);

    for day in 1..=days {
        let index = start + day as usize - 1;

        if index < GRID_CELLS {
            grid[index] = Some(day as u8);
        }
    }

    grid
}
