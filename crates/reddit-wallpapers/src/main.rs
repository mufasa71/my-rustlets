mod access_token;
mod app_config;
mod cli;
mod data;

use anyhow::{Result, anyhow};
use clap::Parser;
use cli::Cli;
use data::{AccessToken, RedditData, Wallpaper};
use log::{Level, error, info, warn};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tokio::task::JoinSet;

use crate::access_token::get_access_token;
use crate::app_config::get_app_config;
use crate::data::RedditWallpaperError;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let output = cli.output.as_deref().unwrap_or("Pictures/Wallpapers");
    if let Some(cli_log_level) = cli.log_level {
        let log_level = match cli_log_level {
            1 => Some(Level::Error),
            2 => Some(Level::Warn),
            3 => Some(Level::Info),
            4 => Some(Level::Debug),
            5 => Some(Level::Trace),
            _ => None,
        };
        if let Some(ll) = log_level {
            simple_logger::init_with_level(ll).expect("Logger init failed");
        }
    }
    let app_id = cli.app_id.clone();
    let app_secret = cli.app_secret.clone();
    let user_agent = cli.user_agent.clone();
    let app_config = get_app_config(app_id, app_secret, user_agent)?;
    let client = reqwest::Client::builder()
        .user_agent(&app_config.user_agent)
        .build()?;
    let mut q: Vec<(&str, String)> = vec![];

    if let Some(limit) = cli.limit {
        q.push(("limit", limit.to_string()));
    }

    if let Some(t) = cli.t {
        q.push(("t", t.to_string()))
    }

    let token = match get_access_token(&client, &app_config).await {
        Ok(token) => token,
        Err(e) => {
            error!("Reddit authentication failed: {:?}", e);
            return Ok(());
        }
    };

    let response = client
        .get("https://oauth.reddit.com/r/wallpaper/top.json")
        .bearer_auth(&token)
        .query(&q)
        .send()
        .await
        .and_then(|r| r.error_for_status());

    let response = match response {
        Ok(response) => response,
        Err(e) => {
            error!("Request to Reddit failed: {:?}", e);
            return Ok(());
        }
    };

    let body = response.json::<RedditData>().await?;

    let mut set = JoinSet::new();
    let children = body.data.children;

    if children.is_empty() {
        info!("No images found");
    }

    for child in children {
        let Wallpaper { url, id } = child.data;
        let file_name = get_file_name(&url, &id);

        match file_name {
            Err(e) => warn!("Cannot get file name: {:?}", e),
            Ok(file_name) => {
                let mut file_path = dirs::home_dir().expect("Home dir not found!");
                file_path = file_path.join(output).join(file_name);

                let is_file_exists = Path::is_file(file_path.as_path());

                if is_file_exists {
                    info!("File {:?} exists - skipping", file_path);
                } else {
                    let user_agent = app_config.user_agent.clone();
                    set.spawn(async move {
                        download_image(&url, &file_path, user_agent.as_str()).await
                    });
                }
            }
        }
    }

    while let Some(result) = set.join_next().await {
        match result {
            Ok(result) => match result {
                Ok(url) => info!("Download finished for {}", url),
                Err(error) => {
                    error!("Error occured: {:?}", error);
                }
            },
            Err(e) => {
                error!("Error spawning process: {:?}", e);
            }
        }
    }

    Ok(())
}

fn get_file_name(url: &str, id: &str) -> Result<String> {
    let ext = image::ImageFormat::from_path(url);
    match ext {
        Ok(f) => match f {
            image::ImageFormat::Png => Ok(format!("wallpapers-{}.png", id)),
            image::ImageFormat::Jpeg => Ok(format!("wallpapers-{}.jpg", id)),
            _ => Err(anyhow!("Not supported file extension: {:?})", f)),
        },
        Err(_) => Err(anyhow!("File extension not determined for url: {}", url)),
    }
}

async fn download_image(url: &str, file_path_buf: &PathBuf, user_agent: &str) -> Result<String> {
    info!("Download from {} to {:?}", url, file_path_buf);

    let client = reqwest::Client::builder().user_agent(user_agent).build()?;

    let image = client.get(url).send().await?;

    if image.status() == reqwest::StatusCode::OK {
        let mut file = std::fs::File::create(file_path_buf)?;
        let content_bytes = image.bytes().await?;
        let mut content = Cursor::new(content_bytes);
        std::io::copy(&mut content, &mut file)?;

        // check if file is image by opening it
        let image = image::open(file_path_buf);

        match image {
            Ok(_) => {}
            Err(error) => {
                info!("Image cannot be opened, remove file {:?}", file_path_buf);
                std::fs::remove_file(file_path_buf)?;
                return Err(anyhow!(RedditWallpaperError::ImageOpenError(error)));
            }
        }

        info!("Image saved in {:?}", file_path_buf);
        Ok(String::from(url))
    } else {
        Err(anyhow!("Image download error",))
    }
}
