mod plot;

use crate::plot::{Line, Plotter};
use anyhow::Error;
use plotters::prelude::{Color, BLUE, GREEN, RED};
use project::solver::{ExternalSolver, Solver, StopCondition};
use project::task::{f, CauchyTask};
use std::fs::File;
use std::io::Write;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::LazyLock;
use libloading::Library;

fn build_line(
    xs: &[f64],
    ys: &[f64],
    color: impl Color,
    label: impl Into<String>,
    dashed: bool,
) -> Line {
    Line::new(
        xs.iter().cloned().zip(ys.iter().cloned()),
        color,
        label,
        dashed,
    )
}

const RUN_TIME: f64 = 10.0;
const VIEWPORT_SIZE: (Range<f64>, Range<f64>) = (-0.1..RUN_TIME + 0.1, -0.1..1.1);
const PLOT_SIZE: (u32, u32) = (640, 480);
const ARTIFACT_NAME: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("./out/plot"));
const SOLVER_PATH: &'static str = "solvers/solver.so";

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

    let mut output_file = File::create(ARTIFACT_NAME.with_extension("csv"))?;
    let (mut ts, mut xs1, mut xs2, mut xs3) = (vec![], vec![], vec![], vec![]);

    // Write csv header
    writeln!(&mut output_file, "t, x1, x2, x3")?;
    let library = unsafe { Library::new(SOLVER_PATH)? };
    let solver = unsafe { ExternalSolver::build(&library)? };
    let stop = StopCondition::Timed { maximum: RUN_TIME };

    for (t, xs) in solver.solve_task(&task, stop).into_iter() {
        ts.push(t);
        xs1.push(xs[0]);
        xs2.push(xs[1]);
        xs3.push(xs[2]);
        writeln!(output_file, "{}, {}, {}, {}", t, xs[0], xs[1], xs[2])?;
    }

    Plotter::new(
        ARTIFACT_NAME.with_extension("svg"),
        PLOT_SIZE,
        VIEWPORT_SIZE,
        [
            build_line(&ts, &xs1, &RED, "x_1", false),
            build_line(&ts, &xs2, &GREEN, "x_2", false),
            build_line(&ts, &xs3, &BLUE, "x_3", false),
        ],
    )
    .draw()?;

    Ok(())
}
