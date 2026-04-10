use crate::config::TimezoneConfig;
use crate::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use chrono::{Local, TimeZone, Utc};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct TimezoneInfo {
    pub name: String,
    pub icon: String,
    pub time: String,
    pub date: String,
    pub tz: String,  // 时区标识，用于前端实时计算
    pub is_primary: bool,
}

impl TimezoneInfo {
    pub fn from_config(config: &TimezoneConfig, is_primary: bool) -> Self {
        let naive_now = if config.tz == "local" || config.tz == "Local" {
            Local::now().naive_local()
        } else {
            match config.tz.parse::<chrono_tz::Tz>() {
                Ok(tz) => {
                    let utc = Utc::now();
                    tz.from_utc_datetime(&utc.naive_utc()).naive_local()
                }
                Err(_) => Local::now().naive_local(),
            }
        };

        TimezoneInfo {
            name: config.name.clone(),
            icon: config.icon.clone(),
            time: naive_now.format("%H:%M:%S").to_string(),  // 显示到秒
            date: naive_now.format("%m月%d日").to_string(),
            tz: config.tz.clone(),  // 传递时区信息
            is_primary,
        }
    }
}

pub async fn index_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let wallpaper = match state.bing_client.get_wallpaper().await {
        Ok(w) => w,
        Err(e) => {
            tracing::error!("Failed to get wallpaper: {}", e);
            // Return default wallpaper on error
            crate::bing_wallpaper::WallpaperInfo {
                url: "https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=1920".to_string(),
                copyright: "Default wallpaper".to_string(),
                copyright_link: "#".to_string(),
                title: "Default".to_string(),
            }
        }
    };

    // Generate timezone info
    let timezones: Vec<TimezoneInfo> = state
        .config
        .timezones
        .iter()
        .enumerate()
        .map(|(idx, tz)| TimezoneInfo::from_config(tz, idx == 0))
        .collect();

    let now = Local::now();
    let date_str = now.format("%Y年%m月%d日 %A").to_string();
    let time_str = now.format("%H:%M").to_string();

    let mut context = tera::Context::new();
    context.insert("wallpaper_url", &wallpaper.url);
    context.insert("wallpaper_copyright", &wallpaper.copyright);
    context.insert("wallpaper_copyright_link", &wallpaper.copyright_link);
    context.insert("current_date", &date_str);
    context.insert("current_time", &time_str);
    context.insert("search_engines", &state.config.search_engines);
    context.insert("categories", &state.config.categories);
    context.insert("timezones", &timezones);

    // Get default search engine
    let default_engine = state.config.get_default_search_engine();
    context.insert("default_engine", &default_engine);

    match state.tera.render("index.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Template render error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Template error").into_response()
        }
    }
}

pub async fn wallpaper_api_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.bing_client.get_wallpaper().await {
        Ok(wallpaper) => {
            let response = json!({
                "url": wallpaper.url,
                "copyright": wallpaper.copyright,
                "copyright_link": wallpaper.copyright_link,
                "title": wallpaper.title,
            });
            Json(response).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get wallpaper: {}", e);
            let error_response = json!({
                "error": "Failed to fetch wallpaper",
                "message": e.to_string(),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

pub async fn health_handler() -> impl IntoResponse {
    let response = json!({
        "status": "healthy",
        "timestamp": Local::now().to_rfc3339(),
    });
    Json(response)
}
