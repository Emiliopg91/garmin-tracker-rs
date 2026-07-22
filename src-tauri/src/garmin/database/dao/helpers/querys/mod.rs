pub mod insert;
pub mod select;
pub mod update;

pub trait QueryBuilder<T> {
    fn new() -> Self;
}
