use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// 确认对话框
pub struct ConfirmDialog {
    /// 标题
    title: String,
    /// 消息内容
    message: String,
    /// 当前选中的选项（0=否，1=是）
    selected: usize,
    /// 选项文本
    options: Vec<String>,
}

impl ConfirmDialog {
    /// 创建新的确认对话框
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            selected: 1, // 默认选中"是"
            options: vec!["否".to_string(), "是".to_string()],
        }
    }

    /// 切换到下一个选项
    pub fn next_option(&mut self) {
        self.selected = (self.selected + 1) % self.options.len();
    }

    /// 切换到上一个选项
    pub fn prev_option(&mut self) {
        if self.selected == 0 {
            self.selected = self.options.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    /// 获取当前选中的选项索引
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// 是否选择了确认（是）
    pub fn is_confirmed(&self) -> bool {
        self.selected == 1
    }

    /// 渲染对话框
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // 计算对话框大小（居中）
        let popup_area = centered_rect(50, 30, area);

        // 清除背景
        frame.render_widget(Clear, popup_area);

        // 创建对话框内容
        let mut text = Text::from(vec![
            Line::from(""),
            Line::from(self.message.clone()),
            Line::from(""),
        ]);

        // 添加选项
        let options_line: Vec<Span> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, opt)| {
                if i == self.selected {
                    Span::styled(
                        format!(" [{}] ", opt),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(format!("  {}  ", opt), Style::default().fg(Color::Gray))
                }
            })
            .collect();

        text.extend(vec![
            Line::from(options_line),
            Line::from(""),
            Line::from(vec![
                Span::styled("← →", Style::default().fg(Color::Yellow)),
                Span::raw(" 切换 "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" 确认 "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" 取消"),
            ]),
        ]);

        let paragraph = Paragraph::new(text).alignment(Alignment::Center).block(
            Block::default()
                .title(self.title.clone())
                .borders(Borders::ALL)
                .border_style(Color::Yellow),
        );

        frame.render_widget(paragraph, popup_area);
    }
}

/// 创建居中矩形
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// 退出确认对话框
pub struct QuitConfirmDialog {
    dialog: ConfirmDialog,
}

impl QuitConfirmDialog {
    pub fn new() -> Self {
        Self {
            dialog: ConfirmDialog::new(" 确认退出 ", "确定要退出 Antarctica 吗？"),
        }
    }

    pub fn next_option(&mut self) {
        self.dialog.next_option();
    }

    pub fn prev_option(&mut self) {
        self.dialog.prev_option();
    }

    pub fn is_confirmed(&self) -> bool {
        self.dialog.is_confirmed()
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.dialog.render(frame, area);
    }
}

/// 首次运行欢迎对话框
pub struct WelcomeDialog {
    /// 服务器地址输入
    server_url: String,
}

impl WelcomeDialog {
    pub fn new(default_url: &str) -> Self {
        Self {
            server_url: default_url.to_string(),
        }
    }

    /// 获取当前输入的服务器地址
    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    /// 处理键盘输入
    pub fn handle_input(&mut self, c: char) {
        self.server_url.push(c);
    }

    /// 处理退格键
    pub fn handle_backspace(&mut self) {
        self.server_url.pop();
    }

    /// 清空输入
    pub fn clear(&mut self) {
        self.server_url.clear();
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, config_path: &str) {
        // 计算对话框大小（居中，更大的尺寸）
        let popup_area = centered_rect(70, 55, area);

        // 清除背景
        frame.render_widget(Clear, popup_area);

        // 在对话框内部创建布局
        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(12), // 文本内容区域
                Constraint::Length(3),  // 输入框
                Constraint::Length(2),  // 提示信息
            ])
            .margin(1)
            .split(popup_area);

        // 创建欢迎消息内容
        let text = Text::from(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "欢迎使用 Antarctica CLI",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("这是您第一次运行 Antarctica CLI。"),
            Line::from("请输入服务器地址后按 Enter 继续。"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "配置文件路径: ",
                Style::default().fg(Color::Gray),
            )]),
            Line::from(vec![Span::styled(
                config_path,
                Style::default().fg(Color::Gray),
            )]),
            Line::from(""),
        ]);

        let content = Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(content, inner_layout[0]);

        // 渲染输入框
        let input_block = Block::default()
            .title("服务器地址")
            .borders(Borders::ALL)
            .border_style(Color::Yellow);

        let input = Paragraph::new(self.server_url.clone())
            .block(input_block)
            .alignment(Alignment::Left);

        frame.render_widget(input, inner_layout[1]);

        // 渲染提示信息
        let hint = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(" 确认  "),
                Span::styled("Backspace", Style::default().fg(Color::Yellow)),
                Span::raw(" 删除  "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw(" 使用默认值"),
            ]),
        ])
        .alignment(Alignment::Center);

        frame.render_widget(hint, inner_layout[2]);

        // 渲染外边框
        let outer_block = Block::default()
            .title(" 首次运行配置 ")
            .borders(Borders::ALL)
            .border_style(Color::Cyan);

        frame.render_widget(outer_block, popup_area);
    }
}

/// 错误对话框
pub struct ErrorDialog {
    /// 错误码
    error_code: u32,
    /// 错误消息
    message: String,
    /// 详细信息（可选）
    detail: Option<String>,
}

impl ErrorDialog {
    pub fn new(error_code: u32, message: impl Into<String>) -> Self {
        Self {
            error_code,
            message: message.into(),
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // 计算对话框大小（居中，根据内容调整大小）
        let popup_area = centered_rect(60, 40, area);

        // 在对话框内部创建布局
        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // 错误标题和消息
                Constraint::Min(0),    // 详细信息（可扩展）
                Constraint::Length(2), // 提示信息
            ])
            .margin(1)
            .split(popup_area);

        // 渲染外边框
        let outer_block = Block::default()
            .title(" 错误 ")
            .borders(Borders::ALL)
            .border_style(Color::Red);
        frame.render_widget(outer_block, popup_area);

        // 构建错误消息内容（标题部分）
        let header_text = Text::from(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "操作失败",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(self.message.clone()),
            Line::from(""),
            Line::from(vec![
                Span::styled("错误码: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    self.error_code.to_string(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ]);

        let header = Paragraph::new(header_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(header, inner_layout[0]);

        // 如果有详细信息，添加开发者信息区域
        if let Some(ref detail) = self.detail {
            let mut detail_lines = vec![
                Line::from(vec![Span::styled(
                    "--- 开发者信息 ---",
                    Style::default().fg(Color::Gray),
                )]),
                Line::from(""),
            ];

            // 将详细信息分行显示，限制长度避免溢出
            // 计算可用宽度（考虑边距）
            let available_width = area.width.saturating_sub(6) as usize;
            let max_width = available_width.max(50);

            for line in detail.lines().take(5) {
                // 截断过长的行
                let truncated = if line.len() > max_width {
                    format!("{}...", &line[..max_width.saturating_sub(3)])
                } else {
                    line.to_string()
                };
                detail_lines.push(Line::from(vec![Span::styled(
                    truncated,
                    Style::default().fg(Color::Gray),
                )]));
            }

            let detail_text = Text::from(detail_lines);
            let detail_widget = Paragraph::new(detail_text)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .block(Block::default().borders(Borders::NONE));

            frame.render_widget(detail_widget, inner_layout[1]);
        }

        // 渲染提示信息
        let hint = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "按任意键关闭",
                Style::default().fg(Color::Green),
            )]),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));

        frame.render_widget(hint, inner_layout[2]);
    }
}

/// 成功对话框
pub struct SuccessDialog {
    /// 标题
    title: String,
    /// 消息内容
    message: String,
}

impl SuccessDialog {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            title: " 成功 ".to_string(),
            message: message.into(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let popup_area = centered_rect(50, 30, area);

        frame.render_widget(Clear, popup_area);

        let text = Text::from(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "✓",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(self.message.clone()),
            Line::from(""),
            Line::from(vec![Span::styled(
                "按任意键关闭",
                Style::default().fg(Color::Green),
            )]),
        ]);

        let paragraph = Paragraph::new(text).alignment(Alignment::Center).block(
            Block::default()
                .title(self.title.clone())
                .borders(Borders::ALL)
                .border_style(Color::Green),
        );

        frame.render_widget(paragraph, popup_area);
    }
}

/// 等待对话框
pub struct LoadingDialog {
    /// 消息内容
    message: String,
}

impl LoadingDialog {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let popup_area = centered_rect(50, 25, area);

        frame.render_widget(Clear, popup_area);

        let text = Text::from(vec![
            Line::from(""),
            Line::from(vec![Span::styled("⏳", Style::default().fg(Color::Yellow))]),
            Line::from(""),
            Line::from(self.message.clone()),
            Line::from(""),
            Line::from(vec![Span::styled(
                "请稍候...",
                Style::default().fg(Color::Gray),
            )]),
        ]);

        let paragraph = Paragraph::new(text).alignment(Alignment::Center).block(
            Block::default()
                .title(" 处理中 ")
                .borders(Borders::ALL)
                .border_style(Color::Yellow),
        );

        frame.render_widget(paragraph, popup_area);
    }
}
