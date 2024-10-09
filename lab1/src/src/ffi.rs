use crate::solver::{ExternalSolver, Solver, StopCondition};
use crate::task::{CauchyTask, Function};
use std::iter::{once, repeat_with};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::slice;

#[allow(non_camel_case_types)]
type size_t = usize;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FFICauchyTask<'a, T, N> {
    size: size_t,
    initial_time: T,
    derivatives: *const Function<T, N>,
    initial_conditions: *const N,
    _phantom: PhantomData<&'a ()>,
}

pub type SolverPrepareFn<T, N> = extern "C-unwind" fn(FFICauchyTask<T, N>);
pub type SolverEvalNextFn<T, N> = extern "C-unwind" fn(FFICauchyTask<T, N>, out_time: *mut T) -> *const N;

impl<T, N> CauchyTask<T, N> {
    pub fn as_ffi(&self) -> FFICauchyTask<T, N>
    where
        T: Clone,
    {
        FFICauchyTask {
            size: self.size,
            initial_time: self.initial_time.clone(),
            // Lifetime '_ ensures that this pointers will be alive
            derivatives: self.derivatives.as_ptr(),
            initial_conditions: self.initial_conditions.as_ptr(),
            _phantom: PhantomData,
        }
    }
}

impl<T, N> Solver<T, N> for ExternalSolver<'_, T, N>
where
    T: Clone + PartialOrd + Default,
    N: Clone,
{
    fn solve_task(
        mut self,
        task: &CauchyTask<T, N>,
        stop_condition: StopCondition<T>,
    ) -> Vec<(T, Box<[N]>)> {
        (self.symbol_prepare)(task.as_ffi());

        once((task.initial_time.clone(), task.initial_conditions.clone()))
            .chain(repeat_with(|| {
                let (t, xs) = self.next_solution(task);
                (t, Box::from(xs))
            }))
            .take_while(|(t, _)| match &stop_condition {
                // t < 0 => t > maximum and
                // t > 0 => t < maximum
                StopCondition::Timed { maximum } => (t >= &T::default() || t > maximum) && (t <= &T::default() || t < maximum),
            })
            .collect()
    }

    fn next_solution(&mut self, task: &CauchyTask<T, N>) -> (T, &[N]) {
        let ffi = task.as_ffi();
        let mut t = MaybeUninit::uninit();
        let xs = (self.symbol_next)(ffi, t.as_mut_ptr());
        unsafe { (t.assume_init(), slice::from_raw_parts(xs, task.size)) }
    }
}
