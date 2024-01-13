use super::Solver;
use crate::data::{Grid, GridCoord, SolverState, SolvingError};
use itertools::Itertools;
use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::channel,
        Arc,
    },
    thread,
};

pub struct Internal;

impl Solver for Internal {
    fn solve(&self, data: &mut SolverState) -> Result<(), SolvingError> {
        let puzzle_backup = data.params.puzzle.clone();

        let rows = data.params.rows;
        let columns = data.params.columns;
        let states = data.params.states;

        data.params.solution.error = true;
        if rows < columns {
            for first_col_clicks in (0..rows).map(|_| 0..states).multi_cartesian_product() {
                if solve_internal_by_col(data, &puzzle_backup, &first_col_clicks) {
                    break;
                }
            }
        } else {
            for first_row_clicks in (0..columns).map(|_| 0..states).multi_cartesian_product() {
                if solve_internal_by_row(data, &puzzle_backup, &first_row_clicks) {
                    break;
                }
            }
        }

        Ok(())
    }
}

pub struct InternalPar {
    pub threads: usize,
}

impl Solver for InternalPar {
    fn solve(&self, data: &mut SolverState) -> Result<(), SolvingError> {
        let threads = if self.threads == 0 {
            thread::available_parallelism()
                .unwrap_or(NonZeroUsize::new(1).unwrap())
                .get()
        } else {
            self.threads
        };

        let rows = data.params.rows;
        let columns = data.params.columns;
        let states = data.params.states;

        let (tx, rx) = channel();
        let solution_found = Arc::new(AtomicBool::new(false));
        let puzzle_backup = Arc::new(data.params.puzzle.clone());

        data.params.solution.error = true;
        if rows < columns {
            thread::scope(|scope| {
                for i in 0..threads {
                    let thread_tx = tx.clone();
                    let solution_found = solution_found.clone();

                    let mut data = data.clone();
                    let puzzle_backup = puzzle_backup.clone();
                    let first_col_clicks_iter =
                        (0..rows).map(|_| 0..states).multi_cartesian_product();

                    scope.spawn(move || {
                        for first_col_clicks in
                            first_col_clicks_iter.clone().skip(i).step_by(threads)
                        {
                            if solve_internal_by_col(&mut data, &puzzle_backup, &first_col_clicks) {
                                solution_found.store(true, Ordering::Relaxed);
                                let _ = thread_tx.send(data);
                                return;
                            } else if solution_found.load(Ordering::Relaxed) {
                                return;
                            }
                        }
                    });
                }
            })
        } else {
            thread::scope(|scope| {
                for i in 0..threads {
                    let thread_tx = tx.clone();
                    let solution_found = solution_found.clone();

                    let mut data = data.clone();
                    let puzzle_backup = puzzle_backup.clone();
                    let first_row_clicks_iter =
                        (0..columns).map(|_| 0..states).multi_cartesian_product();

                    scope.spawn(move || {
                        for first_row_clicks in
                            first_row_clicks_iter.clone().skip(i).step_by(threads)
                        {
                            if solve_internal_by_row(&mut data, &puzzle_backup, &first_row_clicks) {
                                solution_found.store(true, Ordering::Relaxed);
                                let _ = thread_tx.send(data);
                                return;
                            } else if solution_found.load(Ordering::Relaxed) {
                                return;
                            }
                        }
                    });
                }
            })
        }

        if let Ok(solved) = rx.recv() {
            *data = solved;
        }

        Ok(())
    }
}

#[inline]
fn solve_internal_by_col(
    data: &mut SolverState,
    puzzle_backup: &Grid,
    first_col_clicks: &[usize],
) -> bool {
    let puzzle = &mut data.params.puzzle;
    let solution = &mut data.params.solution;

    let rows = data.params.rows;
    let columns = data.params.columns;
    let states = data.params.states;
    let objective = data.params.objective;

    let mut curr_col_clicks = first_col_clicks.to_vec();
    for col in 0..columns {
        for row in 0..rows {
            solution[GridCoord { row, col }].state = curr_col_clicks[row];
            puzzle.click_adjacent_unchecked(GridCoord { row, col }, curr_col_clicks[row]);
        }
        curr_col_clicks = (0..rows)
            .map(|row| {
                let left_cell_state = puzzle[GridCoord { row, col }].state as isize;
                (objective as isize - left_cell_state).rem_euclid(states as isize) as usize
            })
            .collect();
    }

    if puzzle.storage.iter().all(|cell| cell.state == objective) {
        solution.error = false;
        *puzzle = puzzle_backup.clone();
        true
    } else {
        *puzzle = puzzle_backup.clone();
        false
    }
}

#[inline]
fn solve_internal_by_row(
    data: &mut SolverState,
    puzzle_backup: &Grid,
    first_row_clicks: &[usize],
) -> bool {
    let puzzle = &mut data.params.puzzle;
    let solution = &mut data.params.solution;

    let rows = data.params.rows;
    let columns = data.params.columns;
    let states = data.params.states;
    let objective = data.params.objective;

    let mut curr_row_clicks = first_row_clicks.to_vec();
    for row in 0..rows {
        for col in 0..columns {
            solution[GridCoord { row, col }].state = curr_row_clicks[col];
            puzzle.click_adjacent_unchecked(GridCoord { row, col }, curr_row_clicks[col]);
        }
        curr_row_clicks = (0..columns)
            .map(|col| {
                let top_cell_state = puzzle[GridCoord { row, col }].state as isize;
                (objective as isize - top_cell_state).rem_euclid(states as isize) as usize
            })
            .collect();
    }

    if puzzle.storage.iter().all(|cell| cell.state == objective) {
        solution.error = false;
        *puzzle = puzzle_backup.clone();
        true
    } else {
        *puzzle = puzzle_backup.clone();
        false
    }
}
