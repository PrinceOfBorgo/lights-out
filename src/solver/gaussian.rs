use super::Solver;
use crate::data::{GridCoord, SolverState, SolvingError};
use itertools::Itertools;
use std::sync::atomic::{AtomicBool, Ordering};

/// Above this many valid solutions, the first one is used instead of searching for the fewest clicks
const MAX_CANDIDATES: usize = 4096;

pub struct Gaussian;

impl Solver for Gaussian {
    fn solve(&self, data: &mut SolverState, cancel: &AtomicBool) -> Result<(), SolvingError> {
        let params = &mut data.params;
        params.solution.error = true;

        let k = params.states as i64;
        let objective = params.objective as i64;
        // Chase along the longer side so the system has min(rows, columns) unknowns
        let transpose = params.rows < params.columns;
        let (rows, cols) = if transpose {
            (params.columns, params.rows)
        } else {
            (params.rows, params.columns)
        };
        let coord = |i: usize| {
            let (r, c) = (i / cols, i % cols);
            if transpose {
                GridCoord { row: c, col: r }
            } else {
                GridCoord { row: r, col: c }
            }
        };
        // Puzzle grid as a flat vector
        let puzzle: Vec<i64> = (0..rows * cols)
            .map(|i| params.puzzle[coord(i)].state as i64)
            .collect();

        let Some(clicks) = chase(&puzzle, rows, cols, objective, k, cancel) else {
            return Ok(());
        };

        // Remaining equations: the last row must reach the objective
        let mut matrix = vec![vec![0; cols]; cols];
        let mut rhs = vec![0; cols];
        for c in 0..cols {
            let expr = partial_sum(&clicks, cols, rows - 1, c, k);
            matrix[c].copy_from_slice(&expr[..cols]);
            rhs[c] = (objective - puzzle[(rows - 1) * cols + c] - expr[cols]).rem_euclid(k);
        }

        let v = diagonalize(&mut matrix, &mut rhs, k);
        let Some(mut options) = diagonal_solutions(&matrix, &rhs, k) else {
            return Ok(());
        };

        let candidates = options
            .iter()
            .try_fold(1usize, |acc, o| acc.checked_mul(o.len()))
            .unwrap_or(usize::MAX);
        if candidates > MAX_CANDIDATES {
            options.iter_mut().for_each(|o| o.truncate(1));
        }

        let mut best: Option<(i64, Vec<i64>)> = None;
        for z in options.into_iter().multi_cartesian_product() {
            if cancel.load(Ordering::Relaxed) {
                return Ok(());
            }
            let y: Vec<i64> = v
                .iter()
                .map(|row| {
                    row.iter()
                        .zip(&z)
                        .map(|(a, b)| a * b)
                        .sum::<i64>()
                        .rem_euclid(k)
                })
                .collect();
            let solution = evaluate(&clicks, &y, k);
            let total = solution.iter().sum();
            if best.as_ref().is_none_or(|(t, _)| total < *t) {
                best = Some((total, solution));
            }
        }

        if let Some((_, solution)) = best {
            for (i, clicks) in solution.into_iter().enumerate() {
                params.solution[coord(i)].state = clicks as usize;
            }
            params.solution.error = false;
        }

        Ok(())
    }
}

/// Expresses each cell's clicks as an affine function of the first-row clicks:
/// `cols` coefficients followed by a constant, all mod `k`.
fn chase(
    puzzle: &[i64],
    rows: usize,
    cols: usize,
    objective: i64,
    k: i64,
    cancel: &AtomicBool,
) -> Option<Vec<Vec<i64>>> {
    let mut clicks = vec![vec![0; cols + 1]; rows * cols];
    for (c, expr) in clicks.iter_mut().take(cols).enumerate() {
        expr[c] = 1;
    }
    for r in 1..rows {
        if cancel.load(Ordering::Relaxed) {
            return None;
        }
        for c in 0..cols {
            // Clicks on (r, c) are the last chance to bring (r - 1, c) to the objective.
            let mut expr = partial_sum(&clicks, cols, r - 1, c, k);
            expr.iter_mut().for_each(|e| *e = (-*e).rem_euclid(k));
            expr[cols] = (expr[cols] + objective - puzzle[(r - 1) * cols + c]).rem_euclid(k);
            clicks[r * cols + c] = expr;
        }
    }
    Some(clicks)
}

/// Sum of the clicks affecting (r, c), excluding the cell below it.
fn partial_sum(clicks: &[Vec<i64>], cols: usize, r: usize, c: usize, k: i64) -> Vec<i64> {
    let mut sum = clicks[r * cols + c].clone();
    let mut add = |i: usize| {
        sum.iter_mut()
            .zip(&clicks[i])
            .for_each(|(s, e)| *s = (*s + e) % k);
    };
    if c > 0 {
        add(r * cols + c - 1);
    }
    if c + 1 < cols {
        add(r * cols + c + 1);
    }
    if r > 0 {
        add((r - 1) * cols + c);
    }
    sum
}

fn evaluate(clicks: &[Vec<i64>], y: &[i64], k: i64) -> Vec<i64> {
    let n = y.len();
    clicks
        .iter()
        .map(|e| (e[..n].iter().zip(y).map(|(a, b)| a * b).sum::<i64>() + e[n]).rem_euclid(k))
        .collect()
}

/// Diagonalizes `a` mod `k` with unimodular row and column operations. Row operations are
/// applied to `b`; the returned matrix `v` accumulates column operations, so `x = v·z`.
fn diagonalize(a: &mut [Vec<i64>], b: &mut [i64], k: i64) -> Vec<Vec<i64>> {
    let n = a.len();
    let mut v: Vec<Vec<i64>> = (0..n)
        .map(|i| (0..n).map(|j| i64::from(i == j)).collect())
        .collect();

    for t in 0..n {
        let Some((pr, pc)) = (t..n).cartesian_product(t..n).find(|&(i, j)| a[i][j] != 0) else {
            break;
        };
        a.swap(t, pr);
        b.swap(t, pr);
        a.iter_mut()
            .chain(v.iter_mut())
            .for_each(|row| row.swap(t, pc));

        // Each pass either leaves the pivot unchanged (and both lines clear) or strictly shrinks it.
        loop {
            for i in t + 1..n {
                if a[i][t] != 0 {
                    let (s, u, xg, yg) = bezout(a[t][t], a[i][t]);
                    let (top, bottom) = a.split_at_mut(i);
                    for (pt, qi) in top[t].iter_mut().zip(bottom[0].iter_mut()) {
                        let (p, q) = (*pt, *qi);
                        *pt = (s * p + u * q).rem_euclid(k);
                        *qi = (-yg * p + xg * q).rem_euclid(k);
                    }
                    let (p, q) = (b[t], b[i]);
                    b[t] = (s * p + u * q).rem_euclid(k);
                    b[i] = (-yg * p + xg * q).rem_euclid(k);
                }
            }
            for j in t + 1..n {
                if a[t][j] != 0 {
                    let (s, u, xg, yg) = bezout(a[t][t], a[t][j]);
                    for row in a.iter_mut().chain(v.iter_mut()) {
                        let (p, q) = (row[t], row[j]);
                        row[t] = (s * p + u * q).rem_euclid(k);
                        row[j] = (-yg * p + xg * q).rem_euclid(k);
                    }
                }
            }
            if (t + 1..n).all(|i| a[i][t] == 0) {
                break;
            }
        }
    }
    v
}

/// For pivot `x != 0` and entry `y`, returns `(s, u, x/g, y/g)` with `s·x + u·y = g = gcd(x, y)`.
fn bezout(x: i64, y: i64) -> (i64, i64, i64, i64) {
    if y % x == 0 {
        return (1, 0, 1, y / x);
    }
    let (g, s, u) = ext_gcd(x, y);
    (s, u, x / g, y / g)
}

fn ext_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let (g, s, t) = ext_gcd(b, a % b);
        (g, t, s - (a / b) * t)
    }
}

/// All values of each `z_i` satisfying `d_i·z_i ≡ c_i (mod k)`, or `None` if any has none.
fn diagonal_solutions(d: &[Vec<i64>], c: &[i64], k: i64) -> Option<Vec<Vec<i64>>> {
    (0..d.len())
        .map(|i| {
            let (g, _, _) = ext_gcd(d[i][i], k);
            if c[i] % g != 0 {
                return None;
            }
            let kg = k / g;
            let inv = ext_gcd((d[i][i] / g) % kg, kg).1.rem_euclid(kg);
            let base = (c[i] / g * inv).rem_euclid(kg);
            Some((0..g).map(|j| base + j * kg).collect())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::Internal;
    use rand::RngExt;

    fn random_state(rows: usize, columns: usize, states: usize, objective: usize) -> SolverState {
        let mut state = SolverState::new(rows, columns, states, objective);
        let mut rng = rand::rng();
        for row in 0..rows {
            for col in 0..columns {
                state.params.puzzle[GridCoord { row, col }].state = rng.random_range(0..states);
            }
        }
        state
    }

    fn is_solution(state: &SolverState) -> bool {
        let mut puzzle = state.params.puzzle.clone();
        for row in 0..state.params.rows {
            for col in 0..state.params.columns {
                let coord = GridCoord { row, col };
                puzzle.click_adjacent_unchecked(coord, state.params.solution[coord].state);
            }
        }
        puzzle
            .storage
            .iter()
            .all(|c| c.state == state.params.objective)
    }

    fn total_clicks(state: &SolverState) -> usize {
        state.params.solution.storage.iter().map(|c| c.state).sum()
    }

    #[test]
    fn agrees_with_brute_force() {
        let cancel = AtomicBool::new(false);
        for (rows, columns) in [(1, 1), (3, 3), (4, 4), (5, 5), (2, 6), (6, 3)] {
            for states in [2, 3, 4, 6] {
                for _ in 0..10 {
                    let objective = states - 1;
                    let mut gaussian = random_state(rows, columns, states, objective);
                    let mut brute = gaussian.clone();
                    Gaussian.solve(&mut gaussian, &cancel).unwrap();
                    Internal.solve(&mut brute, &cancel).unwrap();

                    assert_eq!(gaussian.params.solution.error, brute.params.solution.error);
                    if !gaussian.params.solution.error {
                        assert!(is_solution(&gaussian));
                        assert!(total_clicks(&gaussian) <= total_clicks(&brute));
                    }
                }
            }
        }
    }

    #[test]
    fn solves_large_randomized_puzzle() {
        let mut state = SolverState::new(30, 40, 5, 2);
        state.randomize();
        Gaussian.solve(&mut state, &AtomicBool::new(false)).unwrap();
        assert!(!state.params.solution.error);
        assert!(is_solution(&state));
    }
}
