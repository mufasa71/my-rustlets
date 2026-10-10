mod access_token;
mod app_config;
mod cli;
mod data;

use anyhow::{Result, anyhow};
use clap::Parser;
use cli::Cli;
use data::{AccessToken, RedditData, Wallpaper};
use image::imageops::FilterType;
use log::{Level, error, info, warn};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tokio::task::JoinSet;

use crate::access_token::get_access_token;
use crate::app_config::get_app_config;
use crate::data::RedditWallpaperError;

struct ImageProcessor {
    url: String,
    file_name: String,
    output_path: PathBuf,
}

impl ImageProcessor {
    fn new(url: String, id: String, output_path: PathBuf) -> Result<ImageProcessor> {
        if let Ok(ext) = image::ImageFormat::from_path(&url) {
            let file_name = format!(
                "wallpapers-{}.{}",
                id,
                ext.extensions_str().first().expect("Format expected")
            );

            return Ok(ImageProcessor {
                url,
                file_name,
                output_path,
            });
        }

        Err(anyhow!("File extension not determined for url: {}", url))
    }

    fn get_file_path(&self) -> PathBuf {
        self.output_path.join(&self.file_name)
    }

    fn is_file_exists(&self) -> bool {
        let file_path = self.get_file_path();
        Path::is_file(file_path.as_path())
    }

    async fn classify_brigthness(&self) -> Result<(), anyhow::Error> {
        let file_path = self.get_file_path();

        let image = image::open(&file_path)?;
        let gray_img = image.grayscale();
        let final_img = gray_img.resize(100, 100, FilterType::Triangle);
        let luma_img = final_img.to_luma8();
        let total_intensity: u64 = luma_img.pixels().map(|p| p.0[0] as u64).sum();
        let pixel_count = luma_img.pixels().len() as u64;

        let _: () = if pixel_count > 0 {
            // Calculate the mean normalized to 0.0 - 1.0, then multiplied by 100
            let mean_percentage = (total_intensity as f64 / pixel_count as f64) / 255.0 * 100.0;

            let dark_or_light = if mean_percentage < 50.0 {
                "dark"
            } else {
                "light"
            };
            let sorted_output_path = self
                .output_path
                .join("sorted")
                .join(dark_or_light)
                .join(&self.file_name);

            std::fs::copy(file_path, &sorted_output_path)?;

            info!(
                "Image Mean Intensity: {:.4}%. Moving to {:?}",
                mean_percentage, sorted_output_path
            );
        };

        Ok(())
    }

    async fn download_image(&self, user_agent: &str) -> Result<String> {
        let file_path = self.get_file_path();

        info!("Download from {} to {:?}", self.url, file_path);

        let client = reqwest::Client::builder().user_agent(user_agent).build()?;

        let image = client.get(&self.url).send().await?;

        if image.status() == reqwest::StatusCode::OK {
            let mut file = std::fs::File::create(&file_path)?;
            let content_bytes = image.bytes().await?;
            let mut content = Cursor::new(content_bytes);
            std::io::copy(&mut content, &mut file)?;

            // check if file is image by opening it
            let image = image::open(&file_path);

            match image {
                Ok(_) => {}
                Err(error) => {
                    info!("Image cannot be opened, remove file {:?}", file_path);
                    std::fs::remove_file(file_path)?;
                    return Err(anyhow!(RedditWallpaperError::ImageOpenError(error)));
                }
            }

            info!("Image saved in {:?}", file_path);
            Ok(String::from(&self.url))
        } else {
            Err(anyhow!("Image download error",))
        }
    }
}

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
        let home_path = dirs::home_dir().expect("Home dir not found!");
        let output_path = home_path.join(output);

        let img_processor = ImageProcessor::new(url, id, output_path);

        match img_processor {
            Err(e) => warn!("{:?}", e),
            Ok(img_processor) => {
                if img_processor.is_file_exists() {
                    info!("File {:?} exists - skipping", img_processor.get_file_path());
                } else {
                    let user_agent = app_config.user_agent.clone();
                    set.spawn(async move {
                        let result = img_processor.download_image(user_agent.as_str()).await;

                        if cli.classify {
                            img_processor.classify_brigthness().await?;
                        }

                        result
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
