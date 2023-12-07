use std::{env, str::FromStr};

pub fn env_parse<T: FromStr>(env: &str, default: T) -> T {
    match env::var(env) {
        Ok(var) => var.parse::<T>().unwrap_or(default),
        Err(_) => default,
    }
}