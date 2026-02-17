use crate::{
    components::ErrorDialog,
    events::Event,
    ui::{FooterBar, InputField, MainLayout, TitleBar},
    utils::AppError,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, widgets::Clear, Frame};

/// 登录页面状态
pub struct LoginPage {
    /// 用户名输入
    username: String,
    /// 密码输入
    password: String,
    /// 当前聚焦的输入框（0=用户名, 1=密码）
    focus_index: usize,
    /// 错误
    error: Option<AppError>,
    /// 显示错误对话框
    show_error: bool,
    /// 正在加载
    is_loading: bool,
    /// 显示帮助
    show_help: bool,
}

impl LoginPage {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            focus_index: 0,
            error: None,
            show_error: false,
            is_loading: false,
            show_help: false,
        }
    }

    /// 处理输入事件
    pub fn handle_event(&mut self, event: Event) -> LoginAction {
        // 如果显示错误对话框，优先处理
        if self.show_error {
            return self.handle_error_event(event);
        }

        // 如果显示帮助面板
        if self.show_help {
            match event {
                Event::Key(key) if key.code == KeyCode::Char('?') || key.code == KeyCode::Esc => {
                    self.show_help = false;
                    return LoginAction::None;
                }
                _ => return LoginAction::None,
            }
        }

        match event {
            Event::Key(key) => self.handle_key(key),
            _ => LoginAction::None,
        }
    }

    fn handle_error_event(&mut self, event: Event) -> LoginAction {
        match event {
            Event::Key(_) => {
                // 按任意键关闭错误对话框
                self.show_error = false;
                self.error = None;
                LoginAction::None
            }
            _ => LoginAction::None,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> LoginAction {
        match key.code {
            KeyCode::Char('?') => {
                self.show_help = true;
                LoginAction::None
            }
            KeyCode::Tab => {
                self.focus_index = (self.focus_index + 1) % 3; // 0: 用户名, 1: 密码, 2: 确认按钮
                LoginAction::None
            }
            KeyCode::BackTab => {
                self.focus_index = if self.focus_index == 0 {
                    2
                } else {
                    self.focus_index - 1
                };
                LoginAction::None
            }
            KeyCode::Enter => {
                if self.focus_index == 2 || self.focus_index < 2 {
                    // 尝试登录
                    if self.validate() {
                        return LoginAction::Login {
                            username: self.username.clone(),
                            password: self.password.clone(),
                        };
                    }
                }
                LoginAction::None
            }
            KeyCode::Esc => LoginAction::Back,
            KeyCode::Char('r') if self.focus_index >= 2 => LoginAction::GotoRegister,
            KeyCode::Char(c) if self.focus_index == 0 => {
                self.username.push(c);
                self.error = None;
                LoginAction::None
            }
            KeyCode::Char(c) if self.focus_index == 1 => {
                self.password.push(c);
                self.error = None;
                LoginAction::None
            }
            KeyCode::Backspace if self.focus_index == 0 => {
                self.username.pop();
                LoginAction::None
            }
            KeyCode::Backspace if self.focus_index == 1 => {
                self.password.pop();
                LoginAction::None
            }
            _ => LoginAction::None,
        }
    }

    fn validate(&self) -> bool {
        if self.username.is_empty() {
            return false;
        }
        if self.password.is_empty() {
            return false;
        }
        true
    }

    /// 设置错误（使用新的AppError）
    pub fn set_error(&mut self, error: &AppError) {
        self.error = Some(error.clone());
        self.show_error = true;
        self.is_loading = false;
    }

    /// 设置加载状态
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    /// 渲染页面
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let layout = MainLayout::new(frame);

        // 渲染标题
        TitleBar::new("Antarctica-Login").render(frame, layout.title_area);

        // 如果需要显示错误对话框，先清除主内容区和辅助栏，然后只渲染错误对话框
        if self.show_error {
            // 清除主内容区域和辅助栏区域
            frame.render_widget(Clear, layout.main_area);
            frame.render_widget(Clear, layout.footer_area);

            // 在主内容区域渲染错误对话框（使用主内容区的面积）
            if let Some(ref error) = self.error {
                let dialog = ErrorDialog::new(error.code.code(), &error.user_message)
                    .with_detail(error.detail.as_deref().unwrap_or(""));
                dialog.render(frame, layout.main_area);
            }
            return;
        }

        // 渲染主内容区
        self.render_main(frame, layout.main_area);

        // 渲染辅助栏
        let footer = FooterBar::new()
            .with_left_info(vec![
                format!(
                    "用户名: {}",
                    if self.username.is_empty() {
                        "未输入"
                    } else {
                        "已输入"
                    }
                ),
                format!(
                    "密码: {}",
                    if self.password.is_empty() {
                        "未输入"
                    } else {
                        "已输入"
                    }
                ),
            ])
            .with_right_hints(vec![
                ("Tab".to_string(), "切换输入框".to_string()),
                ("Enter".to_string(), "登录".to_string()),
                ("r".to_string(), "注册".to_string()),
                ("Esc".to_string(), "返回".to_string()),
                ("?".to_string(), "帮助".to_string()),
            ]);
        footer.render(frame, layout.footer_left, layout.footer_right);

        // 渲染帮助面板
        if self.show_help {
            self.render_help(frame, area);
        }
    }

    fn render_main(&self, frame: &mut Frame, area: Rect) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            style::{Color, Modifier, Style},
            widgets::{Block, Borders, Paragraph},
        };

        // 创建居中布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 用户名
                Constraint::Length(3), // 密码
                Constraint::Length(3), // 登录按钮
            ])
            .margin((area.height - 9) / 2)
            .split(area);

        // 用户名输入框
        let username_input = InputField::new("用户名")
            .with_value(&self.username)
            .focused(self.focus_index == 0);
        username_input.render(frame, chunks[0]);

        // 密码输入框
        let password_input = InputField::new("密码")
            .with_value(&self.password)
            .password()
            .focused(self.focus_index == 1);
        password_input.render(frame, chunks[1]);

        // 登录按钮
        let button_style = if self.focus_index == 2 {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        };

        let button = Paragraph::new(if self.is_loading {
            "登录中..."
        } else {
            "[ 登录 ]"
        })
        .style(button_style)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(button, chunks[2]);
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        use crate::ui::HelpPanel;

        HelpPanel::new("登录页面帮助")
            .add_item("Tab", "切换到下一个输入框")
            .add_item("Shift+Tab", "切换到上一个输入框")
            .add_item("Enter", "提交登录")
            .add_item("r", "跳转到注册页面")
            .add_item("Esc", "返回上一页")
            .add_item("Backspace", "删除字符")
            .add_item("?", "显示/隐藏帮助")
            .render(frame, area);
    }
}

/// 登录页面动作
#[derive(Debug, Clone)]
pub enum LoginAction {
    None,
    Login { username: String, password: String },
    GotoRegister,
    Back,
}
