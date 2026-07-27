use crate::{AccessToken, app_config::AppConfig};
use anyhow::Result;
use secrecy::ExposeSecret;

pub async fn get_access_token(client: &reqwest::Client, config: &AppConfig) -> Result<String> {
    let token = client
        .post("https://www.reddit.com/api/v1/access_token")
        .basic_auth(
            config.app_id.clone(),
            Some(config.app_secret.expose_secret()),
        )
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await?
        .error_for_status()?
        .json::<AccessToken>()
        .await?;

    Ok(token.access_token)
}
