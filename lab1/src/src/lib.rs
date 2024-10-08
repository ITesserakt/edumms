extern crate core;

pub mod task;
pub mod solver;
pub mod ffi;

macro_rules! assert_is_object_safe {
    ($t:path) => {
        const _: [Box<dyn $t>; 0] = [];
    };
}

pub(crate) use assert_is_object_safe;
