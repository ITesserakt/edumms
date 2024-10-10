use crate::task::CauchyTask;
use std::iter::{once, repeat_with};
use crate::Frozen;

pub trait Solver<T, N>: Sized {
    fn solve_task(
        this: Frozen<&mut Self>,
        task: &CauchyTask<T, N>,
    ) -> impl Iterator<Item = (T, Box<[N]>)>;

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
    pub fn new(step: T) -> Frozen<Self> {
        Frozen(Self {
            step,
            current_time: T::default(),
            last_solution: Box::new([]),
        })
    }
}

impl<S1, S2> Either<Frozen<S1>, Frozen<S2>> {
    pub fn rewrap(self) -> Frozen<Either<S1, S2>> {
        match self {
            Either::Left(x) => Frozen(Either::Left(x.0)),
            Either::Right(x) => Frozen(Either::Right(x.0))
        }
    }
}

impl<T, N> Solver<T, N> for EulerSolver<T, N>
where
    T: Copy + std::ops::Mul<N> + std::ops::Add<Output = T>,
    N: Clone + std::ops::Add<<T as std::ops::Mul<N>>::Output, Output = N>,
{
    fn solve_task(
        this: Frozen<&mut Self>,
        task: &CauchyTask<T, N>,
    ) -> impl Iterator<Item = (T, Box<[N]>)> {
        let this = this.init(|it| {
            it.current_time = task.initial_time;
            it.last_solution = task.initial_conditions.clone();
        });

        once((
            this.current_time,
            this.last_solution.clone(),
        ))
        .chain(repeat_with(move || {
            let (t, xs) = this.next_solution(task);
            assert_eq!(task.size, xs.len(), "Task size should be equal to outputs size");
            (t, Box::from(xs))
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
    fn solve_task(this: Frozen<&mut Self>, task: &CauchyTask<T, N>) -> impl Iterator<Item=(T, Box<[N]>)> {
        match this.0 {
            Either::Left(x) => Either::Left(Solver::solve_task(Frozen(x), task)),
            Either::Right(x) => Either::Right(Solver::solve_task(Frozen(x), task)),
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
