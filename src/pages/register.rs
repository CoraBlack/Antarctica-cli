use crate::{
    components::{ErrorDialog, SuccessDialog},
    events::Event,
    ui::{FooterBar, InputField, MainLayout, TitleBar},
    utils::{AppError, ErrorCode},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

/// 注册页面状态
pub struct RegisterPage {
    /// 用户名
    username: String,
    /// 昵称
    nickname: String,
    /// 邮箱
    email: String,
    /// 密码
    password: String,
    /// 确认密码
    confirm_password: String,
    /// 当前聚焦的输入框
    focus_index: usize,
    /// 错误
    error: Option<AppError>,
    /// 显示错误对话框
    show_error: bool,
    /// 正在加载
    is_loading: bool,
    /// 显示帮助
    show_help: bool,
    /// 成功消息
    success_message: Option<String>,
    /// 显示成功对话框
    show_success: bool,
}

impl RegisterPage {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            nickname: String::new(),
            email: String::new(),
            password: String::new(),
            confirm_password: String::new(),
            focus_index: 0,
            error: None,
            show_error: false,
            is_loading: false,
            show_help: false,
            success_message: None,
            show_success: false,
        }
    }

    pub fn handle_event(&mut self, event: Event) -> RegisterAction {
        // 如果显示成功对话框
        if self.show_success {
            match event {
                Event::Key(_) => {
                    self.show_success = false;
                    self.success_message = None;
                    return RegisterAction::Back;
                }
                _ => return RegisterAction::None,
            }
        }

        // 如果显示错误对话框，优先处理
        if self.show_error {
            return self.handle_error_event(event);
        }

        // 如果显示帮助面板
        if self.show_help {
            match event {
                Event::Key(key) if key.code == KeyCode::Char('?') || key.code == KeyCode::Esc => {
                    self.show_help = false;
                    return RegisterAction::None;
                }
                _ => return RegisterAction::None,
            }
        }

        match event {
            Event::Key(key) => self.handle_key(key),
            _ => RegisterAction::None,
        }
    }

    fn handle_error_event(&mut self, event: Event) -> RegisterAction {
        match event {
            Event::Key(_) => {
                // 按任意键关闭错误对话框
                self.show_error = false;
                self.error = None;
                RegisterAction::None
            }
            _ => RegisterAction::None,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> RegisterAction {
        match key.code {
            KeyCode::Char('?') => {
                self.show_help = true;
                RegisterAction::None
            }
            KeyCode::Tab => {
                self.focus_index = (self.focus_index + 1) % 6;
                RegisterAction::None
            }
            KeyCode::BackTab => {
                self.focus_index = if self.focus_index == 0 {
                    5
                } else {
                    self.focus_index - 1
                };
                RegisterAction::None
            }
            KeyCode::Enter => {
                if let Err(msg) = self.validate() {
                    // 创建验证错误
                    let err = AppError::new(ErrorCode::ValidationError, msg);
                    self.error = Some(err);
                    self.show_error = true;
                } else {
                    return RegisterAction::Register {
                        username: self.username.clone(),
                        nickname: self.nickname.clone(),
                        password: self.password.clone(),
                        email: self.email.clone(),
                    };
                }
                RegisterAction::None
            }
            KeyCode::Esc => RegisterAction::Back,
            KeyCode::Char(c) => {
                match self.focus_index {
                    0 => self.username.push(c),
                    1 => self.nickname.push(c),
                    2 => self.email.push(c),
                    3 => self.password.push(c),
                    4 => self.confirm_password.push(c),
                    _ => {}
                }
                self.error = None;
                RegisterAction::None
            }
            KeyCode::Backspace => {
                match self.focus_index {
                    0 => self.username.pop(),
                    1 => self.nickname.pop(),
                    2 => self.email.pop(),
                    3 => self.password.pop(),
                    4 => self.confirm_password.pop(),
                    _ => None,
                };
                RegisterAction::None
            }
            _ => RegisterAction::None,
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.username.len() < 3 {
            return Err("用户名至少需要3个字符".to_string());
        }
        if self.nickname.is_empty() {
            return Err("昵称不能为空".to_string());
        }
        if !self.email.contains('@') {
            return Err("请输入有效的邮箱地址".to_string());
        }
        if self.password.len() < 8 {
            return Err("密码至少需要8个字符".to_string());
        }
        if self.password != self.confirm_password {
            return Err("两次输入的密码不一致".to_string());
        }
        Ok(())
    }

    /// 设置错误（使用新的AppError）
    pub fn set_error(&mut self, error: &AppError) {
        self.error = Some(error.clone());
        self.show_error = true;
        self.is_loading = false;
    }

    /// 设置成功消息
    pub fn set_success(&mut self, message: String) {
        self.success_message = Some(message);
        self.show_success = true;
        self.is_loading = false;
    }

    /// 设置加载状态
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let layout = MainLayout::new(frame);

        // 渲染标题
        TitleBar::new("Antarctica-Register").render(frame, layout.title_area);

        // 渲染主内容
        self.render_main(frame, layout.main_area);

        // 渲染辅助栏
        let footer = FooterBar::new()
            .with_left_info(vec![
                format!("步骤: {}/5", self.focus_index + 1),
                format!("用户名长度: {}", self.username.len()),
            ])
            .with_right_hints(vec![
                ("Tab".to_string(), "切换".to_string()),
                ("Enter".to_string(), "注册".to_string()),
                ("Esc".to_string(), "返回".to_string()),
                ("?".to_string(), "帮助".to_string()),
            ]);
        footer.render(frame, layout.footer_left, layout.footer_right);

        // 渲染帮助面板
        if self.show_help {
            self.render_help(frame, area);
        }

        // 渲染错误对话框
        if self.show_error {
            if let Some(ref error) = self.error {
                let dialog = ErrorDialog::new(error.code.code(), &error.user_message)
                    .with_detail(error.detail.as_deref().unwrap_or(""));
                dialog.render(frame, area);
            }
        }

        // 渲染成功对话框（最后渲染，覆盖其他内容）
        if self.show_success {
            if let Some(ref message) = self.success_message {
                SuccessDialog::new(message).render(frame, area);
            }
        }
    }

    fn render_main(&self, frame: &mut Frame, area: Rect) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            style::{Color, Modifier, Style},
            widgets::{Block, Borders, Paragraph},
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 用户名
                Constraint::Length(3), // 昵称
                Constraint::Length(3), // 邮箱
                Constraint::Length(3), // 密码
                Constraint::Length(3), // 确认密码
                Constraint::Length(3), // 注册按钮
            ])
            .margin(2)
            .split(area);

        // 用户名
        InputField::new("用户名 (3-32字符)")
            .with_value(&self.username)
            .focused(self.focus_index == 0)
            .render(frame, chunks[0]);

        // 昵称
        InputField::new("昵称 (2-64字符)")
            .with_value(&self.nickname)
            .focused(self.focus_index == 1)
            .render(frame, chunks[1]);

        // 邮箱
        InputField::new("邮箱")
            .with_value(&self.email)
            .focused(self.focus_index == 2)
            .render(frame, chunks[2]);

        // 密码
        InputField::new("密码 (8-128字符)")
            .with_value(&self.password)
            .password()
            .focused(self.focus_index == 3)
            .render(frame, chunks[3]);

        // 确认密码
        InputField::new("确认密码")
            .with_value(&self.confirm_password)
            .password()
            .focused(self.focus_index == 4)
            .render(frame, chunks[4]);

        // 注册按钮
        let button_style = if self.focus_index == 5 {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };

        let button = Paragraph::new(if self.is_loading {
            "注册中..."
        } else {
            "[ 注册 ]"
        })
        .style(button_style)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(button, chunks[5]);
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        use crate::ui::HelpPanel;

        HelpPanel::new("注册页面帮助")
            .add_item("Tab", "切换到下一个输入框")
            .add_item("Shift+Tab", "切换到上一个输入框")
            .add_item("Enter", "提交注册")
            .add_item("Esc", "返回登录页面")
            .add_item("Backspace", "删除字符")
            .add_item("?", "显示/隐藏帮助")
            .render(frame, area);
    }
}

#[derive(Debug, Clone)]
pub enum RegisterAction {
    None,
    Register {
        username: String,
        nickname: String,
        password: String,
        email: String,
    },
    Back,
}
