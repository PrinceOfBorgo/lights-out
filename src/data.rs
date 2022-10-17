use core::fmt::Debug;
use derivative::Derivative;
use derive_more::{Display, Error, From};
use druid::{Data, Lens};
use itertools::Itertools;
use rand::Rng;
use regex::Regex;
use std::{io::Error, string::FromUtf8Error};
use std::{
    ops::{Index, IndexMut},
    process::Command,
    sync::Arc,
};

use crate::settings::Solver;

const SOLVER_PROGRAM: &str = r###"% Valid clicks number in range [0, possible cell states - 1]
clicks(0..States-1) :- states(States).
coord(1..N, 1..M) :- dim(N, M).

% Replace undefined cells with 0-state cells
cell(X, Y, 0) :- 
    coord(X, Y),
    0 { cell(X, Y, State): clicks(State), State > 0 } 0.
% Cells with same coordinates cannot have different states
:- cell(X, Y, State1), cell(X, Y, State2), State1 != State2.

% Cells with coordinates (X,Y) and (A,B) are adjacent (or the same cell)
adjacent(X, Y, A, B) :-
    |X - A| + |Y - B| <= 1,
    coord(X, Y),
    coord(A, B).

% Consider one action for each cell. Every action consist on 0 or more clicks
1 { action(X, Y, Clicks) : clicks(Clicks) } 1 :- coord(X, Y).
% Minimize the number of clicks to solve the puzzle
:~ action(X, Y, Clicks). [Clicks@1, X, Y]

% Sum the total clicks of cells adjacent to cell (X, Y)
sumClicks(X, Y, Sum) :-
    coord(X, Y),
    Sum = #sum{ Clicks, A, B : adjacent(A, B, X, Y), action(A, B, Clicks) }.

% Set resulting grid cells adding the total clicks of its adjacent cells
res(X, Y, Res) :- 
    cell(X, Y, Curr),
    Res = (Curr + Sum) \ States,    % modulo operation to wrap the result
    states(States),
    sumClicks(X, Y, Sum),
    coord(X, Y), 
    clicks(Curr).

% Solution condition: resulting grid cells must be set to objective state
:- res(X, Y, Obj),
    coord(X, Y),
    not objective(Obj).

#show action/3.
"###;

#[derive(From, Debug, Display, Error)]
pub enum SolvingError {
    Io(Error),
    FromUtf8(FromUtf8Error),
    Parsing(ParsingError),
}
#[derive(Debug, Display, Error)]
pub struct ParsingError;

#[derive(Clone, Data, Lens)]
pub struct SolverState {
    pub params: Params,
}

impl SolverState {
    pub fn new(rows: usize, columns: usize, states: usize, objective: usize) -> Self {
        Self {
            params: Params::new(rows, columns, states, objective),
        }
    }

    pub fn solve(&mut self) -> Result<(), SolvingError> {
        match &crate::SETTINGS.solver {
            Solver::Clingo { clingo_path } => self.solve_clingo(clingo_path.clone()),
            Solver::Internal => self.solve_internal(),
        }
    }

    pub fn solve_internal(&mut self) -> Result<(), SolvingError> {
        let puzzle_backup = self.params.puzzle.clone();

        let puzzle = &mut self.params.puzzle;
        let solution = &mut self.params.solution;

        let rows = self.params.rows;
        let columns = self.params.columns;
        let states = self.params.states;
        let objective = self.params.objective;

        if rows < columns {
            for first_col_clicks in (0..rows).map(|_| 0..states).multi_cartesian_product() {
                let mut curr_col_clicks = first_col_clicks;
                for col in 0..columns {
                    for row in 0..rows {
                        solution[GridCoord { row, col }].state = curr_col_clicks[row];
                        puzzle
                            .click_adjacent_unchecked(GridCoord { row, col }, curr_col_clicks[row]);
                    }
                    curr_col_clicks = (0..rows)
                        .map(|row| {
                            let left_cell_state = puzzle[GridCoord { row, col }].state;
                            (objective - left_cell_state).rem_euclid(states)
                        })
                        .collect();
                }
                if puzzle.storage.iter().all(|cell| cell.state == objective) {
                    solution.error = false;
                    *puzzle = puzzle_backup;
                    return Ok(());
                }
                *puzzle = puzzle_backup.clone();
            }
        } else {
            for first_row_clicks in (0..columns).map(|_| 0..states).multi_cartesian_product() {
                let mut curr_row_clicks = first_row_clicks;
                for row in 0..rows {
                    for col in 0..columns {
                        solution[GridCoord { row, col }].state = curr_row_clicks[col];
                        puzzle
                            .click_adjacent_unchecked(GridCoord { row, col }, curr_row_clicks[col]);
                    }
                    curr_row_clicks = (0..columns)
                        .map(|col| {
                            let top_cell_state = puzzle[GridCoord { row, col }].state;
                            (objective - top_cell_state).rem_euclid(states)
                        })
                        .collect();
                }
                if puzzle.storage.iter().all(|cell| cell.state == objective) {
                    solution.error = false;
                    *puzzle = puzzle_backup;
                    return Ok(());
                }
                *puzzle = puzzle_backup.clone();
            }
        }
        solution.error = true;
        Ok(())
    }

    pub fn solve_clingo(&mut self, clingo_path: String) -> Result<(), SolvingError> {
        std::fs::write("lights_out.lp", SOLVER_PROGRAM)?;
        std::fs::write("puzzle.lp", self.puzzle_to_string())?;

        let output = String::from_utf8(
            Command::new(clingo_path)
                .args(["lights_out.lp", "puzzle.lp", "-V0", "-q1"])
                .output()?
                .stdout,
        )?;

        std::fs::remove_file("lights_out.lp")?;
        std::fs::remove_file("puzzle.lp")?;

        let lines: Vec<&str> = output.lines().collect();
        if lines.len() == 3 {
            self.solution_from_string(lines[0])?;
            self.params.solution.error = false;
        } else {
            self.params.solution.error = true;
        }
        Ok(())
    }

    fn puzzle_to_string(&self) -> String {
        let rows = self.params.rows;
        let columns = self.params.columns;
        let states = self.params.states;
        let objective = self.params.objective;

        let mut str = format!("dim({rows},{columns}).states({states}).objective({objective}).");
        for i in 1..=rows {
            for j in 1..=columns {
                let coord = GridCoord {
                    row: i - 1,
                    col: j - 1,
                };
                let cell = self.params.puzzle[coord].state;
                if cell != 0 {
                    str += format!("cell({i},{j},{cell}).").as_str();
                }
            }
        }

        str
    }

    #[inline]
    fn solution_from_string(&mut self, str: &str) -> Result<(), ParsingError> {
        let mut str = String::from(str);
        str.retain(|c| !c.is_whitespace());

        let validation_re = Regex::new(r"^(action\(\d+,\d+,\d+\)\s*)*$").unwrap();
        if !validation_re.is_match(&str) {
            return Err(ParsingError);
        }

        let re = Regex::new(r"action\((?P<i>\d+),(?P<j>\d+),(?P<v>\d+)\)").unwrap();
        for c in re.captures_iter(&str) {
            let i = c.name("i").unwrap().as_str().parse::<usize>().unwrap();
            let j = c.name("j").unwrap().as_str().parse::<usize>().unwrap();
            let v = c.name("v").unwrap().as_str().parse::<usize>().unwrap();

            let coord = GridCoord {
                row: i - 1,
                col: j - 1,
            };
            self.params.solution[coord].state = v;
        }

        Ok(())
    }

    pub fn randomize(&mut self) {
        Arc::make_mut(&mut self.params.puzzle.storage).fill(Cell {
            state: self.params.objective,
        });
        self.params.puzzle.random_clicks();
    }
}

#[derive(Clone, Debug, Derivative, Data, Lens)]
#[derivative(PartialEq)]
pub struct Params {
    pub rows: usize,
    pub columns: usize,
    pub states: usize,
    #[derivative(PartialEq = "ignore")]
    pub objective: usize,
    #[derivative(PartialEq = "ignore")]
    pub puzzle: Grid,
    #[derivative(PartialEq = "ignore")]
    pub solution: Grid,
    #[derivative(PartialEq = "ignore")]
    pub play: bool,
}
impl Params {
    fn new(rows: usize, columns: usize, states: usize, objective: usize) -> Self {
        Self {
            rows,
            columns,
            states,
            objective,
            play: false,
            puzzle: Grid::new(rows, columns, states),
            solution: Grid::new(rows, columns, states),
        }
    }
    pub fn reset_grids(&mut self) {
        self.puzzle = Grid::new(self.rows, self.columns, self.states);
        self.solution = Grid::new(self.rows, self.columns, self.states);
    }
}

#[derive(Clone, Data)]
pub struct Grid {
    pub(crate) rows: usize,
    pub(crate) columns: usize,
    pub(crate) states: usize,
    storage: Arc<Vec<Cell>>,
    pub error: bool,
    pub play: bool,
}

impl Grid {
    pub fn new(rows: usize, columns: usize, states: usize) -> Grid {
        Grid {
            rows,
            columns,
            states,
            error: false,
            play: false,
            storage: Arc::new(vec![Cell::new(); rows * columns]),
        }
    }

    pub(crate) fn click(&mut self, coord: Option<GridCoord>, n: usize) {
        if let Some(coord) = coord {
            if self.play {
                self.click_adjacent_unchecked(coord, n);
            } else {
                self[coord].state = (self[coord].state + n) % self.states
            }
        }
    }

    fn click_adjacent_unchecked(&mut self, coord: GridCoord, n: usize) {
        let coords = self.adjacent(coord);
        coords
            .iter()
            .for_each(|pos| self[*pos].state = (self[*pos].state + n) % self.states);
    }

    fn adjacent(&mut self, coord: GridCoord) -> Vec<GridCoord> {
        let mut adj = vec![];
        for i in -1..=1 {
            for j in -1..=1 {
                if i128::abs(i) + i128::abs(j) < 2 {
                    let row_adj = coord.row as i128 + i;
                    let col_adj = coord.col as i128 + j;
                    if (0..self.rows as i128).contains(&row_adj)
                        && (0..self.columns as i128).contains(&col_adj)
                    {
                        adj.push(GridCoord {
                            row: row_adj as usize,
                            col: col_adj as usize,
                        });
                    }
                }
            }
        }
        adj
    }

    fn random_clicks(&mut self) {
        let mut rng = rand::thread_rng();
        for row in 0..self.rows {
            for col in 0..self.columns {
                self.click_adjacent_unchecked(GridCoord { row, col }, rng.gen_range(0..self.states))
            }
        }
    }
}

impl Index<GridCoord> for Grid {
    type Output = Cell;
    fn index(&self, pos: GridCoord) -> &Self::Output {
        let idx = pos.row * self.columns + pos.col;
        &self.storage[idx]
    }
}

impl IndexMut<GridCoord> for Grid {
    fn index_mut(&mut self, pos: GridCoord) -> &mut Self::Output {
        let idx = pos.row * self.columns + pos.col;
        Arc::make_mut(&mut self.storage).index_mut(idx)
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut output = String::from("\n");
        for row in &self
            .storage
            .iter()
            .map(|cell| cell.state)
            .chunks(self.columns)
        {
            output += &format!("{:?}\n", row.collect::<Vec<usize>>());
        }
        write!(f, "{}", output)
    }
}

#[derive(Clone, Copy, PartialEq, Data)]
pub(crate) struct GridCoord {
    pub(crate) row: usize,
    pub(crate) col: usize,
}

#[derive(Clone, Debug, Data)]
pub(crate) struct Cell {
    pub(crate) state: usize,
}

impl Cell {
    fn new() -> Self {
        Self { state: 0 }
    }
}
