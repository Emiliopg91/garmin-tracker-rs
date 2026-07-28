use std::marker::PhantomData;

#[repr(transparent)]
pub struct ColumnName<T>(&'static str, PhantomData<T>);

impl<T> Clone for ColumnName<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ColumnName<T> {}

impl<T> AsRef<str> for ColumnName<T> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<T> std::fmt::Display for ColumnName<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> ColumnName<T> {
    pub const fn new(value: &'static str) -> Self {
        Self(value, PhantomData)
    }
}
