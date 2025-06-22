use thiserror::Error;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Wallpaper {
    pub id: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct TopDataItem {
    pub data: Wallpaper,
}

#[derive(Deserialize, Debug)]
pub struct TopData {
    pub children: Vec<TopDataItem>,
}

#[derive(Deserialize, Debug)]
pub struct RedditData {
    pub data: TopData,
}

#[derive(Error, Debug)]
pub enum RedditWallpaperError {
    #[error("Download error occuried: {0}")]
    ImageOpenError(#[from] image::ImageError),
}
