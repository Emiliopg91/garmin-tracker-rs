use crate::garmin::database::dao::{Entity, helpers::types::value::Value};

use super::column_name::ColumnName;

pub enum Where<T>
where
    T: Entity,
{
    Eq(ColumnName<T>, Value),
    NotEq(ColumnName<T>, Value),
    Gt(ColumnName<T>, Value),
    Lt(ColumnName<T>, Value),
    In(ColumnName<T>, Vec<Value>),
    InMultiple(Vec<ColumnName<T>>, Vec<Vec<Value>>),
    Null(ColumnName<T>),
    NotNull(ColumnName<T>),
    And(Vec<Where<T>>),
    Or(Vec<Where<T>>),
}

impl<T> Where<T>
where
    T: Entity,
{
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
            Self::In(col, values) => {
                format!("{} IN ({})", col, vec!["?"; values.len()].join(", "))
            }
            Self::InMultiple(cols, values) => {
                let col_list = cols
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                let num_rows = values.len();

                let tuple = format!("({})", vec!["?"; cols.len()].join(", "));
                let tuples = vec![tuple; num_rows].join(", ");

                format!("({}) IN ({})", col_list, tuples)
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
            Self::In(_, vals) => vals,
            Self::InMultiple(_, vals_arr) => {
                let mut params = vec![];
                for vals in vals_arr {
                    for val in vals {
                        params.push(val)
                    }
                }

                params
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

impl<T> Clone for Where<T>
where
    T: Entity,
{
    fn clone(&self) -> Self {
        match self {
            Self::Eq(col, val) => Self::Eq(*col, val.clone()),
            Self::NotEq(col, val) => Self::NotEq(*col, val.clone()),
            Self::Gt(col, val) => Self::Gt(*col, val.clone()),
            Self::Lt(col, val) => Self::Lt(*col, val.clone()),
            Self::In(col, vals) => Self::In(*col, vals.clone()),
            Self::InMultiple(cols, vals) => Self::InMultiple(cols.clone(), vals.clone()),
            Self::Null(col) => Self::Null(*col),
            Self::NotNull(col) => Self::NotNull(*col),
            Self::And(conds) => Self::And(conds.clone()),
            Self::Or(conds) => Self::Or(conds.clone()),
        }
    }
}
