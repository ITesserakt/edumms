use crate::ffi::{SolverEvalNextFn, SolverPrepareFn};
use crate::task::CauchyTask;
use anyhow::Error;
use libloading::{Library, Symbol};
use ouroboros::self_referencing;
use std::ffi::OsStr;

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

#[self_referencing]
pub struct ExternalSolver<T: 'static, N: 'static> {
    library: Library,
    #[borrows(library)]
    #[covariant]
    pub(crate) symbol_prepare: Symbol<'this, SolverPrepareFn<T, N>>,
    #[borrows(library)]
    #[covariant]
    pub(crate) symbol_next: Symbol<'this, SolverEvalNextFn<T, N>>,
}

impl<T, N> ExternalSolver<T, N> {
    // OMG very unsafe code
    pub unsafe fn build(external_library_path: impl AsRef<OsStr>) -> Result<Self, Error> {
        let this = Self::try_new(
            Library::new(external_library_path)?,
            |lib| lib.get(b"solver_prepare\0"),
            |lib| lib.get(b"solver_eval_next\0"),
        )?;
        Ok(this)
    }
}
