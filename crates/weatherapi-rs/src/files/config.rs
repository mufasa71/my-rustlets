use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub api_key: String,
}

pub fn get_config() -> Config {
    let api_key =
        std::env::var("RAPIDAPI_KEY").expect("RAPIDAPI_KEY not found in environment variables");

    Config { api_key }
}
