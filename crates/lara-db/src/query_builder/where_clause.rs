use crate::value::Value;

/// Boolean connector between clauses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Boolean {
    And,
    Or,
}

/// A single WHERE condition.
#[derive(Debug, Clone)]
pub enum WhereClause {
    Basic {
        column: String,
        op: String,
        value: Value,
        boolean: Boolean,
    },
    In {
        column: String,
        values: Vec<Value>,
        negated: bool,
        boolean: Boolean,
    },
    Between {
        column: String,
        min: Value,
        max: Value,
        negated: bool,
        boolean: Boolean,
    },
    Null {
        column: String,
        not_null: bool,
        boolean: Boolean,
    },
    Raw {
        sql: String,
        params: Vec<Value>,
        boolean: Boolean,
    },
    /// Nested group:  AND (a = 1 OR b = 2)
    Nested {
        clauses: Vec<WhereClause>,
        boolean: Boolean,
    },
}

impl WhereClause {
    pub fn boolean(&self) -> Boolean {
        match self {
            Self::Basic { boolean, .. } => *boolean,
            Self::In { boolean, .. } => *boolean,
            Self::Between { boolean, .. } => *boolean,
            Self::Null { boolean, .. } => *boolean,
            Self::Raw { boolean, .. } => *boolean,
            Self::Nested { boolean, .. } => *boolean,
        }
    }
}

/// ORDER BY direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Order {
    Asc,
    Desc,
}

impl Order {
    pub fn as_str(self) -> &'static str {
        match self {
            Order::Asc => "ASC",
            Order::Desc => "DESC",
        }
    }
}

/// A single ORDER BY clause.
#[derive(Debug, Clone)]
pub enum OrderByClause {
    Column { column: String, order: Order },
    Raw(String),
}

/// A single HAVING clause.
#[derive(Debug, Clone)]
pub enum HavingClause {
    Basic { column: String, op: String, value: Value },
    Raw(String),
}
