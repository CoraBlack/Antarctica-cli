# Antarctica CLI

一个高度模块化、强可扩展的博客服务器CLI客户端。

## 功能特点

- **高度模块化设计**: 采用模块化架构，各功能模块独立开发，互不影响
- **强可扩展性**: 通过页面系统轻松添加新功能
- **配置文件系统**: 支持多服务器配置，方便切换不同服务器
- **现代化CLI界面**: 使用cliclack提供美观的交互式命令行界面

## 项目结构

```
antarctica-cli/
├── src/
│   ├── main.rs           # 程序入口
│   ├── app.rs            # 应用程序主逻辑
│   ├── error.rs          # 错误处理
│   ├── config/           # 配置管理模块
│   │   ├── mod.rs
│   │   ├── app_config.rs
│   │   └── server_config.rs
│   ├── pages/            # 页面模块
│   │   ├── mod.rs
│   │   ├── page.rs       # 页面基础trait
│   │   ├── home.rs       # 首页
│   │   ├── auth.rs       # 认证页面(登录/注册)
│   │   └── settings.rs   # 设置页面
│   └── api/              # API客户端模块
│       ├── mod.rs
│       ├── client.rs     # HTTP客户端
│       ├── auth.rs       # 认证API
│       └── models.rs     # 数据模型
├── Cargo.toml            # 项目配置和依赖
└── README.md             # 项目说明
```

## 快速开始

### 安装

```bash
# 克隆仓库
git clone https://github.com/yourusername/antarctica-cli.git
cd antarctica-cli

# 编译项目
cargo build --release

# 运行程序
cargo run
```

### 配置服务器

首次运行时，程序会在设置页面引导您配置服务器：

1. 启动程序
2. 选择"设置"菜单
3. 选择"添加服务器"
4. 输入服务器名称和地址
5. 选择是否使用HTTPS
6. 保存配置

### 使用说明

程序提供以下主要功能：

- **首页**: 应用程序主界面，提供导航菜单
- **登录**: 登录到博客服务器
- **注册**: 注册新用户账号
- **设置**: 配置应用和服务器

## 开发指南

### 添加新页面

要添加新页面，需要：

1. 在`src/pages/`目录下创建新页面文件
2. 实现`Page` trait
3. 在`src/pages/mod.rs`中注册新页面
4. 在`app.rs`中注册页面到页面管理器

示例：

```rust
use crate::pages::{Page, PageAction, PageResult};
use crate::config::AppConfig;
use anyhow::Result;

pub struct MyPage {
    name: String,
    description: String,
}

impl MyPage {
    pub fn new() -> Self {
        Self {
            name: "my_page".to_string(),
            description: "我的页面".to_string(),
        }
    }
}

impl Page for MyPage {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn render(&self) -> Result<()> {
        // 渲染页面内容
        Ok(())
    }

    fn handle_input(&mut self, input: &str, config: &mut AppConfig) -> PageResult {
        // 处理用户输入
        Ok(None)
    }
}
```

### 添加新的API端点

要添加新的API端点，需要：

1. 在`src/api/models.rs`中定义数据模型
2. 在相应的API模块（如`auth.rs`）中添加API方法
3. 在`ApiClient`中实现HTTP请求方法

示例：

```rust
// 在models.rs中定义模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyModel {
    pub id: String,
    pub name: String,
}

// 在client.rs中添加API方法
impl ApiClient {
    pub async fn get_my_model(&self, id: &str) -> anyhow::Result<MyModel> {
        self.get(&format!("/api/v1/my-models/{}", id)).await
    }
}
```

## 配置文件

配置文件位置：

- Windows: `%APPDATA%ntarctica-cli\config.toml`
- Linux/Mac: `~/.config/antarctica-cli/config.toml`

配置文件示例：

```toml
version = "0.1.0"

[servers]
active_server = "my_blog"

[[servers.servers]]
name = "my_blog"
url = "blog.example.com"
secure = true
api_version = "v1"

[preferences]
language = "zh-CN"
theme = "default"
auto_save = true
auto_save_interval = 300
```

## 许可证

MIT License
