use crate::solver::{ExternalSolver, Solver};
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
pub type SolverEvalNextFn<T, N> =
    extern "C-unwind" fn(FFICauchyTask<T, N>, out_time: *mut T) -> *const N;

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
    T: Clone + PartialOrd+ 'static,
    N: Clone,
{
    fn solve_task<const S: usize>(
        mut self,
        task: &CauchyTask<T, N>,
    ) -> impl Iterator<Item = (T, [N; S])> {
        assert_eq!(task.size, S, "Size of task should be equal to given size");
        (self.symbol_prepare)(task.as_ffi());
        let initial_conditions = task.initial_conditions.first_chunk::<S>().unwrap();

        once((task.initial_time.clone(), initial_conditions.clone())).chain(repeat_with(
            move || {
                let (t, xs) = self.next_solution(task);
                let xs_fixed_size: &[N; S] = xs.try_into().unwrap();
                (t, xs_fixed_size.clone())
            },
        ))
    }

    fn next_solution(&mut self, task: &CauchyTask<T, N>) -> (T, &[N]) {
        let ffi = task.as_ffi();
        let mut t = MaybeUninit::uninit();
        let xs = (self.symbol_next)(ffi, t.as_mut_ptr());

        assert!(!xs.is_null(), "Pointer is null");
        unsafe { (t.assume_init(), slice::from_raw_parts(xs, task.size)) }
    }
}
