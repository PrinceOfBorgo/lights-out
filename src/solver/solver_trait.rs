use crate::data::{SolverState, SolvingError};
use std::sync::atomic::AtomicBool;

pub trait Solver {
    fn solve(&self, data: &mut SolverState, cancel: &AtomicBool) -> Result<(), SolvingError>;
}
