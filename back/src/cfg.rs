use std::env;

#[derive(Deserialize)]
pub struct Config {
    pub db: Db,
}

#[derive(Deserialize)]
pub struct Db {
    pub host: String,
}

impl Config {
    pub fn from_env() -> Config {
        Config {
            db: Db {
                host: env::var("CORINOR_DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            },
        }
    }
}
