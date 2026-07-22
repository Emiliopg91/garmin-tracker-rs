pub enum OrderBy {
    Asc(&'static str),
    Desc(&'static str),
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
