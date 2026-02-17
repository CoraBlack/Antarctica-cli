use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// 主布局结构
pub struct MainLayout {
    /// 标题区域（3格高）
    pub title_area: Rect,
    /// 主内容区域
    pub main_area: Rect,
    /// 辅助栏区域（3格高）
    pub footer_area: Rect,
    /// 左侧辅助信息区域
    pub footer_left: Rect,
    /// 右侧操作提示区域
    pub footer_right: Rect,
}

impl MainLayout {
    /// 计算布局
    pub fn new(frame: &Frame) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 标题区：3格高
                Constraint::Min(0),    // 主内容区：填充剩余空间
                Constraint::Length(3), // 辅助栏：3格高
            ])
            .split(frame.area());

        // 分割辅助栏为左右两部分
        let footer_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);

        Self {
            title_area: chunks[0],
            main_area: chunks[1],
            footer_area: chunks[2],
            footer_left: footer_chunks[0],
            footer_right: footer_chunks[1],
        }
    }
}

/// 标题栏组件
pub struct TitleBar {
    title: String,
}

impl TitleBar {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let title = Paragraph::new(self.title.clone())
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Color::Cyan),
            );

        frame.render_widget(title, area);
    }
}

/// 辅助栏组件
pub struct FooterBar {
    left_info: Vec<String>,
    right_hints: Vec<(String, String)>, // (按键, 描述)
}

impl FooterBar {
    pub fn new() -> Self {
        Self {
            left_info: vec![],
            right_hints: vec![],
        }
    }

    pub fn with_left_info(mut self, info: Vec<String>) -> Self {
        self.left_info = info;
        self
    }

    pub fn with_right_hints(mut self, hints: Vec<(String, String)>) -> Self {
        self.right_hints = hints;
        self
    }

    pub fn render(&self, frame: &mut Frame, left_area: Rect, right_area: Rect) {
        // 左侧辅助信息
        let left_content = if self.left_info.is_empty() {
            "暂无信息".to_string()
        } else {
            self.left_info.join(" | ")
        };

        let left = Paragraph::new(left_content)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Color::Gray)
                    .title("信息"),
            );

        frame.render_widget(left, left_area);

        // 右侧操作提示
        let mut hint_spans = vec![];
        for (i, (key, desc)) in self.right_hints.iter().enumerate() {
            if i > 0 {
                hint_spans.push(Span::raw(" | "));
            }
            hint_spans.push(Span::styled(
                format!("{}: {}", key, desc),
                Style::default().fg(Color::Yellow),
            ));
        }

        if hint_spans.is_empty() {
            hint_spans.push(Span::raw("按 ? 查看帮助"));
        }

        let right_text = Text::from(Line::from(hint_spans));
        let right = Paragraph::new(right_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Color::Gray)
                    .title("操作")
                    .title_alignment(Alignment::Right),
            );

        frame.render_widget(right, right_area);
    }
}

/// 帮助面板
pub struct HelpPanel {
    title: String,
    content: Vec<(String, String)>, // (操作, 描述)
}

impl HelpPanel {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: vec![],
        }
    }

    pub fn add_item(mut self, key: impl Into<String>, desc: impl Into<String>) -> Self {
        self.content.push((key.into(), desc.into()));
        self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let text: Vec<Line> = self
            .content
            .iter()
            .map(|(key, desc)| {
                Line::from(vec![
                    Span::styled(
                        format!("{:<12}", key),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(desc),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(self.title.clone())
                    .borders(Borders::ALL)
                    .border_style(Color::Yellow),
            )
            .wrap(Wrap { trim: true });

        // 创建居中弹出框
        let popup_area = centered_rect(60, 70, area);
        frame.render_widget(Clear, popup_area);
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

/// 输入框组件
pub struct InputField {
    label: String,
    value: String,
    is_password: bool,
    is_focused: bool,
}

impl InputField {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: String::new(),
            is_password: false,
            is_focused: false,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn password(mut self) -> Self {
        self.is_password = true;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let display_value = if self.is_password {
            "*".repeat(self.value.len())
        } else {
            self.value.clone()
        };

        let block = Block::default()
            .title(self.label.clone())
            .borders(Borders::ALL)
            .border_style(if self.is_focused {
                Color::Yellow
            } else {
                Color::Gray
            });

        let paragraph = Paragraph::new(display_value).block(block);
        frame.render_widget(paragraph, area);
    }
}
