#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct ColumnName(&'static str);

impl AsRef<str> for ColumnName {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl std::fmt::Display for ColumnName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ColumnName {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}
