use crate::solver::Solver;
use crate::task::CauchyTask;
use crate::Frozen;
use std::ops::Index;

pub struct Solution<T, N> {
    time: Box<[T]>,
    outputs: Box<[N]>,
}

pub enum StopCondition<T> {
    // Absolute maximum time to compute the solution
    Timed { maximum: T },
}

impl<T: PartialOrd, N> Solution<T, N> {
    pub fn time(&self) -> &[T] {
        &self.time
    }

    pub fn compute<S: Solver<T, N>>(
        solver: Frozen<&mut S>,
        task: &CauchyTask<T, N>,
        stop: StopCondition<T>,
    ) -> Self {
        let data = S::solve_task(solver, task)
            .take_while(|(t, _)| match &stop {
                StopCondition::Timed { maximum } => t <= maximum,
            })
            .fold(
                (vec![], Vec::<Vec<N>>::new()),
                |(mut ts, mut xs), (t, x)| {
                    ts.push(t);
                    for (idx, item) in Box::into_iter(x).enumerate() {
                        xs.resize_with((idx + 1).clamp(xs.len(), usize::MAX), || vec![]);
                        xs[idx].push(item);
                    }

                    (ts, xs)
                },
            );
        Self {
            time: data.0.into_boxed_slice(),
            outputs: Box::from_iter(data.1.into_iter().flatten()),
        }
    }
}

impl<T, N> Index<usize> for Solution<T, N> {
    type Output = [N];

    fn index(&self, index: usize) -> &Self::Output {
        let stripe_size = self.time.len();
        &self.outputs[stripe_size * index..stripe_size * (index + 1)]
    }
}
