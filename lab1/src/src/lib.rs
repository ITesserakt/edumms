pub mod task;
pub mod solver;
pub mod ffi;
pub mod interval;
pub mod solution;

pub struct Frozen<T>(pub(crate) T);

impl<T> Frozen<T> {
    pub fn as_mut(&mut self) -> Frozen<&mut T> {
        Frozen(&mut self.0)
    }

    pub(crate) fn init(mut self, f: impl FnOnce(&mut T)) -> T {
        f(&mut self.0);
        self.0
    }
}
