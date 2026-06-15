mod bing_wallpaper;
mod cli;
mod config;
mod handlers;
mod state;

use crate::state::AppState;
use axum::{
    routing::get,
    Router,
};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "onepage")]
#[command(about = "OnePage - 简洁美观的浏览器首页")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动 Web 服务 (默认)
    Serve,
    /// 链接管理
    #[command(subcommand)]
    Link(LinkCommands),
    /// 分类管理
    #[command(subcommand)]
    Category(CategoryCommands),
}

#[derive(Subcommand)]
enum LinkCommands {
    /// 添加新链接
    Add {
        /// 链接名称
        name: String,
        /// 链接 URL
        url: String,
        /// 图标 (emoji 或字符串，可选)
        #[arg(short, long)]
        icon: Option<String>,
        /// 所属分类
        #[arg(short, long)]
        category: Option<String>,
    },
    /// 删除链接
    Remove {
        /// 链接名称
        name: String,
        /// 指定分类 (可选，不指定则搜索所有分类)
        #[arg(short, long)]
        category: Option<String>,
    },
    /// 列出所有链接
    List {
        /// 按分类筛选
        #[arg(short, long)]
        category: Option<String>,
    },
}

#[derive(Subcommand)]
enum CategoryCommands {
    /// 列出所有分类
    List,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Serve) | None => {
            run_server().await?;
        }
        Some(Commands::Link(cmd)) => {
            handle_link_command(cmd)?;
        }
        Some(Commands::Category(cmd)) => {
            handle_category_command(cmd)?;
        }
    }

    Ok(())
}

async fn run_server() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "onepage=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = config::Config::load()?;
    tracing::info!("Configuration loaded successfully");

    // Resolve asset root so templates/static work regardless of the current
    // working directory (set ONEPAGE_ASSET_DIR for deployments not launched
    // from the project root). Defaults to "." to preserve existing behavior.
    let asset_dir = std::env::var("ONEPAGE_ASSET_DIR").unwrap_or_else(|_| ".".to_string());
    let template_glob = format!("{}/templates/**/*", asset_dir);
    let static_dir = format!("{}/static", asset_dir);

    // Initialize Tera templates
    let mut tera = match tera::Tera::new(&template_glob) {
        Ok(t) => {
            tracing::info!("Templates loaded successfully");
            t
        }
        Err(e) => {
            tracing::error!("Template parsing error: {}", e);
            return Err(e.into());
        }
    };

    // Sanitize URLs rendered into href attributes: only allow http(s), root-
    // relative, or fragment links. Anything else (e.g. `javascript:`) collapses
    // to "#" so config edited outside the CLI can't inject a scheme-based XSS.
    tera.register_filter(
        "safe_url",
        |value: &tera::Value, _: &HashMap<String, tera::Value>| {
            let s = value.as_str().unwrap_or("");
            let safe = s.starts_with("http://")
                || s.starts_with("https://")
                || s.starts_with('/')
                || s.starts_with('#');
            Ok(tera::Value::String(if safe { s.to_string() } else { "#".to_string() }))
        },
    );

    // Create application state
    let app_state = Arc::new(AppState::new(config.clone(), tera)?);

    // Build router
    let app = Router::new()
        .route("/", get(handlers::index_handler))
        .route("/api/wallpaper", get(handlers::wallpaper_api_handler))
        .route("/health", get(handlers::health_handler))
        .nest_service("/static", ServeDir::new(static_dir))
        .with_state(app_state);

    // Start server — honor the configured host instead of hardcoding 0.0.0.0,
    // so `host = "127.0.0.1"` actually binds loopback only.
    let ip: IpAddr = config.server.host.parse().map_err(|e| {
        anyhow::anyhow!("Invalid server host '{}': {}", config.server.host, e)
    })?;
    let addr = SocketAddr::new(ip, config.server.port);
    tracing::info!("Starting server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn handle_link_command(cmd: LinkCommands) -> anyhow::Result<()> {
    let mut handler = cli::CliHandler::new()?;

    match cmd {
        LinkCommands::Add {
            name,
            url,
            icon,
            category,
        } => {
            handler.add_link(&name, &url, icon.as_deref(), category.as_deref())?;
        }
        LinkCommands::Remove { name, category } => {
            handler.remove_link(&name, category.as_deref())?;
        }
        LinkCommands::List { category } => {
            handler.list_links(category.as_deref())?;
        }
    }

    Ok(())
}

fn handle_category_command(cmd: CategoryCommands) -> anyhow::Result<()> {
    let handler = cli::CliHandler::new()?;

    match cmd {
        CategoryCommands::List => {
            handler.list_categories()?;
        }
    }

    Ok(())
}
