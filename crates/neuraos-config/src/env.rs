use std::env;

/// Get a required environment variable, panic with a helpful message if missing
pub fn require(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Required environment variable `{}` is not set", key))
}

/// Get an optional environment variable
pub fn optional(key: &str) -> Option<String> {
    env::var(key).ok()
}

/// Get an environment variable with a default value
pub fn with_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Get an environment variable parsed as a specific type
pub fn parse<T: std::str::FromStr>(key: &str, default: T) -> T
where
    T::Err: std::fmt::Display,
{
    match env::var(key) {
        Ok(val) => val.parse().unwrap_or_else(|e| {
            tracing::warn!("Failed to parse env var `{}`: {}, using default", key, e);
            default
        }),
        Err(_) => default,
    }
}

/// Check if we're running in a specific environment
pub fn is_production() -> bool {
    matches!(
        env::var("NEURAOS_ENV").as_deref(),
        Ok("production") | Ok("prod")
    )
}

pub fn is_development() -> bool {
    matches!(
        env::var("NEURAOS_ENV").as_deref(),
        Ok("development") | Ok("dev") | Err(_)
    )
}

pub fn environment() -> String {
    with_default("NEURAOS_ENV", "development")
}
