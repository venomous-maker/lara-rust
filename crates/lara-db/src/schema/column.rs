/// A column definition within a schema blueprint.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: ColumnType,
    pub nullable: bool,
    pub default: Option<String>,
    pub unique: bool,
    pub primary: bool,
    pub auto_increment: bool,
    pub unsigned: bool,
    pub comment: Option<String>,
    pub references: Option<ForeignKeyRef>,
}

#[derive(Debug, Clone)]
pub enum ColumnType {
    TinyInteger,
    SmallInteger,
    Integer,
    BigInteger,
    Float,
    Double,
    Decimal { precision: u8, scale: u8 },
    Char(u32),
    Varchar(u32),
    Text,
    MediumText,
    LongText,
    Boolean,
    Date,
    Time,
    DateTime,
    Timestamp,
    Json,
    Uuid,
    Binary,
    Enum(Vec<String>),
    Blob,
    Custom(String),
}

impl ColumnType {
    pub fn to_sql(&self, grammar: crate::connection::Grammar) -> String {
        use crate::connection::Grammar;
        match (self, grammar) {
            (ColumnType::TinyInteger, _)    => "TINYINT".into(),
            (ColumnType::SmallInteger, _)   => "SMALLINT".into(),
            (ColumnType::Integer, _)        => "INTEGER".into(),
            (ColumnType::BigInteger, _)     => "BIGINT".into(),
            (ColumnType::Float, _)          => "FLOAT".into(),
            (ColumnType::Double, _)         => "DOUBLE PRECISION".into(),
            (ColumnType::Decimal { precision, scale }, _) => format!("DECIMAL({}, {})", precision, scale),
            (ColumnType::Char(n), _)        => format!("CHAR({})", n),
            (ColumnType::Varchar(n), _)     => format!("VARCHAR({})", n),
            (ColumnType::Text, _)           => "TEXT".into(),
            (ColumnType::MediumText, Grammar::Mysql) => "MEDIUMTEXT".into(),
            (ColumnType::MediumText, _)     => "TEXT".into(),
            (ColumnType::LongText, Grammar::Mysql)   => "LONGTEXT".into(),
            (ColumnType::LongText, _)       => "TEXT".into(),
            (ColumnType::Boolean, Grammar::Postgres) => "BOOLEAN".into(),
            (ColumnType::Boolean, _)        => "TINYINT(1)".into(),
            (ColumnType::Date, _)           => "DATE".into(),
            (ColumnType::Time, _)           => "TIME".into(),
            (ColumnType::DateTime, Grammar::Postgres) => "TIMESTAMP".into(),
            (ColumnType::DateTime, _)       => "DATETIME".into(),
            (ColumnType::Timestamp, _)      => "TIMESTAMP".into(),
            (ColumnType::Json, Grammar::Mysql | Grammar::Postgres) => "JSON".into(),
            (ColumnType::Json, _)           => "TEXT".into(),
            (ColumnType::Uuid, Grammar::Postgres) => "UUID".into(),
            (ColumnType::Uuid, _)           => "CHAR(36)".into(),
            (ColumnType::Binary, Grammar::Postgres) => "BYTEA".into(),
            (ColumnType::Binary, _)         => "BLOB".into(),
            (ColumnType::Blob, _)           => "BLOB".into(),
            (ColumnType::Enum(v), Grammar::Mysql) => {
                format!("ENUM({})", v.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", "))
            }
            (ColumnType::Enum(_), _)        => "VARCHAR(255)".into(),
            (ColumnType::Custom(s), _)      => s.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ForeignKeyRef {
    pub referenced_table: String,
    pub referenced_column: String,
    pub on_delete: Option<ForeignKeyAction>,
    pub on_update: Option<ForeignKeyAction>,
}

#[derive(Debug, Clone, Copy)]
pub enum ForeignKeyAction {
    Cascade,
    Restrict,
    SetNull,
    NoAction,
}

impl ForeignKeyAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cascade  => "CASCADE",
            Self::Restrict => "RESTRICT",
            Self::SetNull  => "SET NULL",
            Self::NoAction => "NO ACTION",
        }
    }
}

impl ColumnDef {
    pub fn new(name: &str, col_type: ColumnType) -> Self {
        Self {
            name: name.to_string(),
            col_type,
            nullable: false,
            default: None,
            unique: false,
            primary: false,
            auto_increment: false,
            unsigned: false,
            comment: None,
            references: None,
        }
    }

    // All modifier methods return `&mut Self` so they can be chained on `&mut ColumnDef`
    pub fn nullable(&mut self) -> &mut Self    { self.nullable = true; self }
    pub fn not_null(&mut self) -> &mut Self    { self.nullable = false; self }
    pub fn unique(&mut self) -> &mut Self      { self.unique = true; self }
    pub fn primary(&mut self) -> &mut Self     { self.primary = true; self }
    pub fn auto_increment(&mut self) -> &mut Self { self.auto_increment = true; self }
    pub fn unsigned(&mut self) -> &mut Self    { self.unsigned = true; self }
    pub fn default(&mut self, val: impl Into<String>) -> &mut Self {
        self.default = Some(val.into()); self
    }
    pub fn comment(&mut self, c: impl Into<String>) -> &mut Self {
        self.comment = Some(c.into()); self
    }
    pub fn references(&mut self, table: &str, column: &str) -> &mut Self {
        self.references = Some(ForeignKeyRef {
            referenced_table: table.to_string(),
            referenced_column: column.to_string(),
            on_delete: None,
            on_update: None,
        });
        self
    }
    pub fn on_delete(&mut self, action: ForeignKeyAction) -> &mut Self {
        if let Some(ref mut r) = self.references { r.on_delete = Some(action); }
        self
    }
    pub fn on_update(&mut self, action: ForeignKeyAction) -> &mut Self {
        if let Some(ref mut r) = self.references { r.on_update = Some(action); }
        self
    }

    pub fn to_sql(&self, grammar: crate::connection::Grammar) -> String {
        use crate::connection::Grammar;
        let type_str = if self.auto_increment && matches!(grammar, Grammar::Postgres) {
            // Use SERIAL / BIGSERIAL for Postgres auto-increment
            match &self.col_type {
                ColumnType::Integer    => "SERIAL".to_string(),
                ColumnType::BigInteger => "BIGSERIAL".to_string(),
                _ => self.col_type.to_sql(grammar),
            }
        } else {
            self.col_type.to_sql(grammar)
        };

        let mut sql = format!("{} {}", self.name, type_str);
        if self.unsigned { sql.push_str(" UNSIGNED"); }
        if self.primary  { sql.push_str(" PRIMARY KEY"); }
        if self.auto_increment && !matches!(grammar, Grammar::Postgres) {
            sql.push_str(" AUTO_INCREMENT");
        }
        if !self.nullable { sql.push_str(" NOT NULL"); }
        if let Some(ref d) = self.default { sql.push_str(&format!(" DEFAULT {}", d)); }
        if self.unique { sql.push_str(" UNIQUE"); }
        sql
    }
}
