
//! # envtyped
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
//! # std::env::set_var("ENVTYPED_DOC_PORT", "9000");
//! let port: u16 = envtyped::var_or("ENVTYPED_DOC_PORT", 8080)?;
//! assert_eq!(port, 9000);
//! # Ok(())
//! # }
//! ```
//!
//! Collecting every error in one go:
//!
//! ```
//! let mut cfg = envtyped::Loader::new();
//! let _port: u16 = cfg.var_or("ENVTYPED_DOC_NOPE_PORT", 8080);
//! let _db: String = cfg.var("ENVTYPED_DOC_NOPE_DB"); // missing -> recorded
//! assert!(cfg.finish().is_err());
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::{env, error::Error, fmt, str:FromStr};

pub enum EnvError {
    // for handling if the variable is not set (missing)
    Missing {
        name: String, // the missing variable name
    },

    // for handing if the variable's value is not a valid unicode
    NotUnicode {
        name: String // the variable name
    }

    // The variable value exists but couldn't be processed or parsed
    Invalid{
        // the variable name
        name: String,
        // the raw value of the variable
        value: String,
        // the reason why the processing or the parsing failed
        reason: String,
    }
    
}


