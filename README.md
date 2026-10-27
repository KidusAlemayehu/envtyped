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

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
