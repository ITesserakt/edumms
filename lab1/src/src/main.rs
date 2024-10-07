use anyhow::Error;
use project::solver::{EulerSolver, Solver, StopCondition};
use project::task::{f, CauchyTask};
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Error> {
    let k1 = 0.577;
    let k2 = 0.422;

    let task = CauchyTask::new(
        [
            f(move |_, [x1, _, _]| -k1 * x1),
            f(move |_, [x1, x2, _]| k1 * x1 - k2 * x2),
            f(move |_, [_, x2, _]| k2 * x2),
        ],
        0.0,
        [1.0, 0.0, 0.0],
    );

    let mut output_file = File::create("output.csv")?;

    // Write csv header
    writeln!(&mut output_file, "t, x1, x2, x3")?;
    for (t, xs) in EulerSolver::new(0.1)
        .solve_task(&task, StopCondition::Timed { maximum: 10.0 })
        .into_iter()
    {
        writeln!(output_file, "{}, {}, {}, {}", t, xs[0], xs[1], xs[2])?;
    }

    Ok(())
}
