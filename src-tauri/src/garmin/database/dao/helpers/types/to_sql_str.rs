use std::fmt::Debug;

use rusqlite::ToSql;

pub trait ToSqlStr: ToSql + Debug {
    fn clone_box(&self) -> Box<dyn ToSqlStr>;
}

impl<T> ToSqlStr for T
where
    T: ToSql + Debug + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn ToSqlStr> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ToSqlStr> {
    fn clone(&self) -> Box<dyn ToSqlStr> {
        (**self).clone_box()
    }
}
