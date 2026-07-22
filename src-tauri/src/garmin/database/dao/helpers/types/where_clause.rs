use crate::garmin::database::dao::helpers::types::to_sql_str::ToSqlStr;

#[derive(Clone)]
pub enum Where {
    Eq(&'static str, Box<dyn ToSqlStr>),
    NotEq(&'static str, Box<dyn ToSqlStr>),
    Gt(&'static str, Box<dyn ToSqlStr>),
    Lt(&'static str, Box<dyn ToSqlStr>),
    Null(&'static str),
    NotNull(&'static str),
    And(Box<Where>, Box<Where>),
    Or(Box<Where>, Box<Where>),
}

impl Where {
    pub fn to_sql(&self) -> String {
        match self {
            Self::Eq(col, _) => {
                format!("{}=?", col)
            }
            Self::NotEq(col, _) => {
                format!("{}!=?", col)
            }
            Self::Gt(col, _) => {
                format!("{}>?", col)
            }
            Self::Lt(col, _) => {
                format!("{}<?", col)
            }
            Self::Null(col) => {
                format!("{} IS NULL", col)
            }
            Self::NotNull(col) => {
                format!("{} IS NOT NULL", col)
            }
            Self::And(cond1, cond2) => {
                format!("({} AND {})", cond1.to_sql(), cond2.to_sql())
            }
            Self::Or(cond1, cond2) => {
                format!("({} OR {})", cond1.to_sql(), cond2.to_sql())
            }
        }
    }

    pub fn into_params(self) -> Vec<Box<dyn ToSqlStr>> {
        match self {
            Self::Eq(_, val) | Self::NotEq(_, val) | Self::Gt(_, val) | Self::Lt(_, val) => {
                vec![val]
            }
            Self::Null(_) | Self::NotNull(_) => {
                vec![]
            }
            Self::And(cond1, cond2) | Self::Or(cond1, cond2) => {
                let mut params = cond1.into_params();
                params.extend(cond2.into_params());
                params
            }
        }
    }
}
