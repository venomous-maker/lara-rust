/// Converts QueryBuilder where clauses into a MongoDB-style JSON filter.
/// The MongoDriver deserializes this into a `bson::Document`.
use serde_json::{json, Value as JsonValue};
use crate::value::Value;
use super::where_clause::{Boolean, WhereClause};

/// Build a MongoDB `$and`-combined filter document from a slice of WhereClause.
pub fn clauses_to_filter(clauses: &[WhereClause]) -> JsonValue {
    if clauses.is_empty() {
        return json!({});
    }

    // Separate AND and OR groups
    let mut and_parts: Vec<JsonValue> = Vec::new();
    let mut or_parts: Vec<JsonValue> = Vec::new();

    for clause in clauses {
        let expr = clause_to_expr(clause);
        match clause.boolean() {
            Boolean::And => and_parts.push(expr),
            Boolean::Or  => or_parts.push(expr),
        }
    }

    match (and_parts.len(), or_parts.len()) {
        (0, 0) => json!({}),
        (_, 0) => {
            if and_parts.len() == 1 { and_parts.remove(0) }
            else { json!({ "$and": and_parts }) }
        }
        (0, _) => json!({ "$or": or_parts }),
        _ => json!({ "$and": [{ "$and": and_parts }, { "$or": or_parts }] }),
    }
}

fn clause_to_expr(clause: &WhereClause) -> JsonValue {
    match clause {
        WhereClause::Basic { column, op, value, .. } => {
            let v = value_to_json(value);
            match op.as_str() {
                "="  => json!({ column: v }),
                "!=" => json!({ column: { "$ne": v } }),
                ">"  => json!({ column: { "$gt": v } }),
                ">=" => json!({ column: { "$gte": v } }),
                "<"  => json!({ column: { "$lt": v } }),
                "<=" => json!({ column: { "$lte": v } }),
                "LIKE" | "like" => {
                    // Convert SQL % wildcards to regex
                    let pattern = v.as_str().unwrap_or("").replace('%', ".*").replace('_', ".");
                    json!({ column: { "$regex": pattern, "$options": "i" } })
                }
                "NOT LIKE" => {
                    let pattern = v.as_str().unwrap_or("").replace('%', ".*").replace('_', ".");
                    json!({ column: { "$not": { "$regex": pattern } } })
                }
                _ => json!({ column: v }),
            }
        }

        WhereClause::In { column, values, negated, .. } => {
            let arr: Vec<JsonValue> = values.iter().map(value_to_json).collect();
            if *negated {
                json!({ column: { "$nin": arr } })
            } else {
                json!({ column: { "$in": arr } })
            }
        }

        WhereClause::Between { column, min, max, negated, .. } => {
            let lo = value_to_json(min);
            let hi = value_to_json(max);
            if *negated {
                json!({ "$or": [{ column: { "$lt": lo } }, { column: { "$gt": hi } }] })
            } else {
                json!({ column: { "$gte": lo, "$lte": hi } })
            }
        }

        WhereClause::Null { column, not_null, .. } => {
            if *not_null {
                json!({ column: { "$ne": null } })
            } else {
                json!({ column: { "$eq": null } })
            }
        }

        WhereClause::Raw { sql, .. } => {
            // Raw SQL can't be auto-converted; pass as a comment-like field
            json!({ "__raw": sql })
        }

        WhereClause::Nested { clauses, .. } => {
            clauses_to_filter(clauses)
        }
    }
}

fn value_to_json(v: &Value) -> JsonValue {
    match v {
        Value::Null      => JsonValue::Null,
        Value::Bool(b)   => JsonValue::Bool(*b),
        Value::Int(n)    => JsonValue::Number((*n).into()),
        Value::Float(f)  => serde_json::Number::from_f64(*f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        Value::Text(s)   => JsonValue::String(s.clone()),
        Value::Bytes(b)  => JsonValue::String(base64(b)),
        Value::Json(v)   => v.clone(),
    }
}

fn base64(data: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };
        out.push(T[(b0 >> 2)] as char);
        out.push(T[((b0 & 3) << 4) | (b1 >> 4)] as char);
        out.push(if chunk.len() > 1 { T[((b1 & 0xf) << 2) | (b2 >> 6)] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[b2 & 0x3f] as char } else { '=' });
    }
    out
}
