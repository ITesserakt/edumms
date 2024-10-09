mod config;
mod plot;

use crate::config::Config;
use crate::plot::{Line, Plotter};
use anyhow::Error;
use libloading::Library;
use plotters::prelude::{Color, BLUE, GREEN, RED};
use project::solver::{ExternalSolver, Solver, StopCondition};
use project::task::{f, CauchyTask};
use std::fs::File;
use std::io::{Read, Write};
use std::sync::LazyLock;

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
    path.push(&CONFIG.general.solver);
    let solver_lib_path = path.with_extension("so");
    unsafe { Library::new(solver_lib_path).expect("Could not load solver library") }
});

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

    let mut output_file = File::create(CONFIG.general.output_dir.join("data.csv"))?;
    let (mut ts, mut xs1, mut xs2, mut xs3) = (vec![], vec![], vec![], vec![]);

    // Write csv header
    writeln!(&mut output_file, "t, x1, x2, x3")?;
    let solver = unsafe { ExternalSolver::build(&*LIBRARY)? };
    let stop = StopCondition::Timed {
        maximum: CONFIG.general.t_max,
    };

    for (t, xs) in solver.solve_task(&task, stop).into_iter() {
        ts.push(t);
        xs1.push(xs[0]);
        xs2.push(xs[1]);
        xs3.push(xs[2]);
        writeln!(output_file, "{}, {}, {}, {}", t, xs[0], xs[1], xs[2])?;
    }

    Plotter::new(
        CONFIG.general.output_dir.join("plot.svg"),
        CONFIG.plotting.plot_size,
        (
            CONFIG.plotting.viewport.x.clone(),
            CONFIG.plotting.viewport.y.clone(),
        ),
        [
            build_line(&ts, &xs1, &RED, "x_1", false),
            build_line(&ts, &xs2, &GREEN, "x_2", false),
            build_line(&ts, &xs3, &BLUE, "x_3", false),
        ],
    )
    .draw()?;

    Ok(())
}
