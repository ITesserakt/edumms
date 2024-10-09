use std::ops::Range;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "def_runtime")]
    pub general: Runtime,
    #[serde(default = "def_plotting")]
    pub plotting: Plot
}

#[derive(Serialize, Deserialize)]
pub struct Runtime {
    #[serde(default = "def_t_max")]
    pub t_max: f64,
    #[serde(default = "def_output_dir")]
    pub output_dir: PathBuf,
    #[serde(default = "def_lib_dir")]
    pub lib_dir: PathBuf,
    pub solver: String
}

#[derive(Serialize, Deserialize)]
pub struct Plot {
    #[serde(default = "def_viewport")]
    pub viewport_x: Range<f64>,
    #[serde(default = "def_viewport")]
    pub viewport_y: Range<f64>,
    #[serde(default = "def_plot_size")]
    pub plot_size: (u32, u32)
}

fn def_plot_size() -> (u32, u32) {
    (640, 480)
}

fn def_viewport() -> Range<f64> {
    -0.1..1.1
}

fn def_lib_dir() -> PathBuf {
    PathBuf::from("./solvers/")
}

fn def_output_dir() -> PathBuf {
    PathBuf::from("./out/")
}

fn def_t_max() -> f64 {
    1.0
}

fn def_runtime() -> Runtime {
    Runtime {
        t_max: def_t_max(),
        output_dir: def_output_dir(),
        lib_dir: def_lib_dir(),
        solver: "euler".to_string()
    }
}

fn def_plotting() -> Plot {
    Plot {
        viewport_x: def_viewport(),
        viewport_y: def_viewport(),
        plot_size: def_plot_size(),
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: def_runtime(),
            plotting: def_plotting(),
        }
    }
}
