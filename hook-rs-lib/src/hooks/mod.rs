pub mod vtable;

pub trait Hook: Sync + Send {
    fn new(original_function: usize) -> Self
    where
        Self: Sized;
    fn original_function(&self) -> usize;
}
