use crate::garmin::database::dao::helpers::types::value::Value;

use super::column_name::ColumnName;

#[derive(Clone)]
pub enum Where {
    Eq(ColumnName, Value),
    NotEq(ColumnName, Value),
    Gt(ColumnName, Value),
    Lt(ColumnName, Value),
    Null(ColumnName),
    NotNull(ColumnName),
    And(Vec<Where>),
    Or(Vec<Where>),
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
            Self::And(conditions) => conditions
                .clone()
                .into_iter()
                .map(|condition| match condition {
                    Where::And(_) | Where::Or(_) => {
                        format!("({})", condition.to_sql())
                    }
                    _ => condition.to_sql(),
                })
                .collect::<Vec<String>>()
                .join(" AND "),
            Self::Or(conditions) => conditions
                .clone()
                .into_iter()
                .map(|condition| match condition {
                    Where::And(_) | Where::Or(_) => {
                        format!("({})", condition.to_sql())
                    }
                    _ => condition.to_sql(),
                })
                .collect::<Vec<String>>()
                .join(" OR "),
        }
    }

    pub fn into_params(self) -> Vec<Value> {
        match self {
            Self::Eq(_, val) | Self::NotEq(_, val) | Self::Gt(_, val) | Self::Lt(_, val) => {
                vec![val]
            }
            Self::Null(_) | Self::NotNull(_) => {
                vec![]
            }
            Self::And(conditions) | Self::Or(conditions) => {
                let mut params = vec![];
                for condition in conditions {
                    params.extend(condition.into_params());
                }
                params
            }
        }
    }
}
