use std::{borrow::Cow, env};

/// Read an environment variable and return it as a clone-on-write pointer,
/// since it can also return a default value if the variable is not found.
pub fn read_env_str<'a>(name: &str, default: &'a str) -> Cow<'a, str> {
    match env::var(name) {
        Ok(value) => Cow::Owned(value),
        Err(_) => {
            println!(
                "Could not find value of {}, using default {}.",
                name, default
            );
            Cow::Borrowed(default)
        }
    }
}

/// Read an environment variable and return it as a u32.
pub fn read_env_u32(name: &str, default: u32) -> u32 {
    match env::var(name) {
        Ok(value) => value.parse().unwrap_or_else(|_| {
            println!(
                "Could not parse value of {}, using default {}.",
                name, default
            );
            default
        }),
        Err(_) => {
            println!(
                "Could not find value of {}, using default {}.",
                name, default
            );
            default
        }
    }
}
