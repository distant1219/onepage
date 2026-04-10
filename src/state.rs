use crate::bing_wallpaper::BingWallpaperClient;
use crate::config::Config;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub bing_client: Arc<BingWallpaperClient>,
    pub tera: Arc<tera::Tera>,
}

impl AppState {
    pub fn new(config: Config, tera: tera::Tera) -> anyhow::Result<Self> {
        let bing_client = BingWallpaperClient::new(
            config.bing.market.clone(),
            config.bing.refresh_interval,
        );

        Ok(Self {
            config: Arc::new(config),
            bing_client: Arc::new(bing_client),
            tera: Arc::new(tera),
        })
    }
}
