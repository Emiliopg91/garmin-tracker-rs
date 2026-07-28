use crate::garmin::database::dao::Entity;

use super::column_name::ColumnName;

pub enum OrderBy<T>
where
    T: Entity,
{
    Asc(ColumnName<T>),
    Desc(ColumnName<T>),
}

impl<T> Clone for OrderBy<T>
where
    T: Entity,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for OrderBy<T> where T: Entity {}

impl<T> OrderBy<T>
where
    T: Entity,
{
    pub fn to_sql(self) -> String {
        match self {
            OrderBy::Asc(col) => {
                format!("{} ASC", col)
            }
            OrderBy::Desc(col) => {
                format!("{} DESC", col)
            }
        }
    }
}
