pub mod error;
pub mod rules;

use serde_json::Value;
use std::collections::HashMap;
use error::{ValidateResult, ValidationErrors};
use rules::RuleFn;

pub use error::ValidationErrors as Errors;
pub use error::ValidatorError;

// ── Rule enum ─────────────────────────────────────────────────────────────────

/// Built-in rules that can be declared by name + params.
pub enum Rule {
    Required,
    Email,
    Numeric,
    Integer,
    Boolean,
    Url,
    Uuid,
    Ip,
    Date,
    Confirmed,
    Min(f64),
    Max(f64),
    Between(f64, f64),
    MinLength(usize),
    MaxLength(usize),
    BetweenLength(usize, usize),
    In(Vec<String>),
    NotIn(Vec<String>),
    Regex(String),
    StartsWith(String),
    EndsWith(String),
    Contains(String),
    Same(String),
    Different(String),
    RequiredIf(String, String),
    ProhibitedIf(String, String),
    Nullable,
    Sometimes,
    /// Custom rule — a closure that returns `Some(error_msg)` on failure.
    Custom(std::sync::Arc<dyn Fn(&str, &Value, &serde_json::Map<String, Value>) -> Option<String> + Send + Sync>),
}

impl std::fmt::Debug for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom(_) => write!(f, "Rule::Custom(...)"),
            _ => write!(f, "Rule::{}", self.name()),
        }
    }
}

impl Rule {
    fn name(&self) -> &'static str {
        match self {
            Self::Required => "Required",
            Self::Email => "Email",
            Self::Numeric => "Numeric",
            Self::Integer => "Integer",
            Self::Boolean => "Boolean",
            Self::Url => "Url",
            Self::Uuid => "Uuid",
            Self::Ip => "Ip",
            Self::Date => "Date",
            Self::Confirmed => "Confirmed",
            Self::Min(_) => "Min",
            Self::Max(_) => "Max",
            Self::Between(_, _) => "Between",
            Self::MinLength(_) => "MinLength",
            Self::MaxLength(_) => "MaxLength",
            Self::BetweenLength(_, _) => "BetweenLength",
            Self::In(_) => "In",
            Self::NotIn(_) => "NotIn",
            Self::Regex(_) => "Regex",
            Self::StartsWith(_) => "StartsWith",
            Self::EndsWith(_) => "EndsWith",
            Self::Contains(_) => "Contains",
            Self::Same(_) => "Same",
            Self::Different(_) => "Different",
            Self::RequiredIf(_, _) => "RequiredIf",
            Self::ProhibitedIf(_, _) => "ProhibitedIf",
            Self::Nullable => "Nullable",
            Self::Sometimes => "Sometimes",
            Self::Custom(_) => "Custom",
        }
    }

    /// Convenience constructor for custom closures.
    pub fn custom<F>(f: F) -> Self
    where
        F: Fn(&str, &Value, &serde_json::Map<String, Value>) -> Option<String> + Send + Sync + 'static,
    {
        Self::Custom(std::sync::Arc::new(f))
    }
}

// ── Validator ─────────────────────────────────────────────────────────────────

/// Validates a `serde_json::Map` of input against a set of field rules.
pub struct Validator {
    rules: HashMap<String, Vec<Rule>>,
}

impl Validator {
    pub fn new() -> Self {
        Self { rules: HashMap::new() }
    }

    /// Add rules for a field.
    pub fn field(mut self, field: impl Into<String>, field_rules: Vec<Rule>) -> Self {
        self.rules.insert(field.into(), field_rules);
        self
    }

    /// Run validation.
    pub fn validate(&self, data: &serde_json::Map<String, Value>) -> ValidateResult<()> {
        let mut errors = ValidationErrors::new();

        for (field, field_rules) in &self.rules {
            let value = data.get(field).cloned().unwrap_or(Value::Null);
            let mut nullable = false;
            let mut sometimes = false;

            for rule in field_rules {
                match rule {
                    Rule::Nullable  => nullable = true,
                    Rule::Sometimes => sometimes = true,
                    _ => {}
                }
            }

            if sometimes && value.is_null() { continue; }
            if nullable && value.is_null()  { continue; }

            for rule in field_rules {
                if let Some(m) = apply_rule(field, &value, data, rule) {
                    errors.add(field, m);
                }
            }
        }

        if errors.is_empty() { Ok(()) }
        else { Err(ValidatorError::ValidationFailed(errors)) }
    }
}

impl Default for Validator {
    fn default() -> Self { Self::new() }
}

fn apply_rule(
    field: &str,
    value: &Value,
    data: &serde_json::Map<String, Value>,
    rule: &Rule,
) -> Option<String> {
    match rule {
        Rule::Required         => rules::required(field, value, data),
        Rule::Email            => rules::email(field, value, data),
        Rule::Numeric          => rules::numeric(field, value, data),
        Rule::Integer          => rules::integer(field, value, data),
        Rule::Boolean          => rules::boolean_rule(field, value, data),
        Rule::Url              => rules::url_rule(field, value, data),
        Rule::Uuid             => rules::uuid_rule(field, value, data),
        Rule::Ip               => rules::ip_rule(field, value, data),
        Rule::Date             => rules::date_rule(field, value, data),
        Rule::Confirmed        => rules::confirmed(field, value, data),
        Rule::Min(n)           => rules::min_value(*n)(field, value, data),
        Rule::Max(n)           => rules::max_value(*n)(field, value, data),
        Rule::Between(a, b)    => rules::between(*a, *b)(field, value, data),
        Rule::MinLength(n)     => rules::min_length(*n)(field, value, data),
        Rule::MaxLength(n)     => rules::max_length(*n)(field, value, data),
        Rule::BetweenLength(a,b) => rules::min_length(*a)(field, value, data)
            .or_else(|| rules::max_length(*b)(field, value, data)),
        Rule::In(opts)         => rules::in_list(opts.clone())(field, value, data),
        Rule::NotIn(opts)      => rules::not_in(opts.clone())(field, value, data),
        Rule::Regex(pat)       => rules::regex_rule(pat)(field, value, data),
        Rule::StartsWith(p)    => rules::starts_with_rule(p)(field, value, data),
        Rule::EndsWith(s)      => rules::ends_with_rule(s)(field, value, data),
        Rule::Contains(n)      => rules::contains_rule(n)(field, value, data),
        Rule::Same(other)      => rules::same(other)(field, value, data),
        Rule::Different(other) => rules::different(other)(field, value, data),
        Rule::RequiredIf(of,ov)   => rules::required_if(of, ov)(field, value, data),
        Rule::ProhibitedIf(of,ov) => rules::prohibited_if(of, ov)(field, value, data),
        Rule::Custom(f)        => f(field, value, data),
        Rule::Nullable | Rule::Sometimes => None,
    }
}

/// Convenience: validate a JSON value directly.
pub fn validate(
    data: serde_json::Value,
    field_rules: HashMap<String, Vec<Rule>>,
) -> ValidateResult<()> {
    let map = match data {
        Value::Object(m) => m,
        _ => return Err(ValidatorError::Rule("Expected JSON object".into())),
    };
    let mut v = Validator::new();
    for (field, r) in field_rules { v = v.field(field, r); }
    v.validate(&map)
}
