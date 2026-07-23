use std::fmt::Debug;

use rusqlite::ToSql;

pub trait ToSqlStr: ToSql + Debug {}
impl<T> ToSqlStr for T where T: ToSql + Debug + Clone + 'static {}
