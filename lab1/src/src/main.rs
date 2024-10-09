mod config;
pub mod plot;

use crate::config::Config;
use crate::plot::{Line, Plotter};
use anyhow::Error;
use itertools::Itertools;
use libloading::{library_filename, Library};
use plotters::prelude::{Color, BLUE, GREEN, RED};
use project::solver::{ExternalSolver, Solver};
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
    let solver_lib_name = library_filename(&CONFIG.general.solver);
    path.push(solver_lib_name);
    unsafe { Library::new(path).expect("Could not load solver library") }
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

    // Write csv header
    writeln!(&mut output_file, "t, x1, x2, x3")?;
    // OMG very unsafe code
    let solver = unsafe { ExternalSolver::build(&*LIBRARY)? };

    // Compute sequence of solutions simultaneously writing them into csv file
    let (ts, xs1, xs2, xs3): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) = solver
        .solve_task(&task)
        .take_while(|(t, _): &(f64, _)| t.abs() <= CONFIG.general.t_max.abs())
        .map(|(t, [x1, x2, x3])| (t, x1, x2, x3))
        .inspect(|(t, x1, x2, x3)| writeln!(output_file, "{}, {}, {}, {}", t, x1, x2, x3).unwrap())
        .multiunzip();

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
    .draw(CONFIG.general.output_type)?;

    Ok(())
}
