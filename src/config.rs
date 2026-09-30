use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub argon2_pepper: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenv().ok();

        envy::from_env::<Config>()
    }
}
