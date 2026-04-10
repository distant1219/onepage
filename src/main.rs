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
use std::net::SocketAddr;
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

    // Initialize Tera templates
    let tera = match tera::Tera::new("templates/**/*") {
        Ok(t) => {
            tracing::info!("Templates loaded successfully");
            t
        }
        Err(e) => {
            tracing::error!("Template parsing error: {}", e);
            return Err(e.into());
        }
    };

    // Create application state
    let app_state = Arc::new(AppState::new(config.clone(), tera)?);

    // Build router
    let app = Router::new()
        .route("/", get(handlers::index_handler))
        .route("/api/wallpaper", get(handlers::wallpaper_api_handler))
        .route("/health", get(handlers::health_handler))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
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
