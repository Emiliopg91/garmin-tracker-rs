use rusqlite::{
    ToSql,
    types::{Null, ToSqlOutput},
};

#[derive(Clone, Debug)]
pub enum Value {
    IntSize(isize),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UntSize(usize),
    Unt8(u8),
    Unt16(u16),
    Unt32(u32),
    Unt64(u64),
    Float32(f32),
    Float64(f64),
    Bool(bool),
    Text(String),
    Null,
}

impl Value {
    pub fn to_sql_str(&self) -> String {
        match self {
            Value::IntSize(v) => v.to_string(),
            Value::Int8(v) => v.to_string(),
            Value::Int16(v) => v.to_string(),
            Value::Int32(v) => v.to_string(),
            Value::Int64(v) => v.to_string(),
            Value::UntSize(v) => v.to_string(),
            Value::Unt8(v) => v.to_string(),
            Value::Unt16(v) => v.to_string(),
            Value::Unt32(v) => v.to_string(),
            Value::Unt64(v) => v.to_string(),
            Value::Float32(v) => v.to_string(),
            Value::Float64(v) => v.to_string(),
            Value::Bool(v) => v.to_string(),
            Value::Text(v) => v.clone(),
            Value::Null => "NULL".to_string(),
        }
    }
}

impl ToSql for Value {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(match self {
            // rusqlite representa enteros internamente como i64,
            // así que los tipos más pequeños se amplían (widening, sin pérdida)
            Value::IntSize(v) => ToSqlOutput::from(*v as i64),
            Value::Int8(v) => ToSqlOutput::from(*v as i64),
            Value::Int16(v) => ToSqlOutput::from(*v as i64),
            Value::Int32(v) => ToSqlOutput::from(*v as i64),
            Value::Int64(v) => ToSqlOutput::from(*v),
            Value::UntSize(v) => ToSqlOutput::from(*v as i64),
            Value::Unt8(v) => ToSqlOutput::from(*v as i64),
            Value::Unt16(v) => ToSqlOutput::from(*v as i64),
            Value::Unt32(v) => ToSqlOutput::from(*v as i64),
            Value::Unt64(v) => ToSqlOutput::from(*v as i64),
            Value::Float32(v) => ToSqlOutput::from(*v as f64),
            Value::Float64(v) => ToSqlOutput::from(*v),
            Value::Bool(v) => ToSqlOutput::from(*v),
            Value::Text(v) => ToSqlOutput::from(v.clone()),
            Value::Null => ToSqlOutput::from(Null),
        })
    }
}

impl From<isize> for Value {
    fn from(v: isize) -> Self {
        Value::IntSize(v)
    }
}
impl From<i8> for Value {
    fn from(v: i8) -> Self {
        Value::Int8(v)
    }
}
impl From<i16> for Value {
    fn from(v: i16) -> Self {
        Value::Int16(v)
    }
}
impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Int32(v)
    }
}
impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int64(v)
    }
}
impl From<usize> for Value {
    fn from(v: usize) -> Self {
        Value::UntSize(v)
    }
}
impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Value::Unt8(v)
    }
}
impl From<u16> for Value {
    fn from(v: u16) -> Self {
        Value::Unt16(v)
    }
}
impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::Unt32(v)
    }
}
impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value::Unt64(v)
    }
}
impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Float32(v)
    }
}
impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Float64(v)
    }
}
impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}
impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Text(v)
    }
}
impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Text(v.to_string())
    }
}
impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => Value::Null,
        }
    }
}
