use crate::{
    api::BlogListItem,
    components::{ErrorDialog, QuitConfirmDialog, WelcomeDialog},
    config::Config,
    events::Event,
    ui::{FooterBar, MainLayout, TitleBar},
    utils::AppError,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// 主页状态
pub struct HomePage {
    /// 博客列表
    blogs: Vec<BlogListItem>,
    /// 当前选中的索引
    selected_index: usize,
    /// 加载状态
    is_loading: bool,
    /// 错误消息（使用新的错误系统）
    error: Option<AppError>,
    /// 显示错误对话框
    show_error: bool,
    /// 显示帮助
    show_help: bool,
    /// 显示退出确认对话框
    show_quit_confirm: bool,
    /// 退出确认对话框
    quit_confirm_dialog: QuitConfirmDialog,
    /// 是否是首次运行
    is_first_run: bool,
    /// 显示欢迎对话框
    show_welcome: bool,
    /// 欢迎对话框
    welcome_dialog: Option<WelcomeDialog>,
    /// 配置文件路径（用于显示）
    config_path: String,
    /// 默认服务器地址
    default_server_url: String,
    /// 是否已登录
    is_authenticated: bool,
}

impl HomePage {
    pub fn new() -> Self {
        Self::new_with_first_run(false)
    }

    pub fn new_with_first_run(is_first_run: bool) -> Self {
        // 获取配置路径和默认服务器地址
        let (config_path, default_server_url) = match Config::config_path() {
            Ok(path) => (
                path.to_string_lossy().to_string(),
                "http://localhost:8080".to_string(),
            ),
            _ => (String::new(), "http://localhost:8080".to_string()),
        };

        // 如果是首次运行，创建欢迎对话框
        let welcome_dialog = if is_first_run {
            Some(WelcomeDialog::new(&default_server_url))
        } else {
            None
        };

        Self {
            blogs: vec![],
            selected_index: 0,
            is_loading: !is_first_run, // 首次运行时不显示加载中
            error: None,
            show_error: false,
            show_help: false,
            show_quit_confirm: false,
            quit_confirm_dialog: QuitConfirmDialog::new(),
            is_first_run,
            show_welcome: is_first_run,
            welcome_dialog,
            config_path,
            default_server_url,
            is_authenticated: false,
        }
    }

    /// 设置博客列表
    pub fn set_blogs(&mut self, blogs: Vec<BlogListItem>) {
        self.blogs = blogs;
        self.is_loading = false;
        self.selected_index = 0;
    }

    /// 设置错误（使用新的AppError）
    pub fn set_error(&mut self, error: AppError) {
        self.error = Some(error);
        self.show_error = true;
        self.is_loading = false;
    }

    pub fn handle_event(&mut self, event: Event) -> HomeAction {
        // 如果显示欢迎对话框，优先处理
        if self.show_welcome {
            return self.handle_welcome_event(event);
        }

        // 如果显示错误对话框，优先处理
        if self.show_error {
            return self.handle_error_event(event);
        }

        // 如果显示退出确认对话框
        if self.show_quit_confirm {
            return self.handle_quit_confirm_event(event);
        }

        // 如果显示帮助面板
        if self.show_help {
            match event {
                Event::Key(key) if key.code == KeyCode::Char('?') || key.code == KeyCode::Esc => {
                    self.show_help = false;
                    return HomeAction::None;
                }
                _ => return HomeAction::None,
            }
        }

        match event {
            Event::Key(key) => self.handle_key(key),
            _ => HomeAction::None,
        }
    }

    fn handle_welcome_event(&mut self, event: Event) -> HomeAction {
        if let Some(ref mut dialog) = self.welcome_dialog {
            match event {
                Event::Key(key) => match key.code {
                    KeyCode::Enter => {
                        // 确认输入的服务器地址
                        let server_url = dialog.server_url().to_string();
                        self.show_welcome = false;
                        self.is_first_run = false;
                        // 清空 welcome_dialog，释放内存
                        self.welcome_dialog = None;
                        return HomeAction::SetServerUrl(server_url);
                    }
                    KeyCode::Esc => {
                        // 使用默认地址
                        let server_url = self.default_server_url.clone();
                        self.show_welcome = false;
                        self.is_first_run = false;
                        self.welcome_dialog = None;
                        return HomeAction::SetServerUrl(server_url);
                    }
                    KeyCode::Backspace => {
                        dialog.handle_backspace();
                        HomeAction::None
                    }
                    KeyCode::Char(c) => {
                        dialog.handle_input(c);
                        HomeAction::None
                    }
                    _ => HomeAction::None,
                },
                _ => HomeAction::None,
            }
        } else {
            HomeAction::None
        }
    }

    fn handle_error_event(&mut self, event: Event) -> HomeAction {
        match event {
            Event::Key(_) => {
                // 按任意键关闭错误对话框
                self.show_error = false;
                self.error = None;
                HomeAction::None
            }
            _ => HomeAction::None,
        }
    }

    fn handle_quit_confirm_event(&mut self, event: Event) -> HomeAction {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Left => {
                    self.quit_confirm_dialog.prev_option();
                    HomeAction::None
                }
                KeyCode::Right => {
                    self.quit_confirm_dialog.next_option();
                    HomeAction::None
                }
                KeyCode::Enter => {
                    if self.quit_confirm_dialog.is_confirmed() {
                        HomeAction::Quit
                    } else {
                        self.show_quit_confirm = false;
                        HomeAction::None
                    }
                }
                KeyCode::Esc => {
                    self.show_quit_confirm = false;
                    HomeAction::None
                }
                _ => HomeAction::None,
            },
            _ => HomeAction::None,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> HomeAction {
        match key.code {
            KeyCode::Char('?') => {
                self.show_help = true;
                HomeAction::None
            }
            KeyCode::Char('q') => {
                // 显示退出确认对话框
                self.show_quit_confirm = true;
                HomeAction::None
            }
            KeyCode::Char('l') => HomeAction::GotoLogin,
            KeyCode::Char('o') => {
                if self.is_authenticated {
                    HomeAction::GotoProfile
                } else {
                    HomeAction::None
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.blogs.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.blogs.len();
                }
                HomeAction::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.blogs.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        self.blogs.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                }
                HomeAction::None
            }
            KeyCode::Enter => {
                if let Some(blog) = self.blogs.get(self.selected_index) {
                    return HomeAction::ViewBlog(blog.id.clone());
                }
                HomeAction::None
            }
            _ => HomeAction::None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, is_authenticated: bool) {
        self.is_authenticated = is_authenticated;
        let layout = MainLayout::new(frame);

        // 渲染标题
        TitleBar::new("Antarctica-Home").render(frame, layout.title_area);

        // 渲染主内容
        self.render_main(frame, layout.main_area);

        // 渲染辅助栏
        let left_info = if is_authenticated {
            vec!["已登录".to_string()]
        } else {
            vec!["未登录".to_string()]
        };

        let hints = if is_authenticated {
            vec![
                ("↑/k".to_string(), "上一条".to_string()),
                ("↓/j".to_string(), "下一条".to_string()),
                ("Enter".to_string(), "查看".to_string()),
                ("o".to_string(), "个人资料".to_string()),
                ("q".to_string(), "退出".to_string()),
                ("?".to_string(), "帮助".to_string()),
            ]
        } else {
            vec![
                ("↑/k".to_string(), "上一条".to_string()),
                ("↓/j".to_string(), "下一条".to_string()),
                ("Enter".to_string(), "查看".to_string()),
                ("l".to_string(), "登录".to_string()),
                ("?".to_string(), "帮助".to_string()),
                ("q".to_string(), "退出".to_string()),
            ]
        };

        let footer = FooterBar::new()
            .with_left_info(left_info)
            .with_right_hints(hints);
        footer.render(frame, layout.footer_left, layout.footer_right);

        // 渲染帮助面板
        if self.show_help {
            self.render_help(frame, area);
        }

        // 渲染退出确认对话框
        if self.show_quit_confirm {
            self.quit_confirm_dialog.render(frame, area);
        }

        // 渲染错误对话框（在欢迎对话框之下）
        if self.show_error && !self.show_welcome {
            if let Some(ref error) = self.error {
                let dialog = ErrorDialog::new(error.code.code(), &error.user_message)
                    .with_detail(error.detail.as_deref().unwrap_or(""));
                dialog.render(frame, area);
            }
        }

        // 渲染欢迎对话框（最后渲染，覆盖其他内容）
        if self.show_welcome {
            if let Some(ref dialog) = self.welcome_dialog {
                dialog.render(frame, area, &self.config_path);
            }
        }
    }

    fn render_main(&self, frame: &mut Frame, area: Rect) {
        if self.is_loading {
            let loading = Paragraph::new("加载中...")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(loading, area);
            return;
        }

        if self.blogs.is_empty() {
            let empty = Paragraph::new("暂无文章")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(empty, area);
            return;
        }

        // 创建博客列表
        let items: Vec<ListItem> = self
            .blogs
            .iter()
            .enumerate()
            .map(|(i, blog)| {
                let style = if i == self.selected_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let content = Text::from(vec![
                    Line::from(vec![Span::styled(
                        &blog.title,
                        style.add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![Span::styled(
                        format!(
                            "  作者: {} | 时间: {}",
                            blog.author.username, blog.created_at
                        ),
                        if i == self.selected_index {
                            style
                        } else {
                            Style::default().fg(Color::Gray)
                        },
                    )]),
                ]);

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("最新文章 ({})", self.blogs.len())),
        );

        frame.render_widget(list, area);
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        use crate::ui::HelpPanel;

        HelpPanel::new("主页帮助")
            .add_item("↑/k", "选择上一篇文章")
            .add_item("↓/j", "选择下一篇文章")
            .add_item("Enter", "查看选中的文章")
            .add_item("o", "跳转到个人信息(已登录)")
            .add_item("l", "跳转到登录页面")
            .add_item("q", "退出应用程序")
            .add_item("?", "显示/隐藏帮助")
            .render(frame, area);
    }
}

#[derive(Debug, Clone)]
pub enum HomeAction {
    None,
    ViewBlog(String),
    GotoLogin,
    GotoProfile,
    Quit,
    SetServerUrl(String),
}
