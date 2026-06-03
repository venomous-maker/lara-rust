/// JOIN type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Cross,
}

impl JoinType {
    pub fn sql_keyword(self) -> &'static str {
        match self {
            JoinType::Inner => "INNER JOIN",
            JoinType::Left  => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Cross => "CROSS JOIN",
        }
    }
}

/// A single JOIN clause.
#[derive(Debug, Clone)]
pub struct Join {
    pub join_type: JoinType,
    pub table: String,
    pub on_local: String,
    pub on_operator: String,
    pub on_foreign: String,
}

impl Join {
    pub fn new(
        join_type: JoinType,
        table: &str,
        on_local: &str,
        on_operator: &str,
        on_foreign: &str,
    ) -> Self {
        Self {
            join_type,
            table: table.to_string(),
            on_local: on_local.to_string(),
            on_operator: on_operator.to_string(),
            on_foreign: on_foreign.to_string(),
        }
    }
}
