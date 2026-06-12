// ─── < Imports > ────────────────────────────────────────────────────

use chrono::{Datelike, NaiveDate};

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

pub fn monday_offset(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(year, month, 1)
        .map(|date| date.weekday().num_days_from_monday())
        .unwrap_or(0)
}

pub fn month_grid(year: i32, month: u32) -> [Option<u8>; GRID_CELLS] {
    let mut grid = [None; GRID_CELLS];

    let start = monday_offset(year, month) as usize;
    let days = days_in_month(year, month);

    for day in 1..=days {
        let index = start + day as usize - 1;

        if index < GRID_CELLS {
            grid[index] = Some(day as u8);
        }
    }

    grid
}
