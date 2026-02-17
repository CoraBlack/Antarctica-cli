# Antarctica CLI

Antarctica CLI - 一个基于终端的博客管理系统客户端，提供现代化的TUI界面。

## 功能特点

- **用户认证**: 登录、注册、退出登录
- **博客浏览**: 查看最新公开文章，支持源码/渲染模式切换
- **博客编辑**: Vim-like编辑器，支持Markdown编辑
- **个人中心**: 管理个人博客文章
- **现代化UI**: 基于ratatui的终端用户界面

## 界面设计

- 三格高标题栏显示当前界面名称
- 中间功能区域
- 三格高辅助栏（左：状态信息，右：操作提示）
- 按 `?` 键显示帮助面板

## 项目结构

```
antarctica-cli/
├── src/
│   ├── main.rs              # 程序入口
│   ├── lib.rs               # 库入口，模块导出
│   ├── app/
│   │   └── mod.rs           # 应用程序主逻辑和页面路由
│   ├── api/
│   │   └── mod.rs           # API客户端，请求处理
│   ├── config/
│   │   └── mod.rs           # 配置管理（服务器地址、用户信息等）
│   ├── pages/
│   │   ├── mod.rs           # 页面定义
│   │   ├── home.rs          # 主页（最新文章列表）
│   │   ├── login.rs         # 登录页面
│   │   ├── register.rs      # 注册页面
│   │   ├── profile.rs       # 个人资料页面
│   │   ├── blog_view.rs     # 博客预览页面
│   │   └── blog_edit.rs     # 博客编辑页面
│   ├── components/
│   │   ├── mod.rs           # 组件导出
│   │   └── dialog.rs        # 对话框组件（确认、成功、错误等）
│   ├── ui/
│   │   └── mod.rs           # UI组件（布局、标题栏、辅助栏、输入框等）
│   ├── events/
│   │   └── mod.rs           # 事件处理
│   └── utils/
│       └── mod.rs           # 工具函数（错误定义、错误码等）
├── Cargo.toml               # 项目配置和依赖
└── README.md               # 项目说明
```

## 依赖

- **ratatui**: TUI框架
- **crossterm**: 跨平台终端处理
- **reqwest**: HTTP客户端
- **tokio**: 异步运行时
- **pulldown-cmark**: Markdown解析

## 快速开始

### 编译运行

```bash
# 编译项目
cargo build --release

# 运行程序
cargo run
```

### 首次运行

首次运行时会提示配置服务器地址：
- 默认服务器地址: `http://localhost:8080`
- 配置文件位置: `~/.config/antarctica-cli/config.json`

### 界面操作

| 按键 | 功能 |
|------|------|
| `?` | 显示/隐藏帮助面板 |
| `q` / `Esc` | 返回上一页 |
| `Enter` | 确认/选择 |
| `Tab` | 切换输入框 |
| `↑/k` | 上移/上滚 |
| `↓/j` | 下移/下滚 |

### 主要页面

1. **主页**: 显示最新公开博客列表
   - `l` - 登录
   - `o` - 个人中心（需登录）
   - `Enter` - 查看博客详情

2. **登录页**: 用户登录
   - `Tab` - 切换输入框
   - `Enter` - 登录
   - `r` - 跳转注册页

3. **注册页**: 新用户注册
   - `Tab` - 切换输入框
   - `Enter` - 注册

4. **个人中心**: 管理我的博客
   - `n` - 新建博客
   - `Enter` - 查看/编辑博客

5. **博客预览**: 查看博客内容
   - `t` - 切换源码/渲染视图
   - `↑/k` `↓/j` - 滚动
   - `e` - 编辑（仅自己的博客）
   - `F10` - 上传发布（仅自己的博客）

6. **博客编辑**: 编辑博客
   - `i` - 进入/退出编辑模式
   - `t` - 预览模式
   - `Tab` - 切换标题/内容焦点
   - `p` - 切换公开/私有
   - `Ctrl+S` / `F10` - 保存
   - `Esc` - 返回（未保存会提示确认）

## 配置文件

- **位置**: `~/.config/antarctica-cli/config.json`
- **内容**:
```json
{
    "server_url": "http://localhost:8080",
    "auth_token": null,
    "current_user": null
}
```

## 开发指南

### 添加新页面

1. 在 `src/pages/` 创建新页面文件
2. 实现页面渲染和事件处理
3. 在 `app/mod.rs` 中添加页面路由

### 错误处理

项目使用统一的错误处理系统：
- `ErrorCode` 定义错误码
- `AppError` 封装用户友好的错误信息
- UI层通过对话框组件展示错误

## 许可证

MIT License
