pub trait Downgrade<T> {
    fn is_downgradable(&self) -> bool;
    fn downgrade(&self) -> Option<T>;
}
