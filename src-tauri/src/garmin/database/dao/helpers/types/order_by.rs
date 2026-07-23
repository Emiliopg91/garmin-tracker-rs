use super::column_name::ColumnName;

pub enum OrderBy {
    Asc(ColumnName),
    Desc(ColumnName),
}

impl OrderBy {
    pub fn to_sql(&self) -> String {
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
