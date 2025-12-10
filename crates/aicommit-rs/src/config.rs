use std::fs::File;
use std::io::Read;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub openai_api_key: String,
    pub openai_api_url: String,
    pub model_name: String,
}

const CONFIG_FILE_NAME: &str = ".aicommit.toml";

pub fn get_config() -> Config {
    let path = dirs::home_dir().expect("Home dir not found");
    let mut config_file = File::open(path.join(CONFIG_FILE_NAME)).expect("Config file not found");
    let mut config_str = String::new();
    config_file
        .read_to_string(&mut config_str)
        .expect("Error reading config file");
    let config: Config = toml::from_str(&config_str).expect("Failed to parse config file");

    config
}
