use std::sync::Arc;

trait Provider<T> {
    fn is_complete(&self) -> bool;
    // Option, so I can save allocation of "dummy" when data is not available
    fn get(&self) -> Option<Arc<T>>;
}