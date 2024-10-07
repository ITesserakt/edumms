use std::fmt::Debug;
use num_traits::Float;

pub struct CauchyTask<T, N> {
    pub(crate) size: usize,
    pub(crate) initial_conditions: Box<[N]>,
    pub(crate) initial_time: T,
    pub(crate) definitions: Box<[Function<T, N>]>,
}

pub struct Function<T, N> {
    pointer: Box<dyn Fn(T, &[N]) -> N>,
}

impl<T, N> Function<T, N> {
    pub fn new<F, const S: usize>(f: F) -> Self
    where
        F: Fn(T, &[N; S]) -> N + 'static,
        N: Copy,
        T: Float
    {
        Self {
            pointer: Box::new(move |t, inputs| {
                assert_eq!(inputs.len(), S);

                let inputs_slice = inputs
                    .first_chunk()
                    .expect("Size of an input array should be equal to degree");

                f(t, inputs_slice)
            }),
        }
    }
    
    pub fn eval(&self, time: T, input: &[N]) -> N {
        (self.pointer)(time, input)
    } 
}

impl<T, N> CauchyTask<T, N> {
    pub fn new<const S: usize>(
        definitions: [Function<T, N>; S],
        initial_time: T,
        initial_conditions: [N; S],
    ) -> Self
    where
        N: Copy + PartialEq + Debug + 'static,
    {
        Self {
            size: S,
            definitions: Box::new(definitions),
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
