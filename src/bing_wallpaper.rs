use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Deserialize)]
struct BingImageResponse {
    images: Vec<BingImage>,
}

#[derive(Debug, Clone, Deserialize)]
struct BingImage {
    url: String,
    urlbase: String,
    copyright: String,
    copyrightlink: String,
    title: String,
}

#[derive(Debug, Clone)]
pub struct WallpaperInfo {
    pub url: String,
    pub copyright: String,
    pub copyright_link: String,
    pub title: String,
}

pub struct BingWallpaperClient {
    market: String,
    client: reqwest::Client,
    cached_wallpaper: Arc<RwLock<Option<(WallpaperInfo, DateTime<Utc>)>>>,
    refresh_interval_minutes: i64,
}

impl BingWallpaperClient {
    pub fn new(market: String, refresh_interval_minutes: u64) -> Self {
        Self {
            market,
            client: reqwest::Client::new(),
            cached_wallpaper: Arc::new(RwLock::new(None)),
            refresh_interval_minutes: refresh_interval_minutes as i64,
        }
    }

    pub async fn get_wallpaper(&self) -> anyhow::Result<WallpaperInfo> {
        // Check cache first
        {
            let cache = self.cached_wallpaper.read().await;
            if let Some((wallpaper, cached_at)) = cache.as_ref() {
                let elapsed = Utc::now().signed_duration_since(*cached_at);
                if elapsed.num_minutes() < self.refresh_interval_minutes {
                    tracing::debug!("Returning cached wallpaper");
                    return Ok(wallpaper.clone());
                }
            }
        }

        // Fetch new wallpaper
        let wallpaper = self.fetch_wallpaper().await?;

        // Update cache
        {
            let mut cache = self.cached_wallpaper.write().await;
            *cache = Some((wallpaper.clone(), Utc::now()));
        }

        Ok(wallpaper)
    }

    async fn fetch_wallpaper(&self) -> anyhow::Result<WallpaperInfo> {
        let url = format!(
            "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt={}",
            self.market
        );

        tracing::info!("Fetching Bing wallpaper from: {}", url);

        let response = self.client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.0")
            .send()
            .await?;

        let bing_response: BingImageResponse = response.json().await?;

        if let Some(image) = bing_response.images.into_iter().next() {
            let full_url = if image.url.starts_with("http") {
                image.url
            } else {
                format!("https://www.bing.com{}", image.url)
            };

            Ok(WallpaperInfo {
                url: full_url,
                copyright: image.copyright,
                copyright_link: image.copyrightlink,
                title: image.title,
            })
        } else {
            anyhow::bail!("No wallpaper found in Bing response")
        }
    }
}
