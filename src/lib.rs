//! # typedenv
//!
//! Tiny, zero-dependency typed access to environment variables.
//!
//! * Parse into any `T: FromStr`.
//! * A set-but-invalid value is an **error**, never silently replaced by a default.
//! * Errors name the variable and say what was wrong.
//! * [`Loader`] collects *all* problems and reports them at once.
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # unsafe { std::env::set_var("TYPEDENV_DOC_PORT", "9000"); }
//! let port: u16 = typedenv::var_or("TYPEDENV_DOC_PORT", 8080)?;
//! assert_eq!(port, 9000);
//! # Ok(())
//! # }
//! ```
//!
//! Collecting every error in one go:
//!
//! ```
//! let mut cfg = typedenv::Loader::new();
//! let _port: u16 = cfg.var_or("TYPEDENV_DOC_NOPE_PORT", 8080);
//! let _db: String = cfg.var("TYPEDENV_DOC_NOPE_DB"); // missing -> recorded
//! assert!(cfg.finish().is_err());
//! ```

#![cfg_attr(not(test), forbid(unsafe_code))]
#![warn(missing_docs)]

use std::{env, error::Error, fmt, str::FromStr};

/// An error that occurred while reading or parsing an environment variable.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EnvError {
    /// The environment variable is not set.
    Missing {
        /// The name of the missing variable.
        name: String,
    },

    /// The environment variable's value is not valid Unicode.
    NotUnicode {
        /// The name of the variable.
        name: String,
    },

    /// The environment variable value exists but could not be parsed into the target type.
    Invalid {
        /// The name of the variable.
        name: String,
        /// The raw value of the variable.
        value: String,
        /// The reason why parsing or processing failed.
        reason: String,
    },
}

impl EnvError {
    /// The name of the variable this error is about.
    pub fn name(&self) -> &str {
        match self {
            EnvError::Missing { name }
            | EnvError::NotUnicode { name }
            | EnvError::Invalid { name, .. } => name,
        }
    }
}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvError::Missing { name } => {
                write!(f, "environment variable `{name}` is not set")
            }
            EnvError::NotUnicode { name } => {
                write!(f, "environment variable `{name}` is not valid unicode")
            }
            EnvError::Invalid { name, value, reason } => write!(
                f,
                "environment variable `{name}` has invalid value `{value}`: {reason}"
            ),
        }
    }
}

impl Error for EnvError {}

/// Several [`EnvError`]s collected by a [`Loader`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Errors(Vec<EnvError>);

impl Errors {
    /// Multiple [`EnvError`]s collected by a [`Loader`] as a slice.
    pub fn as_slice(&self) -> &[EnvError] {
        &self.0
    }

    /// Iterate over the collected [`EnvError`]s.
    pub fn iter(&self) -> std::slice::Iter<'_, EnvError> {
        self.0.iter()
    }

    /// Number of collected errors.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if there are no errors.
    ///
    /// Always `false` for errors returned by [`Loader::finish`].
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Consumes the container and returns the inner vector of [`EnvError`]s.
    pub fn into_vec(self) -> Vec<EnvError> {
        self.0
    }
}

impl std::ops::Deref for Errors {
    type Target = [EnvError];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for Errors {
    type Item = EnvError;
    type IntoIter = std::vec::IntoIter<EnvError>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Errors {
    type Item = &'a EnvError;
    type IntoIter = std::slice::Iter<'a, EnvError>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = self.0.len();
        write!(f, "{n} configuration error{}:", if n == 1 { "" } else { "s" })?;
        for e in &self.0 {
            write!(f, "\n - {e}")?;
        }
        Ok(())
    }
}

impl Error for Errors {}

fn parse_str<T>(s: &str) -> Result<T, String>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    s.parse::<T>().map_err(|e| e.to_string())
}

fn parse_bool(s: &str) -> Result<bool, String> {
    match s.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" => Ok(true),
        "0" | "false" | "no" | "n" | "off" => Ok(false),
        _ => Err("expected one of true/false/1/0/yes/no/y/n/on/off".to_string()),
    }
}

/// Reads and parses a variable. `Ok(None)` means "not set".
fn read<T>(
    name: &str,
    parse: impl FnOnce(&str) -> Result<T, String>,
) -> Result<Option<T>, EnvError> {
    match env::var(name) {
        Ok(raw) => match parse(raw.trim()) {
            Ok(v) => Ok(Some(v)),
            Err(reason) => Err(EnvError::Invalid {
                name: name.to_string(),
                value: raw,
                reason,
            }),
        },
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(EnvError::NotUnicode {
            name: name.to_string(),
        }),
    }
}

fn missing(name: &str) -> EnvError {
    EnvError::Missing {
        name: name.to_string(),
    }
}

/// Reads a required variable.
pub fn var<T>(name: &str) -> Result<T, EnvError>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    read(name, parse_str::<T>)?.ok_or_else(|| missing(name))
}

/// Reads a variable, using `default` only when it is **unset**.
///
/// A value that is set but malformed is still an error.
pub fn var_or<T>(name: &str, default: T) -> Result<T, EnvError>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    Ok(read(name, parse_str::<T>)?.unwrap_or(default))
}

/// Reads an optional variable: `Ok(None)` when unset.
pub fn var_opt<T>(name: &str) -> Result<Option<T>, EnvError>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    read(name, parse_str::<T>)
}

/// Reads a required boolean. Accepts `true/false/1/0/yes/no/y/n/on/off`
/// (case-insensitive).
pub fn var_bool(name: &str) -> Result<bool, EnvError> {
    read(name, parse_bool)?.ok_or_else(|| missing(name))
}

/// Reads a boolean, using `default` only when unset.
pub fn var_bool_or(name: &str, default: bool) -> Result<bool, EnvError> {
    Ok(read(name, parse_bool)?.unwrap_or(default))
}

// ---------------------------------------------------------------------------
// Loader: collect every problem, report once
// ---------------------------------------------------------------------------

/// Collects errors across many lookups so you can report them all at once.
///
/// Each method returns a usable placeholder on failure (the default, or
/// `T::default()`), records the error, and lets you write straight-line code.
/// Call [`Loader::finish`] once at the end and **do not use the values if it
/// returns `Err`**.
#[derive(Debug, Default)]
pub struct Loader {
    errors: Vec<EnvError>,
}

impl Loader {
    /// Creates an empty loader.
    pub fn new() -> Self {
        Self::default()
    }

    /// Required variable. Returns `T::default()` and records an error on failure.
    ///
    /// For types that do not implement [`Default`], consider [`Loader::var_or`]
    /// or [`Loader::var_opt`].
    pub fn var<T>(&mut self, name: &str) -> T
    where
        T: FromStr + Default,
        T::Err: fmt::Display,
    {
        match read(name, parse_str::<T>) {
            Ok(Some(v)) => v,
            Ok(None) => {
                self.errors.push(missing(name));
                T::default()
            }
            Err(e) => {
                self.errors.push(e);
                T::default()
            }
        }
    }

    /// Variable with a default used only when unset. Malformed values are
    /// recorded as errors (and `default` is returned as a placeholder).
    pub fn var_or<T>(&mut self, name: &str, default: T) -> T
    where
        T: FromStr,
        T::Err: fmt::Display,
    {
        match read(name, parse_str::<T>) {
            Ok(Some(v)) => v,
            Ok(None) => default,
            Err(e) => {
                self.errors.push(e);
                default
            }
        }
    }

    /// Optional variable. `None` when unset or when malformed (the latter is
    /// also recorded as an error).
    pub fn var_opt<T>(&mut self, name: &str) -> Option<T>
    where
        T: FromStr,
        T::Err: fmt::Display,
    {
        match read(name, parse_str::<T>) {
            Ok(v) => v,
            Err(e) => {
                self.errors.push(e);
                None
            }
        }
    }

    /// Required boolean (`true/false/1/0/yes/no/y/n/on/off`).
    pub fn var_bool(&mut self, name: &str) -> bool {
        match read(name, parse_bool) {
            Ok(Some(v)) => v,
            Ok(None) => {
                self.errors.push(missing(name));
                false
            }
            Err(e) => {
                self.errors.push(e);
                false
            }
        }
    }

    /// Boolean with a default used only when unset.
    pub fn var_bool_or(&mut self, name: &str, default: bool) -> bool {
        match read(name, parse_bool) {
            Ok(Some(v)) => v,
            Ok(None) => default,
            Err(e) => {
                self.errors.push(e);
                default
            }
        }
    }

    /// Finishes loading. Returns `Err` listing every problem found, or `Ok(())` if no errors occurred.
    #[must_use = "configuration errors are recorded in the returned Result and must be handled"]
    pub fn finish(self) -> Result<(), Errors> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(Errors(self.errors))
        }
    }
}

// ---------------------------------------------------------------------------
// tests (each test uses its own unique variable names; env is process-global)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// `set_var` is `unsafe` in edition 2024. Every test uses its own unique
    /// variable names, so nothing here races with another test.
    fn set(key: &str, value: &str) {
        // SAFETY: unique names per test; no other code reads these variables
        // through libc concurrently.
        unsafe { env::set_var(key, value) }
    }

    #[test]
    fn parses_defaults_and_optional() {
        set("ENVTYPED_T1_PORT", " 8081 ");
        assert_eq!(var::<u16>("ENVTYPED_T1_PORT").unwrap(), 8081);
        assert_eq!(var_or("ENVTYPED_T1_MISSING", 8080u16).unwrap(), 8080);
        assert_eq!(var_opt::<u16>("ENVTYPED_T1_MISSING").unwrap(), None);
    }

    #[test]
    fn missing_required_is_error() {
        let e = var::<u16>("ENVTYPED_T2_MISSING").unwrap_err();
        assert_eq!(
            e,
            EnvError::Missing {
                name: "ENVTYPED_T2_MISSING".into()
            }
        );
    }

    #[test]
    fn invalid_value_is_not_swallowed_by_default() {
        set("ENVTYPED_T3_BAD", "abc");
        let e = var_or("ENVTYPED_T3_BAD", 1u16).unwrap_err();
        assert!(matches!(e, EnvError::Invalid { .. }));
        assert_eq!(e.name(), "ENVTYPED_T3_BAD");
        assert!(e.to_string().contains("`abc`"));
    }

    #[test]
    fn bool_forms() {
        for (raw, want) in [("1", true), ("YES", true), ("on", true), ("0", false), ("No", false)] {
            set("ENVTYPED_T4_BOOL", raw);
            assert_eq!(var_bool("ENVTYPED_T4_BOOL").unwrap(), want, "{raw}");
        }
        set("ENVTYPED_T4_BOOL", "maybe");
        assert!(var_bool("ENVTYPED_T4_BOOL").is_err());
        assert!(var_bool_or("ENVTYPED_T4_UNSET", true).unwrap());
    }

    #[test]
    fn loader_collects_all_errors() {
        set("ENVTYPED_T5_PORT", "abc");
        set("ENVTYPED_T5_DEBUG", "maybe");

        let mut cfg = Loader::new();
        let _port: u16 = cfg.var_or("ENVTYPED_T5_PORT", 8080);
        let _debug = cfg.var_bool_or("ENVTYPED_T5_DEBUG", false);
        let _db: String = cfg.var("ENVTYPED_T5_DB");
        let _ok: u16 = cfg.var_or("ENVTYPED_T5_FINE", 1);

        let errs = cfg.finish().unwrap_err();
        assert_eq!(errs.len(), 3);
        let text = errs.to_string();
        assert!(text.starts_with("3 configuration errors:"));
        assert!(text.contains("ENVTYPED_T5_DB"));
    }

    #[test]
    fn loader_ok_when_everything_valid() {
        set("ENVTYPED_T6_PORT", "9000");
        let mut cfg = Loader::new();
        let port: u16 = cfg.var("ENVTYPED_T6_PORT");
        assert_eq!(port, 9000);
        assert!(cfg.finish().is_ok());
    }

    #[test]
    fn test_env_error_name() {
        let err1 = EnvError::Missing {
            name: "FOO".into(),
        };
        assert_eq!(err1.name(), "FOO");

        let err2 = EnvError::NotUnicode {
            name: "BAR".into(),
        };
        assert_eq!(err2.name(), "BAR");

        let err3 = EnvError::Invalid {
            name: "BAZ".into(),
            value: "xyz".into(),
            reason: "bad".into(),
        };
        assert_eq!(err3.name(), "BAZ");
    }

    #[test]
    fn test_errors_display_single() {
        let mut loader = Loader::new();
        let _ = loader.var::<u32>("ENVTYPED_TEST_MISSING_SINGLE_VAR");
        let err = loader.finish().unwrap_err();
        assert!(err.to_string().starts_with("1 configuration error:"));
        assert_eq!(err.len(), 1);
        assert!(!err.is_empty());
    }
}
