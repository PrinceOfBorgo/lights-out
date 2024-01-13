use core::fmt::Debug;
use derivative::Derivative;
use derive_more::{Display, Error, From};
use druid::{Data, Lens};
use itertools::Itertools;
use rand::Rng;
use std::time::Instant;
use std::{io::Error, string::FromUtf8Error};
use std::{
    ops::{Index, IndexMut},
    sync::Arc,
};

use crate::settings::Solver;
use crate::solver::{self, Solver as SolverTrait};

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
        self.params.solve_time.clear();
        let time = Instant::now();

        let solver: Box<dyn SolverTrait> = match crate::SETTINGS.solver {
            Solver::Clingo { ref clingo_path } => Box::new(solver::Clingo {
                clingo_path: clingo_path.clone(),
            }),
            Solver::Internal => Box::new(solver::Internal),
            Solver::InternalPar { threads } => Box::new(solver::InternalPar { threads }),
        };

        let result = solver.solve(self);

        self.params.solve_time = format!("{:?}", time.elapsed());
        result
    }

    pub fn randomize(&mut self) {
        self.params.solve_time.clear();
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
    #[derivative(PartialEq = "ignore")]
    pub solve_time: String,
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
            solve_time: String::new(),
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
    pub(crate) storage: Arc<Vec<Cell>>,
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
                self[coord].state = (self[coord].state + n) % self.states;
            }
        }
    }

    pub(crate) fn click_adjacent_unchecked(&mut self, coord: GridCoord, n: usize) {
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

    pub(crate) fn random_clicks(&mut self) {
        let mut rng = rand::thread_rng();
        for row in 0..self.rows {
            for col in 0..self.columns {
                let n = rng.gen_range(0..self.states);
                self.click_adjacent_unchecked(GridCoord { row, col }, n);
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
