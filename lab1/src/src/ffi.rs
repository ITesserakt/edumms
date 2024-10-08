use crate::solver::{ExternalSolver, Solver};
use crate::task::{CauchyTask, Function};
use std::marker::PhantomData;

#[allow(non_camel_case_types)]
type size_t = usize;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct FFICauchyTask<'a, T, N> {
    size: size_t,
    initial_time: T,
    derivatives: *const Function<T, N>,
    initial_conditions: *const N,
    _phantom: PhantomData<&'a ()>,
}

struct FFISolutionIter<'a, T, N> {
    task: &'a CauchyTask<T, N>,
}

macro_rules! define_evaluator {
    ($name:ident, $time_t:ty => $xs_t:ty) => {
        mod $name {
            extern "C" {
                pub fn solver_eval_next(
                    task: $crate::ffi::FFICauchyTask<$time_t, $xs_t>,
                    out_time: *mut $time_t,
                    out_xs: *mut $xs_t,
                );
                pub fn solver_prepare(task: $crate::ffi::FFICauchyTask<$time_t, $xs_t>);
            }
            
            impl $crate::solver::Solver<$time_t, $xs_t> for $crate::solver::ExternalSolver<$time_t, $xs_t> {
                fn solve_task(
                    self,
                    task: &$crate::task::CauchyTask<$time_t, $xs_t>,
                    stop_condition: $crate::solver::StopCondition<$time_t>,
                ) -> Vec<($time_t, Box<[$xs_t]>)> {
                    use std::iter::once;
                    
                    let ffi: $crate::ffi::FFICauchyTask<$time_t, $xs_t> = task.into();
                    unsafe { self::solver_prepare(ffi) };
            
                    once((task.initial_time, task.initial_conditions.clone()))
                        .chain($crate::ffi::FFISolutionIter { task })
                        .take_while(|(t, _)| match stop_condition {
                            $crate::solver::StopCondition::Timed { maximum } => t < &maximum,
                        })
                        .collect()
                }
            
                fn next_solution(&mut self, task: &$crate::task::CauchyTask<$time_t, $xs_t>) -> ($time_t, &[$xs_t]) {
                    use std::mem::MaybeUninit;
                    
                    let ffi: $crate::ffi::FFICauchyTask<$time_t, $xs_t> = task.into();
                    let mut time: MaybeUninit<$time_t> = MaybeUninit::uninit();
                    let mut xs: MaybeUninit<&[$xs_t]> = MaybeUninit::uninit();
                    unsafe {
                        self::solver_eval_next(ffi, time.as_mut_ptr(), xs.as_mut_ptr().cast());
                    }
                    unsafe { (time.assume_init(), xs.assume_init()) }
                }
            }
        }
    };
}

define_evaluator!(f64, std::ffi::c_double => std::ffi::c_double);

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

impl<T, N> Iterator for FFISolutionIter<'_, T, N>
where
    ExternalSolver<T, N>: Solver<T, N>,
    N: Clone,
{
    type Item = (T, Box<[N]>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut solver = ExternalSolver::new();
        let (t, xs) = solver.next_solution(self.task);
        Some((t, Box::<[N]>::from(xs)))
    }
}
