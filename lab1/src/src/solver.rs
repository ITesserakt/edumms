use crate::task::CauchyTask;
use std::iter::{once, repeat_with};

pub trait Solver<T, N> {
    fn solve_task<const S: usize>(
        self,
        task: &CauchyTask<T, N>,
    ) -> impl Iterator<Item = (T, [N; S])>;

    fn next_solution(&mut self, task: &CauchyTask<T, N>) -> (T, &[N]);
}

pub struct EulerSolver<T, N> {
    step: T,
    current_time: T,
    last_solution: Box<[N]>,
}

pub enum Either<S1, S2> {
    Left(S1),
    Right(S2)
}

impl<T: Default, N> EulerSolver<T, N> {
    pub fn new(step: T) -> Self {
        Self {
            step,
            current_time: T::default(),
            last_solution: Box::new([]),
        }
    }
}

impl<T, N> Solver<T, N> for EulerSolver<T, N>
where
    T: Copy + std::ops::Mul<N> + std::ops::Add<Output = T>,
    N: Clone + std::ops::Add<<T as std::ops::Mul<N>>::Output, Output = N>,
{
    fn solve_task<const S: usize>(
        mut self,
        task: &CauchyTask<T, N>,
    ) -> impl Iterator<Item = (T, [N; S])> {
        assert_eq!(task.size, S, "Task size should be equal to given size");
        self.current_time = task.initial_time;
        self.last_solution = task.initial_conditions.clone();

        once((
            self.current_time,
            self.last_solution.first_chunk().unwrap().clone(),
        ))
        .chain(repeat_with(move || {
            let (t, xs) = self.next_solution(task);
            let xs = xs.first_chunk().unwrap();
            (t, xs.clone())
        }))
    }

    fn next_solution(&mut self, task: &CauchyTask<T, N>) -> (T, &[N]) {
        let xs = self
            .last_solution
            .iter()
            .zip(&task.derivatives)
            .map(|(y, f)| y.clone() + self.step * f.eval(self.current_time, &self.last_solution))
            .collect();

        self.last_solution = xs;
        self.current_time = self.current_time + self.step;
        (self.current_time, &self.last_solution)
    }
}

impl<S1, S2, T, N> Solver<T, N> for Either<S1, S2>
where
    S1: Solver<T, N>,
    S2: Solver<T, N>
{
    fn solve_task<const S: usize>(self, task: &CauchyTask<T, N>) -> impl Iterator<Item=(T, [N; S])> {
        match self {
            Either::Left(x) => Either::Left(x.solve_task(task)),
            Either::Right(x) => Either::Right(x.solve_task(task)),
        }
    }

    fn next_solution(&mut self, task: &CauchyTask<T, N>) -> (T, &[N]) {
        match self {
            Either::Left(x) => x.next_solution(task),
            Either::Right(x) => x.next_solution(task)
        }
    }
}

impl<I1, I2, T> Iterator for Either<I1, I2>
where 
    I1: Iterator<Item = T>,
    I2: Iterator<Item = T>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(x) => x.next(),
            Either::Right(x) => x.next()
        }
    }
}
