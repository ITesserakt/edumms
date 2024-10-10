mod config;
pub mod plot;

use crate::config::Config;
use crate::plot::{Line, Plotter};
use anyhow::Error;
use libloading::{library_filename, Library};
use plotters::prelude::{Color, ShapeStyle, BLUE, GREEN, RED};
use project::ffi::{CanSolve, ExternalSolver};
use project::interval::Interval;
use project::solution::{Solution, StopCondition};
use project::solver::{Either, EulerSolver, Solver};
use project::task::{f, CauchyTask};
use project::Frozen;
use std::fs::File;
use std::io::{Read, Write};
use std::ops::{Add, Mul, Neg, Sub};
use std::sync::LazyLock;

fn build_line(
    xs: &[f64],
    ys: &[f64],
    color: impl Into<ShapeStyle>,
    label: impl Into<String>,
) -> Vec<Line> {
    vec![Line::new(
        xs.iter().cloned().zip(ys.iter().cloned()),
        color,
        label,
        false,
    )]
}

fn build_line_interval(
    xs: &[f64],
    ys: &[Interval<f64>],
    color: impl Color + Clone,
    label: impl Into<String> + Clone,
    dashed: bool,
) -> Vec<Line> {
    let start_color = color.mix(0.5);
    let end_color = color.mix(0.5);

    vec![
        Line::new(
            xs.iter().cloned().zip(ys.iter().map(|it| it.start())),
            start_color,
            label.clone(),
            dashed,
        ),
        Line::new(
            xs.iter().cloned().zip(ys.iter().map(|it| it.end())),
            end_color,
            label,
            dashed,
        ),
    ]
}

const CONFIG_PATH: &'static str = "config.toml";

static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let Ok(mut config_file) = File::open(CONFIG_PATH) else {
        return Config::default();
    };
    let mut buffer = String::new();
    config_file
        .read_to_string(&mut buffer)
        .expect("Could not read config file");
    toml::from_str(&buffer)
        .map_err(|e| format!("{} at {:?}", e.message(), e.span()))
        .expect("Could not parse config file")
});

static LIBRARY: LazyLock<Library> = LazyLock::new(|| {
    let mut path = CONFIG.general.lib_dir.clone();
    let solver_lib_name = library_filename(&CONFIG.general.solver);
    path.push(solver_lib_name);
    unsafe { Library::new(path).expect("Could not load solver library") }
});

fn get_task<N>(coeffs: [N; 2]) -> CauchyTask<f64, N>
where
    N: From<f64> + Copy + Neg<Output = N> + Mul<Output = N> + Sub<Output = N> + 'static,
{
    CauchyTask::new(
        [
            f(move |_, &[x1, _, _]| -coeffs[0] * x1),
            f(move |_, &[x1, x2, _]| coeffs[0] * x1 - coeffs[1] * x2),
            f(move |_, &[_, x2, _]| coeffs[1] * x2),
        ],
        0.0,
        [1.0, 0.0, 0.0].map(N::from),
    )
}

fn get_solver<N>() -> Frozen<impl Solver<f64, N>>
where
    for<'a> ExternalSolver<'a, f64, N>: CanSolve<f64, N>,
    N: Clone + Add<Output = N> + 'static,
    f64: Mul<N, Output = N>,
{
    if CONFIG.general.solver == "builtin" {
        Either::Left(EulerSolver::new(0.1))
    } else {
        Either::Right(unsafe { ExternalSolver::build(&*LIBRARY) }.expect("Cannot build solver"))
    }
    .rewrap()
}

fn main() -> Result<(), Error> {
    let solution_interval = Solution::compute(
        get_solver().as_mut(),
        &get_task([Interval::new(0.576, 0.578), Interval::from(0.422)]),
        StopCondition::Timed {
            maximum: CONFIG.general.t_max,
        },
    );

    let solution_bench = Solution::compute(
        get_solver().as_mut(),
        &get_task([0.577, 0.422]),
        StopCondition::Timed {
            maximum: CONFIG.general.t_max,
        },
    );

    // Save csv file with computed values
    let mut csv_output_file = File::create(CONFIG.general.output_dir.join("data.csv"))?;
    writeln!(csv_output_file, "t, x1, x2, x3")?;
    for (idx, t) in solution_bench.time().iter().enumerate() {
        writeln!(
            csv_output_file,
            "{}, {}, {}, {}",
            t, solution_bench[0][idx], solution_bench[1][idx], solution_bench[2][idx]
        )?;
    }

    let ts = solution_bench.time();
    Plotter::new(
        CONFIG.general.output_dir.join("plot.svg"),
        CONFIG.plotting.plot_size,
        (
            CONFIG.plotting.viewport.x.clone(),
            CONFIG.plotting.viewport.y.clone(),
        ),
        [
            build_line_interval(ts, &solution_interval[0], &RED, "x_1", false),
            build_line_interval(ts, &solution_interval[1], &GREEN, "x_2", false),
            build_line_interval(ts, &solution_interval[2], &BLUE, "x_3", false),
            build_line(ts, &solution_bench[0], RED.stroke_width(2), "x_1"),
            build_line(ts, &solution_bench[1], GREEN.stroke_width(2), "x_1"),
            build_line(ts, &solution_bench[2], BLUE.stroke_width(2), "x_1"),
        ]
        .into_iter()
        .flatten(),
    )
    .draw(CONFIG.plotting.output_type)?;

    Ok(())
}
