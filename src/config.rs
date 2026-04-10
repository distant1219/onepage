use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_server")]
    pub server: ServerConfig,
    #[serde(default = "default_bing")]
    pub bing: BingConfig,
    #[serde(default)]
    pub search_engines: Vec<SearchEngine>,
    #[serde(default)]
    pub categories: Vec<Category>,
    #[serde(default)]
    pub timezones: Vec<TimezoneConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BingConfig {
    #[serde(default = "default_market")]
    pub market: String,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchEngine {
    pub name: String,
    pub url: String,
    pub icon: String,
    #[serde(default)]
    pub default: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Category {
    pub name: String,
    pub icon: String,
    #[serde(default)]
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Link {
    pub name: String,
    pub url: String,
    pub icon: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimezoneConfig {
    pub name: String,
    pub tz: String,
    pub icon: String,
}

fn default_server() -> ServerConfig {
    ServerConfig {
        host: default_host(),
        port: default_port(),
    }
}

fn default_bing() -> BingConfig {
    BingConfig {
        market: default_market(),
        refresh_interval: default_refresh_interval(),
    }
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_market() -> String {
    "zh-CN".to_string()
}

fn default_refresh_interval() -> u64 {
    60
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "onepage.toml".to_string());

        if !Path::new(&config_path).exists() {
            tracing::warn!("Config file not found at {}, using defaults", config_path);
            return Ok(Config::default());
        }

        let builder = config::Config::builder()
            .add_source(config::File::with_name(&config_path))
            .add_source(config::Environment::with_prefix("ONEPAGE").separator("__"));

        let config = builder.build()?;
        Ok(config.try_deserialize()?)
    }

    pub fn get_default_search_engine(&self) -> Option<&SearchEngine> {
        self.search_engines.iter().find(|e| e.default)
            .or_else(|| self.search_engines.first())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: default_server(),
            bing: default_bing(),
            search_engines: vec![
                SearchEngine {
                    name: "Google".to_string(),
                    url: "https://www.google.com/search?q=".to_string(),
                    icon: "🔍".to_string(),
                    default: true,
                },
            ],
            categories: vec![],
            timezones: vec![
                TimezoneConfig {
                    name: "本地".to_string(),
                    tz: "Asia/Shanghai".to_string(),
                    icon: "🏠".to_string(),
                },
            ],
        }
    }
}
