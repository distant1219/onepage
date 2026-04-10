use crate::config::{Category, Config, Link};
use std::io::{self, Write};

#[derive(Debug, Clone)]
pub enum CliError {
    ConfigError(String),
    IoError(String),
    ValidationError(String),
    NotFound(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::ConfigError(msg) => write!(f, "配置错误: {}", msg),
            CliError::IoError(msg) => write!(f, "IO 错误: {}", msg),
            CliError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            CliError::NotFound(msg) => write!(f, "未找到: {}", msg),
        }
    }
}

impl std::error::Error for CliError {}

pub struct CliHandler {
    config: Config,
    config_path: String,
}

impl CliHandler {
    pub fn new() -> Result<Self, CliError> {
        let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "onepage.toml".to_string());
        let config = Config::load().map_err(|e| CliError::ConfigError(e.to_string()))?;
        Ok(Self { config, config_path })
    }

    pub fn add_link(
        &mut self,
        name: &str,
        url: &str,
        icon: Option<&str>,
        category: Option<&str>,
    ) -> Result<(), CliError> {
        // 验证 URL
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(CliError::ValidationError(
                "URL 必须以 http:// 或 https:// 开头".to_string(),
            ));
        }

        let icon = icon.unwrap_or("🔗").to_string();
        let category_name = category.unwrap_or("未分类").to_string();

        let new_link = Link {
            name: name.to_string(),
            url: url.to_string(),
            icon,
        };

        // 查找或创建分类
        let category = self
            .config
            .categories
            .iter_mut()
            .find(|c| c.name == category_name);

        if let Some(cat) = category {
            // 检查是否已存在同名链接
            if cat.links.iter().any(|l| l.name == name) {
                return Err(CliError::ValidationError(format!(
                    "分类 '{}' 中已存在名为 '{}' 的链接",
                    category_name, name
                )));
            }
            cat.links.push(new_link);
            println!("✓ 链接 '{}' 已添加到分类 '{}'", name, category_name);
        } else {
            // 创建新分类
            let new_category = Category {
                name: category_name.clone(),
                icon: "📁".to_string(),
                links: vec![new_link],
            };
            self.config.categories.push(new_category);
            println!("✓ 创建新分类 '{}' 并添加链接 '{}'", category_name, name);
        }

        self.save_config()?;
        Ok(())
    }

    pub fn remove_link(&mut self, name: &str, category: Option<&str>) -> Result<(), CliError> {
        let mut found = false;

        if let Some(cat_name) = category {
            // 在指定分类中删除
            if let Some(cat) = self.config.categories.iter_mut().find(|c| c.name == cat_name) {
                let initial_len = cat.links.len();
                cat.links.retain(|l| l.name != name);
                if cat.links.len() < initial_len {
                    found = true;
                    println!("✓ 已从分类 '{}' 中删除链接 '{}'", cat_name, name);
                    // 如果分类为空，询问是否删除
                    if cat.links.is_empty() {
                        print!("分类 '{}' 已为空，是否删除? [y/N] ", cat_name);
                        io::stdout().flush().unwrap();
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();
                        if input.trim().to_lowercase() == "y" {
                            self.config.categories.retain(|c| c.name != cat_name);
                            println!("✓ 已删除空分类 '{}'", cat_name);
                        }
                    }
                }
            }
        } else {
            // 在所有分类中搜索并删除
            for cat in &mut self.config.categories {
                let initial_len = cat.links.len();
                cat.links.retain(|l| l.name != name);
                if cat.links.len() < initial_len {
                    found = true;
                    println!("✓ 已从分类 '{}' 中删除链接 '{}'", cat.name, name);
                }
            }
        }

        if !found {
            return Err(CliError::NotFound(format!("链接 '{}'", name)));
        }

        self.save_config()?;
        Ok(())
    }

    pub fn list_links(&self, category: Option<&str>) -> Result<(), CliError> {
        if self.config.categories.is_empty() {
            println!("暂无链接分类");
            return Ok(());
        }

        println!("\n{}", "━".repeat(60));
        println!("{:<20} {:<30} {}", "分类", "链接名称", "URL");
        println!("{}", "━".repeat(60));

        for cat in &self.config.categories {
            if let Some(filter) = category {
                if cat.name != filter {
                    continue;
                }
            }

            if !cat.links.is_empty() {
                for (i, link) in cat.links.iter().enumerate() {
                    if i == 0 {
                        println!("{:<20} {} {}",
                            format!("{} {}", cat.icon, cat.name),
                            link.name,
                            truncate_url(&link.url, 30)
                        );
                    } else {
                        println!("{:<20} {} {}",
                            "",
                            link.name,
                            truncate_url(&link.url, 30)
                        );
                    }
                }
            }
        }

        println!("{}", "━".repeat(60));

        // 统计信息
        let total_links: usize = self.config.categories.iter().map(|c| c.links.len()).sum();
        let total_categories = self.config.categories.len();
        println!("\n总计: {} 个分类, {} 个链接", total_categories, total_links);

        Ok(())
    }

    pub fn list_categories(&self) -> Result<(), CliError> {
        if self.config.categories.is_empty() {
            println!("暂无分类");
            return Ok(());
        }

        println!("\n{}", "━".repeat(40));
        println!("{:<5} {:<20} {}", "图标", "分类名称", "链接数");
        println!("{}", "━".repeat(40));

        for cat in &self.config.categories {
            println!("{:<5} {:<20} {}",
                cat.icon,
                cat.name,
                cat.links.len()
            );
        }

        println!("{}", "━".repeat(40));
        Ok(())
    }

    fn save_config(&self) -> Result<(), CliError> {
        let toml_str = toml::to_string_pretty(&self.config)
            .map_err(|e| CliError::ConfigError(e.to_string()))?;

        std::fs::write(&self.config_path, toml_str)
            .map_err(|e| CliError::IoError(e.to_string()))?;

        Ok(())
    }
}

fn truncate_url(url: &str, max_len: usize) -> String {
    if url.len() <= max_len {
        url.to_string()
    } else {
        format!("{}...", &url[..max_len-3])
    }
}
