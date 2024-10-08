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

pub type SolverPrepareFn<T, N> = unsafe extern "C-unwind" fn(FFICauchyTask<T, N>);
pub type SolverEvalNextFn<T, N> =
    unsafe extern "C-unwind" fn(FFICauchyTask<T, N>, out_time: *mut T) -> *const N;

impl<T, N> From<&CauchyTask<T, N>> for FFICauchyTask<'_, T, N>
where
    T: Clone,
{
    fn from(value: &CauchyTask<T, N>) -> Self {
        Self {
            size: value.size,
            initial_time: value.initial_time.clone(),
            // Lifetime '_ ensures that this pointers will be alive
            derivatives: value.derivatives.as_ptr(),
            initial_conditions: value.initial_conditions.as_ptr(),
            _phantom: PhantomData,
        }
    }
}

impl<T, N> Solver<T, N> for ExternalSolver<T, N>
where
    T: Clone + PartialOrd,
    N: Clone,
{
    fn solve_task(
        self,
        task: &CauchyTask<T, N>,
        stop_condition: StopCondition<T>,
    ) -> Vec<(T, Box<[N]>)> {
        let ffi: FFICauchyTask<T, N> = task.into();
        self.with_symbol_prepare(|s| unsafe { s(ffi.clone()) });

        once((task.initial_time.clone(), task.initial_conditions.clone()))
            .chain(repeat_with(move || {
                let mut t = MaybeUninit::uninit();
                let xs = self.with_symbol_next(|s| unsafe { s(ffi.clone(), t.as_mut_ptr()) });
                unsafe {
                    (
                        t.assume_init(),
                        Box::<[N]>::from(slice::from_raw_parts(xs, task.size)),
                    )
                }
            }))
            .take_while(|(t, _)| match &stop_condition {
                StopCondition::Timed { maximum } => t <= maximum,
            })
            .collect()
    }

    fn next_solution(&mut self, task: &CauchyTask<T, N>) -> (T, &[N]) {
        let ffi = task.into();
        let mut t = MaybeUninit::uninit();
        let xs = self.with_symbol_next(|s| unsafe { s(ffi, t.as_mut_ptr()) });
        unsafe { (t.assume_init(), slice::from_raw_parts(xs, task.size)) }
    }
}
