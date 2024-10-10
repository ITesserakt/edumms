use std::ops::Range;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: Runtime,
    #[serde(default)]
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
    pub solver: String,
    #[serde(default)]
    pub output_type: Output
}

#[derive(Serialize, Deserialize, Default, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Output {
    Png,
    #[default]
    Svg
}

#[derive(Serialize, Deserialize)]
pub struct Viewport {
    #[serde(default = "def_viewport")]
    pub x: Range<f64>,
    #[serde(default = "def_viewport")]
    pub y: Range<f64>
}

#[derive(Serialize, Deserialize)]
pub struct Plot {
    #[serde(default)]
    pub viewport: Viewport,
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

impl Default for Viewport {
    fn default() -> Self {
        Self {
            x: def_viewport(),
            y: def_viewport(),
        }
    }
}

impl Default for Plot {
    fn default() -> Self {
        Self {
            viewport: Default::default(),
            plot_size: def_plot_size(),
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            t_max: def_t_max(),
            output_dir: def_output_dir(),
            lib_dir: def_lib_dir(),
            solver: "builtin".to_string(),
            output_type: Default::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: Default::default(),
            plotting: Default::default(),
        }
    }
}
