use crate::task::CauchyTask;
use num_traits::Float;
use std::iter::once;

struct SolutionIter<'t, S, T, N> {
    solver: S,
    task: &'t CauchyTask<T, N>,
}

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

pub struct EulerSolver<T> {
    h: T,
    last_solution: Box<[T]>,
    current_time: T,
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

impl<S, T, N> Iterator for SolutionIter<'_, S, T, N>
where
    S: Solver<T, N>,
    N: Clone,
{
    type Item = (T, Box<[N]>);

    fn next(&mut self) -> Option<Self::Item> {
        let (x, y) = self.solver.next_solution(self.task);
        Some((x, Box::<[N]>::from(y)))
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
            .chain(
                SolutionIter { solver: self, task }.take_while(|(t, _)| match stop_condition {
                    StopCondition::Timed { maximum } => t < &maximum,
                }),
            )
            .collect()
    }

    fn next_solution(&mut self, task: &CauchyTask<T, T>) -> (T, &[T]) {
        let yi = self
            .last_solution
            .iter()
            .zip(&task.definitions)
            .map(|(&y_prev, f)| y_prev + self.h * f.eval(self.current_time, &self.last_solution))
            .collect();
        let xi = self.current_time + self.h;

        self.last_solution = yi;
        self.current_time = xi;

        (self.current_time, &self.last_solution)
    }
}
