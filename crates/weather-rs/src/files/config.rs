use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub api_key: String,
}

pub fn get_config() -> Config {
    let mut rapid_api_path = dirs::config_dir().expect("Could not find config directory");
    rapid_api_path.push("rapidapi.toml");
    let config_str = std::fs::read_to_string(rapid_api_path).expect("Could not read config file");
    let config: Config = toml::from_str(&config_str).expect("Could not parse config file");
    config
}
