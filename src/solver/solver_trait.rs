use crate::data::{SolverState, SolvingError};

pub trait Solver {
    fn solve(&self, data: &mut SolverState) -> Result<(), SolvingError>;
}
