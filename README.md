# envtyped

Tiny, zero-dependency typed access to environment variables in Rust.

## Features

- **Typed access**: Parse into any type `T: FromStr`.
- **Strict defaults**: An environment variable that is set but invalid returns an error—it is never silently replaced by a default.
- **Clear errors**: Tells you which variable failed, what the raw value was, and why parsing failed.
- **Batch validation**: Collect all configuration errors in one go with `Loader`, instead of failing on the first one.
- **Zero dependencies**: Pure standard library.
- **Safe**: Built with `#![forbid(unsafe_code)]`.

## Usage

Add `envtyped` to your `Cargo.toml`:

```toml
[dependencies]
envtyped = "0.1"
```

### Basic Usage

```rust
use envtyped::{var, var_or, var_opt, var_bool};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Required variable
    let database_url: String = var("DATABASE_URL")?;

    // Variable with default (used only when unset)
    let port: u16 = var_or("PORT", 8080)?;

    // Optional variable
    let sentry_dsn: Option<String> = var_opt("SENTRY_DSN")?;

    // Boolean (supports true/false/1/0/yes/no/y/n/on/off)
    let debug: bool = var_bool("DEBUG")?;

    Ok(())
}
```

### Reporting All Errors at Once with `Loader`

Instead of stopping on the first missing or invalid environment variable, use `Loader` to collect every error and report them together:

```rust
use envtyped::Loader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut loader = Loader::new();

    let host: String = loader.var_or("HOST", "127.0.0.1".into());
    let port: u16 = loader.var("PORT");
    let pool_size: usize = loader.var_or("POOL_SIZE", 10);
    let secret_key: String = loader.var("SECRET_KEY");
    let metrics: bool = loader.var_bool_or("METRICS_ENABLED", true);

    // Call finish() and check for errors
    loader.finish()?;

    // Values are safe to use here
    println!("Server starting at {host}:{port}");

    Ok(())
}
```

### Real-World Web Application Example (with `.env` support)

In production web applications (e.g. using Axum, Actix Web, or Rocket), configuration is typically structured into a typed `Config` struct. You can pair `envtyped` with `dotenvy` for seamless `.env` file support during local development:

```toml
[dependencies]
envtyped = "0.1"
dotenvy = "0.15" # Optional: for loading .env files in local dev
```

```rust
use envtyped::{Loader, Errors};

pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub max_connections: u32,
    pub debug: bool,
    pub sentry_dsn: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Errors> {
        let mut loader = Loader::new();

        let config = Self {
            host: loader.var_or("HOST", "127.0.0.1".into()),
            port: loader.var_or("PORT", 8080),
            database_url: loader.var("DATABASE_URL"),
            jwt_secret: loader.var("JWT_SECRET"),
            max_connections: loader.var_or("MAX_CONNECTIONS", 10),
            debug: loader.var_bool_or("DEBUG", false),
            sentry_dsn: loader.var_opt("SENTRY_DSN"),
        };

        // Validate all variables at once
        loader.finish()?;

        Ok(config)
    }
}

fn main() {
    // 1. Load `.env` file if present (ignored in production or if missing)
    dotenvy::dotenv().ok();

    // 2. Parse and validate all config at startup
    let config = match AppConfig::from_env() {
        Ok(cfg) => cfg,
        Err(errs) => {
            eprintln!("Configuration error:\n{errs}");
            std::process::exit(1);
        }
    };

    println!("Starting server on {}:{}", config.host, config.port);
}
```

If multiple variables are missing or malformed, the application halts immediately at startup with an exhaustive report instead of failing one-by-one:

```text
Configuration error:
3 configuration errors:
 - environment variable `DATABASE_URL` is not set
 - environment variable `JWT_SECRET` is not set
 - environment variable `PORT` has invalid value `eight-thousand`: invalid digit found in string
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
