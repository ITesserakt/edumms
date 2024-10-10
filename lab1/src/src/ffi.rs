use crate::solver::Solver;
use crate::task::{CauchyTask, Function};
use anyhow::Error;
use libloading::{Library, Symbol};
use std::iter::{once, repeat_with};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::slice;
use crate::interval::Interval;

#[repr(C)]
struct CauchyTaskRef<'a, T, N> {
    size: usize,
    derivatives: *const Function<T, N>,
    initial_conditions: *const N,
    initial_time: T,
    _phantom: PhantomData<&'a ()>,
}

pub trait CanSolve<T, N> {
    const SUFFIX: &'static [u8];
}

pub struct ExternalSolver<'lib, T, N> {
    prepare: Symbol<'lib, extern "C" fn(CauchyTaskRef<T, N>)>,
    next: Symbol<'lib, extern "C" fn(CauchyTaskRef<T, N>, *mut T) -> *const N>,
    _phantom: PhantomData<&'lib (T, N)>,
}

impl CanSolve<f32, f32> for ExternalSolver<'_, f32, f32> {
    const SUFFIX: &'static [u8] = b"f32_f32";
}

impl CanSolve<f64, f64> for ExternalSolver<'_, f64, f64> {
    const SUFFIX: &'static [u8] = b"f64_f64";
}

impl CanSolve<f64, Interval<f64>> for ExternalSolver<'_, f64, Interval<f64>> {
    const SUFFIX: &'static [u8] = b"f64_If64";
}

impl CanSolve<f64, Interval<f32>> for ExternalSolver<'_, f64, Interval<f32>> {
    const SUFFIX: &'static [u8] = b"f64_If32";
}

impl CanSolve<f32, Interval<f64>> for ExternalSolver<'_, f32, Interval<f64>> {
    const SUFFIX: &'static [u8] = b"f32_If64";
}

impl CanSolve<f32, Interval<f32>> for ExternalSolver<'_, f32, Interval<f32>> {
    const SUFFIX: &'static [u8] = b"f32_If32";
}

impl<'lib, T, N> ExternalSolver<'lib, T, N>
where
    Self: CanSolve<T, N>,
{
    pub unsafe fn build(library: &'lib Library) -> Result<Self, Error> {
        let mut buffer = vec![];
        buffer.extend_from_slice(b"solver_prepare_");
        buffer.extend_from_slice(Self::SUFFIX);
        buffer.push(0);
        let prepare = library.get(&buffer)?;
        buffer.clear();
        buffer.extend_from_slice(b"solver_eval_next_");
        buffer.extend_from_slice(Self::SUFFIX);
        buffer.push(0);

        Ok(Self {
            prepare,
            next: library.get(buffer.as_slice())?,
            _phantom: Default::default(),
        })
    }
}

impl<T, N> CauchyTask<T, N>
where
    T: Clone,
{
    fn as_ffi(&self) -> CauchyTaskRef<T, N> {
        CauchyTaskRef {
            size: self.size,
            derivatives: self.derivatives.as_ptr(),
            initial_conditions: self.initial_conditions.as_ptr(),
            initial_time: self.initial_time.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<'solver, T, N> Solver<T, N> for ExternalSolver<'solver, T, N>
where
    T: Clone,
    N: Clone,
{
    fn solve_task<const S: usize>(
        mut self,
        task: &CauchyTask<T, N>,
    ) -> impl Iterator<Item = (T, [N; S])> {
        assert_eq!(task.size, S, "Task size should be equal to given size");
        let ffi = task.as_ffi();
        (self.prepare)(ffi);

        let initial = task.initial_conditions.first_chunk().unwrap();
        once((task.initial_time.clone(), initial.clone()))
            .chain(repeat_with(move || {
                let (t, xs) = self.next_solution(task);
                let xs = xs.first_chunk().unwrap();
                (t, xs.clone())
            }))
    }

    fn next_solution(&mut self, task: &CauchyTask<T, N>) -> (T, &[N]) {
        let ffi = task.as_ffi();
        let mut time = MaybeUninit::uninit();
        let xs = (self.next)(ffi, time.as_mut_ptr());
        assert!(!xs.is_null(), "Got solution, but it is null");
        unsafe {
            (
                time.assume_init(),
                slice::from_raw_parts(xs as *const _, task.size),
            )
        }
    }
}
