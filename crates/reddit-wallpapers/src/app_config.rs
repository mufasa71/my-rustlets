use anyhow::anyhow;
use config::Config;
use secrecy::SecretString;

static APP_USER_AGENT: &str = concat!(
    "linux:",
    env!("CARGO_PKG_NAME"),
    ":",
    env!("CARGO_PKG_VERSION"),
    " (by /u/insider999)"
);

#[derive(serde::Deserialize)]
pub struct AppConfig {
    pub app_id: String,
    pub app_secret: SecretString,
    pub user_agent: String,
}

pub fn get_app_config(
    app_id: Option<String>,
    app_secret: Option<String>,
    user_agent: Option<String>,
) -> Result<AppConfig, anyhow::Error> {
    Config::builder()
        .set_default("user_agent", APP_USER_AGENT)?
        .add_source(config::Environment::with_prefix("REDDIT"))
        .set_override_option("app_id", app_id)?
        .set_override_option("app_secret", app_secret)?
        .set_override_option("user_agent", user_agent)?
        .build()
        .map_err(|e| anyhow!("Failed to load environment variables: {:?}", e))?
        .try_deserialize::<AppConfig>()
        .map_err(|e| anyhow!("Failed to deserialize config: {:?}", e))
}
