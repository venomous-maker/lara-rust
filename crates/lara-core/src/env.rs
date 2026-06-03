//! Environment helpers — a Laravel-style `env()` that reads from the process
//! environment first, then falls back to a `.env` file.
//!
//! The `.env` file is parsed once, lazily, on the first lookup. By default the
//! file `.env` in the current working directory is used; set `LARA_ENV_FILE` to
//! point elsewhere, or call [`load_from`] *before* the first `env()` call.
//!
//! Process environment variables always take precedence over `.env` entries, so
//! deployments can override file defaults without editing the file.
//!
//! ```no_run
//! use lara_core::env;
//!
//! let debug: bool = env::env_bool("APP_DEBUG", false);
//! let port: u16   = env::env_or_parse("APP_PORT", 8080);
//! let name        = env::env_or("APP_NAME", "Lara");
//! ```

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::OnceLock;

static DOTENV: OnceLock<HashMap<String, String>> = OnceLock::new();

fn dotenv() -> &'static HashMap<String, String> {
    DOTENV.get_or_init(|| {
        let path = std::env::var("LARA_ENV_FILE").unwrap_or_else(|_| ".env".to_string());
        parse_env_file(&path).unwrap_or_default()
    })
}

/// Look up `key` in the process environment, falling back to the `.env` file.
/// Returns `None` if the key is set nowhere.
pub fn env(key: &str) -> Option<String> {
    match std::env::var(key) {
        Ok(v) => Some(v),
        Err(_) => dotenv().get(key).cloned(),
    }
}

/// Like [`env`], but returns `default` when the key is missing.
pub fn env_or(key: &str, default: impl Into<String>) -> String {
    env(key).unwrap_or_else(|| default.into())
}

/// Look up `key` and parse it into `T`. Returns `None` if missing or unparseable.
pub fn env_parse<T: FromStr>(key: &str) -> Option<T> {
    env(key).and_then(|v| v.trim().parse().ok())
}

/// Look up and parse `key` into `T`, falling back to `default`.
pub fn env_or_parse<T: FromStr>(key: &str, default: T) -> T {
    env_parse(key).unwrap_or(default)
}

/// Interpret `key` as a boolean. Truthy values: `1`, `true`, `yes`, `on`
/// (case-insensitive). Any other present value is `false`; a missing key
/// yields `default`.
pub fn env_bool(key: &str, default: bool) -> bool {
    match env(key) {
        Some(v) => matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        None => default,
    }
}

/// Eagerly trigger loading of the default `.env` file (`.env`, or `LARA_ENV_FILE`).
/// Optional — lookups load it lazily anyway.
pub fn load() {
    let _ = dotenv();
}

/// Load a specific `.env` file. Must be called **before** the first `env()`
/// lookup; returns `false` if the store was already initialized.
pub fn load_from(path: &str) -> bool {
    let map = parse_env_file(path).unwrap_or_default();
    DOTENV.set(map).is_ok()
}

// ── .env parsing ─────────────────────────────────────────────────────────────

fn parse_env_file(path: &str) -> Option<HashMap<String, String>> {
    let content = std::fs::read_to_string(path).ok()?;
    Some(parse_env_str(&content))
}

/// Parse the contents of a `.env`-style document into a key→value map.
///
/// Supports `#` comments, blank lines, an optional `export ` prefix, single- and
/// double-quoted values (with `\n`, `\t`, `\"`, `\\` escapes inside double
/// quotes), and trailing ` #` comments on unquoted values.
pub fn parse_env_str(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((key, raw)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        map.insert(key.to_string(), parse_value(raw.trim()));
    }
    map
}

fn parse_value(raw: &str) -> String {
    if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
        return unescape(&raw[1..raw.len() - 1]);
    }
    if raw.len() >= 2 && raw.starts_with('\'') && raw.ends_with('\'') {
        return raw[1..raw.len() - 1].to_string();
    }
    // Unquoted: strip a trailing inline comment introduced by " #".
    match raw.find(" #") {
        Some(i) => raw[..i].trim().to_string(),
        None => raw.to_string(),
    }
}

fn unescape(s: &str) -> String {
    s.replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_pairs_and_comments() {
        let map = parse_env_str(
            "# comment\nAPP_NAME=Lara\nexport APP_ENV=local\n\nDB_PORT=5432 # inline\n",
        );
        assert_eq!(map.get("APP_NAME").map(String::as_str), Some("Lara"));
        assert_eq!(map.get("APP_ENV").map(String::as_str), Some("local"));
        assert_eq!(map.get("DB_PORT").map(String::as_str), Some("5432"));
    }

    #[test]
    fn parses_quoted_values() {
        let map = parse_env_str("GREETING=\"hello world\"\nPATH='a:b:c'\nESC=\"line1\\nline2\"");
        assert_eq!(map.get("GREETING").map(String::as_str), Some("hello world"));
        assert_eq!(map.get("PATH").map(String::as_str), Some("a:b:c"));
        assert_eq!(map.get("ESC").map(String::as_str), Some("line1\nline2"));
    }

    #[test]
    fn process_env_takes_precedence() {
        // Use a unique key so this test is order-independent.
        std::env::set_var("LARA_ENV_TEST_PRECEDENCE", "from_env");
        assert_eq!(env("LARA_ENV_TEST_PRECEDENCE").as_deref(), Some("from_env"));
    }

    #[test]
    fn env_bool_truthiness() {
        std::env::set_var("LARA_ENV_TEST_BOOL_T", "TRUE");
        std::env::set_var("LARA_ENV_TEST_BOOL_F", "nope");
        assert!(env_bool("LARA_ENV_TEST_BOOL_T", false));
        assert!(!env_bool("LARA_ENV_TEST_BOOL_F", true));
        assert!(env_bool("LARA_ENV_TEST_BOOL_MISSING", true));
    }
}
