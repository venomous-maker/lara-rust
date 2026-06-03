//! Minimal SQL → MongoDB translation layer.
//!
//! The query builder, schema builder, and migration runner all emit SQL through
//! the active [`Grammar`](super::Grammar). For the MongoDB driver we parse that
//! framework-generated SQL back into structured Mongo operations, so the exact
//! same higher-level code paths (schema, migrations, aggregates, raw queries)
//! work unchanged across every driver.
//!
//! Only the SQL shapes this framework actually produces are supported — this is
//! deliberately *not* a general-purpose SQL engine. Mongo-native grammar emits
//! `?` placeholders (see `Grammar::Mongodb::placeholder`), so values are pulled
//! positionally from `CompiledQuery::params`.

use serde_json::{json, Map, Value as JsonValue};

use crate::error::{DbError, Result};
use crate::value::Value;

// ── Public IR ───────────────────────────────────────────────────────────────

/// Aggregate function extracted from a `SELECT`.
#[derive(Debug, Clone)]
pub enum Agg {
    Count,
    Sum(String),
    Avg(String),
    Min(String),
    Max(String),
}

#[derive(Debug, Clone, Default)]
pub struct SelectOp {
    pub collection: String,
    pub filter: JsonValue,
    pub projection: Option<JsonValue>,
    pub sort: Option<JsonValue>,
    pub limit: Option<i64>,
    pub skip: Option<u64>,
    /// `SELECT DISTINCT <col>` — single-column distinct only.
    pub distinct: Option<String>,
    /// `(aggregate, output alias)` — e.g. `(Count, "cnt")`.
    pub aggregate: Option<(Agg, String)>,
}

/// A structured Mongo operation parsed from a SQL string.
#[derive(Debug, Clone)]
pub enum MongoOp {
    Select(SelectOp),
    Insert { collection: String, doc: JsonValue },
    Update { collection: String, filter: JsonValue, set: JsonValue },
    Delete { collection: String, filter: JsonValue },
    CreateCollection { name: String },
    DropCollection { name: String },
    RenameCollection { from: String, to: String },
    CreateIndex { collection: String, columns: Vec<String>, unique: bool },
    /// DDL that has no analogue in a schemaless store (e.g. `ALTER TABLE … ADD COLUMN`).
    Noop,
}

/// Parse a framework-generated SQL string + positional params into a [`MongoOp`].
pub fn parse(sql: &str, params: &[Value]) -> Result<MongoOp> {
    let toks = tokenize(sql)?;
    let mut p = Parser { toks, pos: 0, params: params.iter() };
    let op = match p.peek_word().as_deref() {
        Some("SELECT") => MongoOp::Select(p.parse_select()?),
        Some("INSERT") => p.parse_insert()?,
        Some("UPDATE") => p.parse_update()?,
        Some("DELETE") => p.parse_delete()?,
        Some("CREATE") => p.parse_create()?,
        Some("DROP")   => p.parse_drop()?,
        Some("ALTER")  => p.parse_alter()?,
        _ => return Err(perr(format!("cannot translate SQL to MongoDB: `{}`", sql))),
    };
    Ok(op)
}

fn perr(msg: impl Into<String>) -> DbError {
    DbError::UnsupportedOperation(format!("Mongo SQL translation: {}", msg.into()))
}

// ── Tokenizer ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Word(String),
    Placeholder,
    Str(String),
    Num(f64),
    Op(String), // = != > >= < <=
    LParen,
    RParen,
    Comma,
}

fn tokenize(sql: &str) -> Result<Vec<Tok>> {
    let chars: Vec<char> = sql.chars().collect();
    let mut toks = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        match c {
            '?' => { toks.push(Tok::Placeholder); i += 1; }
            '(' => { toks.push(Tok::LParen); i += 1; }
            ')' => { toks.push(Tok::RParen); i += 1; }
            ',' => { toks.push(Tok::Comma); i += 1; }
            '*' => { toks.push(Tok::Word("*".into())); i += 1; }
            '=' => { toks.push(Tok::Op("=".into())); i += 1; }
            '\'' => {
                i += 1;
                let mut s = String::new();
                while i < chars.len() {
                    if chars[i] == '\'' {
                        // `''` is an escaped single quote.
                        if i + 1 < chars.len() && chars[i + 1] == '\'' {
                            s.push('\'');
                            i += 2;
                            continue;
                        }
                        i += 1;
                        break;
                    }
                    s.push(chars[i]);
                    i += 1;
                }
                toks.push(Tok::Str(s));
            }
            '!' => {
                if chars.get(i + 1) == Some(&'=') {
                    toks.push(Tok::Op("!=".into()));
                    i += 2;
                } else {
                    return Err(perr("unexpected `!`"));
                }
            }
            '<' => match chars.get(i + 1) {
                Some('=') => { toks.push(Tok::Op("<=".into())); i += 2; }
                Some('>') => { toks.push(Tok::Op("!=".into())); i += 2; }
                _ => { toks.push(Tok::Op("<".into())); i += 1; }
            },
            '>' => {
                if chars.get(i + 1) == Some(&'=') {
                    toks.push(Tok::Op(">=".into()));
                    i += 2;
                } else {
                    toks.push(Tok::Op(">".into()));
                    i += 1;
                }
            }
            c if c.is_ascii_digit()
                || (c == '-' && chars.get(i + 1).is_some_and(|n| n.is_ascii_digit())) =>
            {
                let mut s = String::new();
                if c == '-' {
                    s.push('-');
                    i += 1;
                }
                let mut seen_dot = false;
                while i < chars.len()
                    && (chars[i].is_ascii_digit() || (chars[i] == '.' && !seen_dot))
                {
                    if chars[i] == '.' {
                        seen_dot = true;
                    }
                    s.push(chars[i]);
                    i += 1;
                }
                let n: f64 = s.parse().map_err(|_| perr(format!("bad number `{}`", s)))?;
                toks.push(Tok::Num(n));
            }
            c if c.is_alphanumeric() || c == '_' || c == '.' => {
                let mut s = String::new();
                while i < chars.len()
                    && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '.')
                {
                    s.push(chars[i]);
                    i += 1;
                }
                toks.push(Tok::Word(s));
            }
            other => return Err(perr(format!("unexpected character `{}`", other))),
        }
    }
    Ok(toks)
}

// ── Parser ───────────────────────────────────────────────────────────────────

struct Parser<'a> {
    toks: Vec<Tok>,
    pos: usize,
    params: std::slice::Iter<'a, Value>,
}

impl<'a> Parser<'a> {
    // ── token helpers ─────────────────────────────────────────────────────────

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn at_end(&self) -> bool {
        self.pos >= self.toks.len()
    }

    fn is_word(&self, kw: &str) -> bool {
        matches!(self.peek(), Some(Tok::Word(w)) if w.eq_ignore_ascii_case(kw))
    }

    fn eat_word(&mut self, kw: &str) -> bool {
        if self.is_word(kw) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_word(&mut self, kw: &str) -> Result<()> {
        if self.eat_word(kw) {
            Ok(())
        } else {
            Err(perr(format!("expected `{}`", kw)))
        }
    }

    fn peek_word(&self) -> Option<String> {
        match self.peek() {
            Some(Tok::Word(w)) => Some(w.to_ascii_uppercase()),
            _ => None,
        }
    }

    fn expect_ident(&mut self) -> Result<String> {
        match self.peek().cloned() {
            Some(Tok::Word(w)) => {
                self.pos += 1;
                Ok(w)
            }
            _ => Err(perr("expected identifier")),
        }
    }

    fn eat_tok(&mut self, t: &Tok) -> bool {
        if self.peek() == Some(t) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_tok(&mut self, t: &Tok) -> Result<()> {
        if self.eat_tok(t) {
            Ok(())
        } else {
            Err(perr(format!("expected `{:?}`", t)))
        }
    }

    fn expect_number(&mut self) -> Result<f64> {
        match self.peek().cloned() {
            Some(Tok::Num(n)) => {
                self.pos += 1;
                Ok(n)
            }
            _ => Err(perr("expected number")),
        }
    }

    fn next_param(&mut self) -> Result<JsonValue> {
        let v = self.params.next().ok_or_else(|| perr("not enough bound parameters"))?;
        Ok(JsonValue::from(v.clone()))
    }

    /// Skip a balanced `( … )` group (used for `CREATE TABLE` column defs).
    fn skip_balanced_parens(&mut self) {
        if !self.eat_tok(&Tok::LParen) {
            return;
        }
        let mut depth = 1;
        while depth > 0 && !self.at_end() {
            match self.toks[self.pos] {
                Tok::LParen => depth += 1,
                Tok::RParen => depth -= 1,
                _ => {}
            }
            self.pos += 1;
        }
    }

    // ── value / column parsing ────────────────────────────────────────────────

    fn parse_value(&mut self) -> Result<JsonValue> {
        match self.peek().cloned() {
            Some(Tok::Placeholder) => {
                self.pos += 1;
                self.next_param()
            }
            Some(Tok::Str(s)) => {
                self.pos += 1;
                Ok(JsonValue::String(s))
            }
            Some(Tok::Num(n)) => {
                self.pos += 1;
                Ok(num_to_json(n))
            }
            Some(Tok::Word(w)) => {
                self.pos += 1;
                Ok(match w.to_ascii_uppercase().as_str() {
                    "NULL" => JsonValue::Null,
                    "TRUE" => JsonValue::Bool(true),
                    "FALSE" => JsonValue::Bool(false),
                    _ => JsonValue::String(w),
                })
            }
            _ => Err(perr("expected a value")),
        }
    }

    // ── SELECT ────────────────────────────────────────────────────────────────

    fn parse_select(&mut self) -> Result<SelectOp> {
        self.expect_word("SELECT")?;
        let distinct = self.eat_word("DISTINCT");

        let mut aggregate = None;
        let mut columns: Vec<String> = Vec::new();
        let mut star = false;

        // Projection list, up to FROM.
        while !self.is_word("FROM") && !self.at_end() {
            if let Some(agg) = self.try_parse_aggregate()? {
                aggregate = Some(agg);
            } else if self.is_word("*") {
                star = true;
                self.pos += 1;
            } else if matches!(self.peek(), Some(Tok::Num(_))) {
                // `SELECT 1` probe — treat as star, optional alias.
                star = true;
                self.pos += 1;
                self.parse_optional_alias();
            } else {
                let col = self.expect_ident()?;
                columns.push(col);
                self.parse_optional_alias();
            }
            if !self.eat_tok(&Tok::Comma) {
                break;
            }
        }

        self.expect_word("FROM")?;
        let collection = self.expect_ident()?;

        if self.is_word("INNER")
            || self.is_word("LEFT")
            || self.is_word("RIGHT")
            || self.is_word("CROSS")
            || self.is_word("JOIN")
        {
            return Err(perr("MongoDB does not support SQL JOINs"));
        }

        let filter = if self.eat_word("WHERE") {
            self.parse_expr()?
        } else {
            json!({})
        };

        // GROUP BY / HAVING are consumed (to keep param ordering correct) but
        // only meaningful for global aggregates, which we already handle.
        if self.eat_word("GROUP") {
            self.expect_word("BY")?;
            loop {
                let _ = self.expect_ident()?;
                if !self.eat_tok(&Tok::Comma) {
                    break;
                }
            }
        }
        if self.eat_word("HAVING") {
            let _ = self.parse_expr()?;
        }

        let sort = if self.eat_word("ORDER") {
            self.expect_word("BY")?;
            let mut sort_doc = Map::new();
            loop {
                let col = self.expect_ident()?;
                let dir = if self.eat_word("DESC") {
                    -1
                } else {
                    self.eat_word("ASC");
                    1
                };
                sort_doc.insert(col, json!(dir));
                if !self.eat_tok(&Tok::Comma) {
                    break;
                }
            }
            Some(JsonValue::Object(sort_doc))
        } else {
            None
        };

        let mut limit = None;
        let mut skip = None;
        loop {
            if self.eat_word("LIMIT") {
                limit = Some(self.expect_number()? as i64);
            } else if self.eat_word("OFFSET") {
                skip = Some(self.expect_number()? as u64);
            } else {
                break;
            }
        }

        let projection = if !star && aggregate.is_none() && !columns.is_empty() {
            let mut proj = Map::new();
            for c in &columns {
                proj.insert(c.clone(), json!(1));
            }
            Some(JsonValue::Object(proj))
        } else {
            None
        };

        let distinct_col = if distinct && columns.len() == 1 {
            Some(columns[0].clone())
        } else {
            None
        };

        Ok(SelectOp {
            collection,
            filter,
            projection,
            sort,
            limit,
            skip,
            distinct: distinct_col,
            aggregate,
        })
    }

    fn try_parse_aggregate(&mut self) -> Result<Option<(Agg, String)>> {
        let func = match self.peek_word() {
            Some(w) if matches!(w.as_str(), "COUNT" | "SUM" | "AVG" | "MIN" | "MAX") => w,
            _ => return Ok(None),
        };
        if !matches!(self.toks.get(self.pos + 1), Some(Tok::LParen)) {
            return Ok(None);
        }
        self.pos += 2; // function name + '('
        let arg = if self.eat_word("*") {
            "*".to_string()
        } else {
            self.expect_ident()?
        };
        self.expect_tok(&Tok::RParen)?;

        let agg = match func.as_str() {
            "COUNT" => Agg::Count,
            "SUM" => Agg::Sum(arg),
            "AVG" => Agg::Avg(arg),
            "MIN" => Agg::Min(arg),
            "MAX" => Agg::Max(arg),
            _ => unreachable!(),
        };
        let alias = self.parse_optional_alias().unwrap_or_else(|| "__agg".to_string());
        Ok(Some((agg, alias)))
    }

    fn parse_optional_alias(&mut self) -> Option<String> {
        if self.eat_word("AS") {
            if let Some(Tok::Word(w)) = self.peek().cloned() {
                self.pos += 1;
                return Some(w);
            }
        }
        None
    }

    // ── INSERT / UPDATE / DELETE ──────────────────────────────────────────────

    fn parse_insert(&mut self) -> Result<MongoOp> {
        self.expect_word("INSERT")?;
        self.expect_word("INTO")?;
        let collection = self.expect_ident()?;

        self.expect_tok(&Tok::LParen)?;
        let mut cols = Vec::new();
        loop {
            cols.push(self.expect_ident()?);
            if !self.eat_tok(&Tok::Comma) {
                break;
            }
        }
        self.expect_tok(&Tok::RParen)?;

        self.expect_word("VALUES")?;
        self.expect_tok(&Tok::LParen)?;
        let mut doc = Map::new();
        for (idx, col) in cols.iter().enumerate() {
            if idx > 0 {
                self.expect_tok(&Tok::Comma)?;
            }
            let v = self.parse_value()?;
            doc.insert(col.clone(), v);
        }
        self.expect_tok(&Tok::RParen)?;
        // A trailing `RETURNING *` (Postgres grammar) is irrelevant here.

        Ok(MongoOp::Insert {
            collection,
            doc: JsonValue::Object(doc),
        })
    }

    fn parse_update(&mut self) -> Result<MongoOp> {
        self.expect_word("UPDATE")?;
        let collection = self.expect_ident()?;
        self.expect_word("SET")?;

        let mut set = Map::new();
        loop {
            let col = self.expect_ident()?;
            self.expect_tok(&Tok::Op("=".into()))?;
            let v = self.parse_value()?;
            set.insert(col, v);
            if !self.eat_tok(&Tok::Comma) {
                break;
            }
        }

        let filter = if self.eat_word("WHERE") {
            self.parse_expr()?
        } else {
            json!({})
        };

        Ok(MongoOp::Update {
            collection,
            filter,
            set: JsonValue::Object(set),
        })
    }

    fn parse_delete(&mut self) -> Result<MongoOp> {
        self.expect_word("DELETE")?;
        self.expect_word("FROM")?;
        let collection = self.expect_ident()?;
        let filter = if self.eat_word("WHERE") {
            self.parse_expr()?
        } else {
            json!({})
        };
        Ok(MongoOp::Delete { collection, filter })
    }

    // ── DDL ───────────────────────────────────────────────────────────────────

    fn parse_create(&mut self) -> Result<MongoOp> {
        self.expect_word("CREATE")?;
        let unique = self.eat_word("UNIQUE");

        if self.eat_word("TABLE") {
            self.eat_if_not_exists();
            let name = self.expect_ident()?;
            self.skip_balanced_parens();
            Ok(MongoOp::CreateCollection { name })
        } else if self.eat_word("INDEX") {
            self.eat_if_not_exists();
            let _index_name = self.expect_ident()?;
            self.expect_word("ON")?;
            let collection = self.expect_ident()?;
            self.expect_tok(&Tok::LParen)?;
            let mut columns = Vec::new();
            loop {
                columns.push(self.expect_ident()?);
                if !self.eat_tok(&Tok::Comma) {
                    break;
                }
            }
            self.expect_tok(&Tok::RParen)?;
            Ok(MongoOp::CreateIndex {
                collection,
                columns,
                unique,
            })
        } else {
            Err(perr("unsupported CREATE statement"))
        }
    }

    fn parse_drop(&mut self) -> Result<MongoOp> {
        self.expect_word("DROP")?;
        if self.eat_word("TABLE") {
            self.eat_if_exists();
            let name = self.expect_ident()?;
            Ok(MongoOp::DropCollection { name })
        } else if self.eat_word("INDEX") {
            // Index drops are best-effort no-ops for the schemaless store.
            Ok(MongoOp::Noop)
        } else {
            Err(perr("unsupported DROP statement"))
        }
    }

    fn parse_alter(&mut self) -> Result<MongoOp> {
        self.expect_word("ALTER")?;
        self.expect_word("TABLE")?;
        let name = self.expect_ident()?;
        if self.eat_word("RENAME") {
            self.expect_word("TO")?;
            let to = self.expect_ident()?;
            Ok(MongoOp::RenameCollection { from: name, to })
        } else {
            // ADD COLUMN / DROP COLUMN / MODIFY — nothing to do in a schemaless store.
            Ok(MongoOp::Noop)
        }
    }

    fn eat_if_not_exists(&mut self) {
        if self.eat_word("IF") {
            self.eat_word("NOT");
            self.eat_word("EXISTS");
        }
    }

    fn eat_if_exists(&mut self) {
        if self.eat_word("IF") {
            self.eat_word("EXISTS");
        }
    }

    // ── WHERE expression ──────────────────────────────────────────────────────

    fn parse_expr(&mut self) -> Result<JsonValue> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<JsonValue> {
        let mut parts = vec![self.parse_and()?];
        while self.eat_word("OR") {
            parts.push(self.parse_and()?);
        }
        Ok(if parts.len() == 1 {
            parts.pop().unwrap()
        } else {
            json!({ "$or": parts })
        })
    }

    fn parse_and(&mut self) -> Result<JsonValue> {
        let mut parts = vec![self.parse_primary()?];
        while self.eat_word("AND") {
            parts.push(self.parse_primary()?);
        }
        Ok(if parts.len() == 1 {
            parts.pop().unwrap()
        } else {
            json!({ "$and": parts })
        })
    }

    fn parse_primary(&mut self) -> Result<JsonValue> {
        if self.eat_tok(&Tok::LParen) {
            let e = self.parse_or()?;
            self.expect_tok(&Tok::RParen)?;
            return Ok(e);
        }
        self.parse_predicate()
    }

    fn parse_predicate(&mut self) -> Result<JsonValue> {
        let col = self.expect_ident()?;

        // IS [NOT] NULL
        if self.eat_word("IS") {
            let not = self.eat_word("NOT");
            self.expect_word("NULL")?;
            return Ok(if not {
                json!({ col: { "$ne": null } })
            } else {
                json!({ col: { "$eq": null } })
            });
        }

        let negated = self.eat_word("NOT");

        // [NOT] IN ( … )
        if self.eat_word("IN") {
            self.expect_tok(&Tok::LParen)?;
            let mut arr = Vec::new();
            if self.peek() != Some(&Tok::RParen) {
                loop {
                    arr.push(self.parse_value()?);
                    if !self.eat_tok(&Tok::Comma) {
                        break;
                    }
                }
            }
            self.expect_tok(&Tok::RParen)?;
            return Ok(if negated {
                json!({ col: { "$nin": arr } })
            } else {
                json!({ col: { "$in": arr } })
            });
        }

        // [NOT] BETWEEN a AND b
        if self.eat_word("BETWEEN") {
            let lo = self.parse_value()?;
            self.expect_word("AND")?;
            let hi = self.parse_value()?;
            return Ok(if negated {
                json!({ "$or": [{ col.clone(): { "$lt": lo } }, { col: { "$gt": hi } }] })
            } else {
                json!({ col: { "$gte": lo, "$lte": hi } })
            });
        }

        // [NOT] LIKE
        if self.eat_word("LIKE") {
            let v = self.parse_value()?;
            let pattern = v.as_str().unwrap_or("").replace('%', ".*").replace('_', ".");
            return Ok(if negated {
                json!({ col: { "$not": { "$regex": pattern, "$options": "i" } } })
            } else {
                json!({ col: { "$regex": pattern, "$options": "i" } })
            });
        }

        // Comparison operator
        if let Some(Tok::Op(op)) = self.peek().cloned() {
            self.pos += 1;
            let v = self.parse_value()?;
            let expr = match op.as_str() {
                "=" => json!({ col: v }),
                "!=" => json!({ col: { "$ne": v } }),
                ">" => json!({ col: { "$gt": v } }),
                ">=" => json!({ col: { "$gte": v } }),
                "<" => json!({ col: { "$lt": v } }),
                "<=" => json!({ col: { "$lte": v } }),
                other => return Err(perr(format!("unknown operator `{}`", other))),
            };
            return Ok(expr);
        }

        Err(perr(format!("cannot parse predicate near `{}`", col)))
    }
}

fn num_to_json(n: f64) -> JsonValue {
    if n.fract() == 0.0 && n.abs() < 9.007e15 {
        JsonValue::Number((n as i64).into())
    } else {
        serde_json::Number::from_f64(n)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_select_with_where_and_order() {
        let op = parse(
            "SELECT * FROM users WHERE age >= ? AND name = ? ORDER BY name ASC LIMIT 10 OFFSET 5",
            &[Value::Int(18), Value::Text("ada".into())],
        )
        .unwrap();
        match op {
            MongoOp::Select(s) => {
                assert_eq!(s.collection, "users");
                assert_eq!(s.limit, Some(10));
                assert_eq!(s.skip, Some(5));
                assert_eq!(s.filter, json!({ "$and": [{ "age": { "$gte": 18 } }, { "name": "ada" }] }));
                assert_eq!(s.sort, Some(json!({ "name": 1 })));
            }
            _ => panic!("expected select"),
        }
    }

    #[test]
    fn parses_count_aggregate() {
        let op = parse(
            "SELECT COUNT(*) as cnt FROM migrations WHERE migration = ?",
            &[Value::Text("create_users".into())],
        )
        .unwrap();
        match op {
            MongoOp::Select(s) => {
                assert!(matches!(s.aggregate, Some((Agg::Count, ref a)) if a == "cnt"));
                assert_eq!(s.filter, json!({ "migration": "create_users" }));
            }
            _ => panic!("expected select"),
        }
    }

    #[test]
    fn parses_max_aggregate() {
        let op = parse("SELECT MAX(batch) as mb FROM migrations", &[]).unwrap();
        match op {
            MongoOp::Select(s) => {
                assert!(matches!(s.aggregate, Some((Agg::Max(ref c), ref a)) if c == "batch" && a == "mb"));
            }
            _ => panic!("expected select"),
        }
    }

    #[test]
    fn parses_insert() {
        let op = parse(
            "INSERT INTO migrations (migration, batch) VALUES (?, ?)",
            &[Value::Text("m1".into()), Value::Int(1)],
        )
        .unwrap();
        match op {
            MongoOp::Insert { collection, doc } => {
                assert_eq!(collection, "migrations");
                assert_eq!(doc, json!({ "migration": "m1", "batch": 1 }));
            }
            _ => panic!("expected insert"),
        }
    }

    #[test]
    fn parses_delete_with_filter() {
        let op = parse(
            "DELETE FROM migrations WHERE migration = ?",
            &[Value::Text("m1".into())],
        )
        .unwrap();
        match op {
            MongoOp::Delete { collection, filter } => {
                assert_eq!(collection, "migrations");
                assert_eq!(filter, json!({ "migration": "m1" }));
            }
            _ => panic!("expected delete"),
        }
    }

    #[test]
    fn parses_in_and_between() {
        let op = parse(
            "SELECT * FROM t WHERE id IN (?, ?, ?) AND age BETWEEN ? AND ?",
            &[
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(18),
                Value::Int(65),
            ],
        )
        .unwrap();
        match op {
            MongoOp::Select(s) => assert_eq!(
                s.filter,
                json!({ "$and": [
                    { "id": { "$in": [1, 2, 3] } },
                    { "age": { "$gte": 18, "$lte": 65 } }
                ] })
            ),
            _ => panic!("expected select"),
        }
    }

    #[test]
    fn parses_create_table_as_collection() {
        let op = parse(
            "CREATE TABLE IF NOT EXISTS users (\n  id BIGINT,\n  name VARCHAR(255)\n)",
            &[],
        )
        .unwrap();
        assert!(matches!(op, MongoOp::CreateCollection { name } if name == "users"));
    }

    #[test]
    fn parses_create_unique_index() {
        let op = parse(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users (email)",
            &[],
        )
        .unwrap();
        match op {
            MongoOp::CreateIndex { collection, columns, unique } => {
                assert_eq!(collection, "users");
                assert_eq!(columns, vec!["email".to_string()]);
                assert!(unique);
            }
            _ => panic!("expected create index"),
        }
    }

    #[test]
    fn alter_add_column_is_noop() {
        let op = parse("ALTER TABLE users ADD COLUMN nickname VARCHAR(50)", &[]).unwrap();
        assert!(matches!(op, MongoOp::Noop));
    }

    #[test]
    fn parses_nested_or_group() {
        let op = parse(
            "SELECT * FROM t WHERE status = ? AND (role = ? OR role = ?)",
            &[
                Value::Text("active".into()),
                Value::Text("admin".into()),
                Value::Text("owner".into()),
            ],
        )
        .unwrap();
        match op {
            MongoOp::Select(s) => assert_eq!(
                s.filter,
                json!({ "$and": [
                    { "status": "active" },
                    { "$or": [{ "role": "admin" }, { "role": "owner" }] }
                ] })
            ),
            _ => panic!("expected select"),
        }
    }
}
