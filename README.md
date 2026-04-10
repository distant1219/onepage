# OnePage - 浏览器首页

一个简洁美观的浏览器首页，使用 Rust + Axum 构建，支持必应每日壁纸、多搜索引擎切换和分类快捷链接。

## 功能特性

- **必应每日壁纸** - 自动获取并显示微软必应每日推荐壁纸
- **多搜索引擎** - 支持 Google、Bing、百度、DuckDuckGo 等，可配置默认引擎
- **分类快捷链接** - 按类别组织常用网站，网格化展示
- **双时区显示** - 同时显示北京和美西时间
- **毛玻璃效果** - 现代化的 UI 设计
- **响应式布局** - 适配桌面和移动设备
- **CLI 工具** - 命令行管理快捷链接

## 快速开始

### 本地运行

1. 确保已安装 Rust (1.75+)
2. 克隆项目并进入目录
3. 运行：

```bash
cargo run
```

服务将在 http://localhost:8080 启动

### Docker 部署

#### 使用 Docker Compose（推荐）

```bash
# 构建并启动
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down
```

#### 使用 Docker 命令

```bash
# 构建镜像
docker build -t onepage .

# 运行容器
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/onepage.toml:/app/config/onepage.toml:ro \
  --name onepage \
  onepage
```

## 配置说明

编辑 `onepage.toml` 文件自定义你的首页：

```toml
[server]
host = "0.0.0.0"
port = 8080

[bing]
market = "zh-CN"  # 必应壁纸市场

# 添加搜索引擎
[[search_engines]]
name = "Google"
url = "https://www.google.com/search?q="
icon = "🔍"
default = true

# 添加快捷链接分类
[[categories]]
name = "常用"
icon = "⭐"

[[categories.links]]
name = "GitHub"
url = "https://github.com"
icon = "🐙"
```

## CLI 工具

OnePage 提供命令行工具管理快捷链接，无需手动编辑配置文件。

### 添加链接

```bash
# 基本用法
./onepage link add "链接名称" "https://example.com"

# 指定图标和分类
./onepage link add "Hacker News" "https://news.ycombinator.com" -i "📰" -c "阅读"
```

### 列出链接

```bash
# 列出所有链接
./onepage link list

# 按分类筛选
./onepage link list -c "常用"
```

### 删除链接

```bash
# 删除链接（自动搜索所有分类）
./onepage link remove "链接名称"

# 从指定分类删除
./onepage link remove "链接名称" -c "常用"
```

### 分类管理

```bash
# 列出所有分类
./onepage category list
```

### 启动服务

```bash
# 启动 Web 服务（默认）
./onepage
# 或
./onepage serve
```

## 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `CONFIG_PATH` | 配置文件路径 | `onepage.toml` |
| `RUST_LOG` | 日志级别 | `info` |

## 技术栈

- **后端**: Rust + Axum
- **模板引擎**: Tera
- **前端**: HTML5 + CSS3 + Vanilla JS
- **部署**: Docker

## 许可证

MIT
