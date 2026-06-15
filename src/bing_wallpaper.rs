use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Deserialize)]
struct BingImageResponse {
    images: Vec<BingImage>,
}

#[derive(Debug, Clone, Deserialize)]
struct BingImage {
    url: String,
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

/// Cached wallpaper paired with the time it was fetched.
type WallpaperCache = Arc<RwLock<Option<(WallpaperInfo, DateTime<Utc>)>>>;

pub struct BingWallpaperClient {
    market: String,
    client: reqwest::Client,
    cached_wallpaper: WallpaperCache,
    // Serializes upstream fetches so a cache miss under concurrency triggers a
    // single Bing request (single-flight) instead of one per in-flight request.
    fetch_lock: Arc<Mutex<()>>,
    refresh_interval_minutes: i64,
}

impl BingWallpaperClient {
    pub fn new(market: String, refresh_interval_minutes: u64) -> Self {
        // Bound the upstream call so a slow/hung Bing API can't stall the
        // homepage handler that awaits it on a cache miss.
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            market,
            client,
            cached_wallpaper: Arc::new(RwLock::new(None)),
            fetch_lock: Arc::new(Mutex::new(())),
            refresh_interval_minutes: refresh_interval_minutes as i64,
        }
    }

    fn cache_is_fresh(&self, cached_at: &DateTime<Utc>) -> bool {
        Utc::now().signed_duration_since(*cached_at).num_minutes() < self.refresh_interval_minutes
    }

    pub async fn get_wallpaper(&self) -> anyhow::Result<WallpaperInfo> {
        // Fast path: fresh cache, no lock contention.
        {
            let cache = self.cached_wallpaper.read().await;
            if let Some((wallpaper, cached_at)) = cache.as_ref() {
                if self.cache_is_fresh(cached_at) {
                    tracing::debug!("Returning cached wallpaper");
                    return Ok(wallpaper.clone());
                }
            }
        }

        // Cache miss: single-flight. Only one task fetches; the rest wait here
        // and then see the freshly-populated cache on the re-check below.
        let _guard = self.fetch_lock.lock().await;
        {
            let cache = self.cached_wallpaper.read().await;
            if let Some((wallpaper, cached_at)) = cache.as_ref() {
                if self.cache_is_fresh(cached_at) {
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
