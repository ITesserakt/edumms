use std::ffi::c_void;
use std::fmt::Debug;
use std::slice;
use num_traits::Float;

pub struct CauchyTask<T, N> {
    pub(crate) size: usize,
    pub(crate) initial_conditions: Box<[N]>,
    pub(crate) initial_time: T,
    pub(crate) derivatives: Box<[Function<T, N>]>,
}

#[repr(C)]
pub struct Function<T, N> {
    state_pointer: *mut c_void,
    fn_pointer: extern "C" fn(*const c_void, T, *const N) -> N,
    destructor: extern "C" fn(*mut c_void)
}

impl<T, N> Function<T, N> {
    pub fn new<F, const S: usize>(f: F) -> Self
    where
        F: Fn(T, &[N; S]) -> N + 'static,
        N: Copy,
        T: Float
    {
        extern "C" fn call_closure<F, T, N, const S: usize>(state: *const c_void, time: T, inputs: *const N) -> N
        where
            F: Fn(T, &[N; S]) -> N + 'static
        {
            // SAFETY: state pointer is managed by only this struct, thus never be null
            let state = unsafe { (state as *const F).as_ref() }.unwrap();
            let inputs = unsafe { slice::from_raw_parts(inputs, S) };
            let inputs = inputs.first_chunk::<S>().expect("Size of an input array should be equal to degree");
            state(time, inputs)
        }
        
        extern "C" fn call_destructor<F, T, N, const S: usize>(state: *mut c_void)
        where
            F: Fn(T, &[N; S]) -> N + 'static
        {
            // SAFETY: state pointer is managed by only this struct, thus never be null
            let _ = unsafe { Box::from_raw(state) };
        }

        Self {
            state_pointer: Box::into_raw(Box::new(f)) as *mut _,
            fn_pointer: call_closure::<F, T, N, S>,
            destructor: call_destructor::<F, T, N, S>
        }
    }
    
    pub fn eval(&self, time: T, input: &[N]) -> N {
        (self.fn_pointer)(self.state_pointer, time, input.as_ptr())
    }
}

impl<T, N> Drop for Function<T, N> {
    fn drop(&mut self) {
        (self.destructor)(self.state_pointer)
    }
}

impl<T, N> CauchyTask<T, N> {
    pub fn new<const S: usize>(
        derivatives: [Function<T, N>; S],
        initial_time: T,
        initial_conditions: [N; S],
    ) -> Self
    where
        N: Copy + PartialEq + Debug + 'static,
    {
        Self {
            size: S,
            derivatives: Box::new(derivatives),
            initial_conditions: Box::new(initial_conditions),
            initial_time
        }
    }
}

pub fn f<T, N, const S: usize>(value: impl Fn(T, &[N; S]) -> N + 'static) -> Function<T, N>
where
    N: Copy,
    T: Float
{
    Function::new(value)
}
