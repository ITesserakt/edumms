use crate::ffi::{SolverEvalNextFn, SolverPrepareFn};
use crate::task::CauchyTask;
use anyhow::Error;
use libloading::{Library, Symbol};

/// Condition used to stop solver
#[derive(Debug, Copy, Clone)]
pub enum StopCondition<T> {
    /// Specifies maximum time solver can compute
    Timed { maximum: T },
}

pub trait Solver<T, N> {
    fn solve_task(
        self,
        task: &CauchyTask<T, N>,
        stop_condition: StopCondition<T>,
    ) -> Vec<(T, Box<[N]>)>;

    fn next_solution(&mut self, task: &CauchyTask<T, N>) -> (T, &[N]);
}

#[derive(Debug)]
pub struct ExternalSolver<'lib, T, N> {
    pub(crate) symbol_prepare: Symbol<'lib, SolverPrepareFn<T, N>>,
    pub(crate) symbol_next: Symbol<'lib, SolverEvalNextFn<T, N>>,
}

impl<'lib, T, N> ExternalSolver<'lib, T, N> {
    // OMG very unsafe code
    pub unsafe fn build(library: &'lib Library) -> Result<Self, Error> {
        Ok(Self {
            symbol_next: library.get(b"solver_prepare\0")?,
            symbol_prepare: library.get(b"solver_eval_next\0")?,
        })
    }
}
