use crate::assert_is_object_safe;
use crate::ffi::{SolverEvalNextFn, SolverPrepareFn};
use crate::task::CauchyTask;
use anyhow::Error;
use libloading::{Library, Symbol};
use num_traits::Float;
use ouroboros::self_referencing;
use std::ffi::OsStr;
use std::iter::{once, repeat_with};

pub enum StopCondition<T> {
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

assert_is_object_safe!(Solver<f64, f64>);

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

pub struct EulerSolver<T> {
    h: T,
    last_solution: Box<[T]>,
    current_time: T,
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

impl<T> EulerSolver<T>
where
    T: Float + Default,
{
    pub fn new(step: T) -> Self {
        Self {
            h: step,
            last_solution: Box::new([]),
            current_time: Default::default(),
        }
    }
}

impl<T> Solver<T, T> for EulerSolver<T>
where
    T: Float,
{
    fn solve_task(
        mut self,
        task: &CauchyTask<T, T>,
        stop_condition: StopCondition<T>,
    ) -> Vec<(T, Box<[T]>)> {
        self.last_solution = task.initial_conditions.clone();
        self.current_time = task.initial_time;

        once((self.current_time, self.last_solution.clone()))
            .chain(repeat_with(|| {
                let (t, xs) = self.next_solution(task);
                (t, Box::<[T]>::from(xs))
            }))
            .take_while(|(t, _)| match &stop_condition {
                StopCondition::Timed { maximum } => t <= maximum,
            })
            .collect()
    }

    fn next_solution(&mut self, task: &CauchyTask<T, T>) -> (T, &[T]) {
        let yi = self
            .last_solution
            .iter()
            .zip(&task.derivatives)
            .map(|(&y_prev, f)| y_prev + self.h * f.eval(self.current_time, &self.last_solution))
            .collect();
        let xi = self.current_time + self.h;

        self.last_solution = yi;
        self.current_time = xi;

        (self.current_time, &self.last_solution)
    }
}
