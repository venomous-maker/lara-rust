use super::column::{ColumnDef, ColumnType, ForeignKeyAction};
use crate::connection::Grammar;

#[derive(Debug, Clone)]
pub struct IndexDef {
    pub name: Option<String>,
    pub columns: Vec<String>,
    pub unique: bool,
}

pub struct Blueprint {
    pub table: String,
    pub columns: Vec<ColumnDef>,
    pub indexes: Vec<IndexDef>,
    pub drop_columns: Vec<String>,
    pub is_create: bool,
}

impl Blueprint {
    pub fn create(table: &str) -> Self {
        Self { table: table.to_string(), columns: Vec::new(), indexes: Vec::new(), drop_columns: Vec::new(), is_create: true }
    }

    pub fn table(table: &str) -> Self {
        Self { table: table.to_string(), columns: Vec::new(), indexes: Vec::new(), drop_columns: Vec::new(), is_create: false }
    }

    // ── Helper: push a column and return &mut to it ──────────────────────────

    fn push(&mut self, col: ColumnDef) -> &mut ColumnDef {
        self.columns.push(col);
        self.columns.last_mut().unwrap()
    }

    // ── Integer columns ───────────────────────────────────────────────────────

    pub fn id(&mut self) -> &mut ColumnDef {
        let mut c = ColumnDef::new("id", ColumnType::BigInteger);
        c.auto_increment(); c.primary(); c.unsigned();
        self.push(c)
    }

    pub fn integer(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::Integer))
    }

    pub fn big_integer(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::BigInteger))
    }

    pub fn tiny_integer(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::TinyInteger))
    }

    pub fn small_integer(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::SmallInteger))
    }

    pub fn unsigned_integer(&mut self, name: &str) -> &mut ColumnDef {
        let mut c = ColumnDef::new(name, ColumnType::Integer); c.unsigned();
        self.push(c)
    }

    pub fn unsigned_big_integer(&mut self, name: &str) -> &mut ColumnDef {
        let mut c = ColumnDef::new(name, ColumnType::BigInteger); c.unsigned();
        self.push(c)
    }

    // ── Float/Decimal ─────────────────────────────────────────────────────────

    pub fn float(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::Float))
    }

    pub fn double(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::Double))
    }

    pub fn decimal(&mut self, name: &str, precision: u8, scale: u8) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::Decimal { precision, scale }))
    }

    // ── String columns ────────────────────────────────────────────────────────

    pub fn string(&mut self, name: &str, length: u32) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::Varchar(length)))
    }

    pub fn char(&mut self, name: &str, length: u32) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::Char(length)))
    }

    pub fn text(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::Text))
    }

    pub fn medium_text(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::MediumText))
    }

    pub fn long_text(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::LongText))
    }

    // ── Other columns ─────────────────────────────────────────────────────────

    pub fn boolean(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::Boolean))
    }

    pub fn json(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::Json))
    }

    pub fn uuid(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::Uuid))
    }

    pub fn binary(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::Binary))
    }

    pub fn enum_col(&mut self, name: &str, variants: &[&str]) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::Enum(variants.iter().map(|s| s.to_string()).collect())))
    }

    // ── Date/time ─────────────────────────────────────────────────────────────

    pub fn date(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::Date))
    }

    pub fn time(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::Time))
    }

    pub fn date_time(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::DateTime))
    }

    pub fn timestamp(&mut self, name: &str) -> &mut ColumnDef {
        self.push(ColumnDef::new(name, ColumnType::Timestamp))
    }

    pub fn timestamps(&mut self) {
        let mut ca = ColumnDef::new("created_at", ColumnType::Timestamp); ca.nullable();
        let mut ua = ColumnDef::new("updated_at", ColumnType::Timestamp); ua.nullable();
        self.columns.push(ca);
        self.columns.push(ua);
    }

    pub fn soft_deletes(&mut self) {
        let mut da = ColumnDef::new("deleted_at", ColumnType::Timestamp); da.nullable();
        self.columns.push(da);
    }

    // ── Foreign keys ──────────────────────────────────────────────────────────

    pub fn foreign(&mut self, column: &str) -> &mut ColumnDef {
        self.unsigned_big_integer(column)
    }

    pub fn foreign_id(&mut self, name: &str) -> &mut ColumnDef {
        self.unsigned_big_integer(name)
    }

    // ── Indexes ───────────────────────────────────────────────────────────────

    pub fn index(&mut self, columns: &[&str]) {
        self.indexes.push(IndexDef { name: None, columns: columns.iter().map(|s| s.to_string()).collect(), unique: false });
    }

    pub fn unique_index(&mut self, columns: &[&str]) {
        self.indexes.push(IndexDef { name: None, columns: columns.iter().map(|s| s.to_string()).collect(), unique: true });
    }

    // ── Drop ──────────────────────────────────────────────────────────────────

    pub fn drop_column(&mut self, name: &str) {
        self.drop_columns.push(name.to_string());
    }

    // ── SQL compilation ───────────────────────────────────────────────────────

    pub fn to_sql(&self, grammar: Grammar) -> Vec<String> {
        let mut statements = Vec::new();

        if self.is_create {
            let col_defs: Vec<String> = self.columns.iter().map(|c| c.to_sql(grammar)).collect();
            let sql = format!("CREATE TABLE IF NOT EXISTS {} (\n  {}\n)", self.table, col_defs.join(",\n  "));
            statements.push(sql);
        } else {
            for col in &self.columns {
                statements.push(format!("ALTER TABLE {} ADD COLUMN {}", self.table, col.to_sql(grammar)));
            }
            for col in &self.drop_columns {
                statements.push(format!("ALTER TABLE {} DROP COLUMN {}", self.table, col));
            }
        }

        for idx in &self.indexes {
            let unique = if idx.unique { "UNIQUE " } else { "" };
            let idx_name = idx.name.clone().unwrap_or_else(|| format!("idx_{}_{}", self.table, idx.columns.join("_")));
            statements.push(format!("CREATE {}INDEX IF NOT EXISTS {} ON {} ({})", unique, idx_name, self.table, idx.columns.join(", ")));
        }

        statements
    }
}
