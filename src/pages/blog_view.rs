use crate::{
    api::{BlogDetail, Visibility},
    components::{ErrorDialog, LoadingDialog, MarkdownRenderer, SuccessDialog},
    events::Event,
    ui::{FooterBar, MainLayout, TitleBar},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    style::Color,
    style::Style,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct BlogViewPage {
    blog: Option<BlogDetail>,
    current_username: Option<String>,
    view_mode: ViewMode,
    scroll_offset: usize,
    max_scroll: usize,
    show_help: bool,
    is_loading: bool,
    error: Option<String>,
    success_message: Option<String>,
    show_success: bool,
    show_error: bool,
    is_uploading: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ViewMode {
    Source,
    Rendered,
}

impl BlogViewPage {
    pub fn new() -> Self {
        Self {
            blog: None,
            current_username: None,
            view_mode: ViewMode::Rendered,
            scroll_offset: 0,
            max_scroll: 0,
            show_help: false,
            is_loading: true,
            error: None,
            success_message: None,
            show_success: false,
            show_error: false,
            is_uploading: false,
        }
    }

    pub fn set_current_username(&mut self, username: String) {
        self.current_username = Some(username);
    }

    pub fn set_blog(&mut self, blog: BlogDetail) {
        self.blog = Some(blog);
        self.is_loading = false;
        self.scroll_offset = 0;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.is_loading = false;
    }

    pub fn set_success(&mut self, message: String) {
        self.success_message = Some(message);
        self.show_success = true;
        self.is_uploading = false;
    }

    pub fn set_upload_error(&mut self, error: String) {
        self.error = Some(error);
        self.show_error = true;
        self.is_uploading = false;
    }

    pub fn set_uploading(&mut self, uploading: bool) {
        self.is_uploading = uploading;
    }

    fn is_blog_owner(&self) -> bool {
        if let (Some(blog), Some(username)) = (&self.blog, &self.current_username) {
            &blog.author.username == username
        } else {
            false
        }
    }

    pub fn handle_event(&mut self, event: Event) -> BlogViewAction {
        if self.show_success {
            match event {
                Event::Key(_) => {
                    self.show_success = false;
                    self.success_message = None;
                    return BlogViewAction::None;
                }
                _ => return BlogViewAction::None,
            }
        }

        if self.show_error {
            match event {
                Event::Key(_) => {
                    self.show_error = false;
                    self.error = None;
                    return BlogViewAction::None;
                }
                _ => return BlogViewAction::None,
            }
        }

        if self.show_help {
            match event {
                Event::Key(key) if key.code == KeyCode::Char('?') || key.code == KeyCode::Esc => {
                    self.show_help = false;
                    return BlogViewAction::None;
                }
                _ => return BlogViewAction::None,
            }
        }

        match event {
            Event::Key(key) => self.handle_key(key),
            _ => BlogViewAction::None,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> BlogViewAction {
        match key.code {
            KeyCode::Char('?') => {
                self.show_help = true;
                BlogViewAction::None
            }
            KeyCode::Esc | KeyCode::Char('q') => BlogViewAction::Back,
            KeyCode::Char('t') => {
                self.view_mode = if self.view_mode == ViewMode::Source {
                    ViewMode::Rendered
                } else {
                    ViewMode::Source
                };
                BlogViewAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.scroll_offset < self.max_scroll {
                    self.scroll_offset = self.scroll_offset.saturating_add(1);
                }
                BlogViewAction::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                BlogViewAction::None
            }
            KeyCode::Char('e') => {
                if self.is_blog_owner() {
                    if let Some(ref blog) = self.blog {
                        return BlogViewAction::EditBlog(blog.id.clone());
                    }
                }
                BlogViewAction::None
            }
            KeyCode::F(10) => {
                if self.is_blog_owner() {
                    if let Some(ref blog) = self.blog {
                        return BlogViewAction::UploadBlog(blog.id.clone());
                    }
                }
                BlogViewAction::None
            }
            _ => BlogViewAction::None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = MainLayout::new(frame);

        let title_text = if let Some(ref blog) = self.blog {
            format!("Antarctica-Blog: {}", blog.title)
        } else {
            "Antarctica-Blog".to_string()
        };
        TitleBar::new(title_text).render(frame, layout.title_area);

        self.render_main(frame, layout.main_area);

        let mode_str = match self.view_mode {
            ViewMode::Source => "源码",
            ViewMode::Rendered => "渲染",
        };

        let left_info = if let Some(ref blog) = self.blog {
            vec![
                format!("作者: {}", blog.author.username),
                format!(
                    "可见: {}",
                    if blog.visibility == Visibility::Public {
                        "公开"
                    } else {
                        "私有"
                    }
                ),
            ]
        } else {
            vec![]
        };

        let is_owner = self.is_blog_owner();
        let mut hints = vec![
            ("↑/k".to_string(), "上滚".to_string()),
            ("↓/j".to_string(), "下滚".to_string()),
            ("t".to_string(), "切换视图".to_string()),
        ];
        if is_owner {
            hints.push(("e".to_string(), "编辑".to_string()));
            hints.push(("F10".to_string(), "上传".to_string()));
        }
        hints.push(("q/Esc".to_string(), "返回".to_string()));
        hints.push(("?".to_string(), "帮助".to_string()));

        FooterBar::new()
            .with_left_info(left_info)
            .with_right_hints(hints)
            .render(frame, layout.footer_left, layout.footer_right);

        if self.is_uploading {
            LoadingDialog::new("正在上传...").render(frame, area);
        }

        if self.show_help {
            self.render_help(frame, area);
        }

        if self.show_error {
            if let Some(ref error) = self.error {
                ErrorDialog::new(0, error).render(frame, area);
            }
        }

        if self.show_success {
            if let Some(ref message) = self.success_message {
                SuccessDialog::new(message).render(frame, area);
            }
        }
    }

    fn render_main(&mut self, frame: &mut Frame, area: Rect) {
        if self.is_loading {
            let loading = Paragraph::new("加载中...")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(loading, area);
            return;
        }

        if let Some(ref error) = self.error {
            let error_text = Paragraph::new(error.clone())
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Red));
            frame.render_widget(error_text, area);
            return;
        }

        if let Some(ref blog) = self.blog {
            let mode_title = match self.view_mode {
                ViewMode::Source => "源码模式",
                ViewMode::Rendered => "渲染模式",
            };

            let block = Block::default()
                .title(mode_title)
                .borders(Borders::ALL)
                .border_style(Color::Cyan);

            let inner_area = block.inner(area);
            frame.render_widget(block, area);

            match self.view_mode {
                ViewMode::Source => {
                    let content = &blog.content;
                    let line_count = self.calculate_wrapped_lines(content, inner_area.width);
                    let visible_lines = inner_area.height as usize;
                    self.max_scroll = if line_count > visible_lines {
                        line_count - visible_lines
                    } else {
                        0
                    };

                    if self.scroll_offset > self.max_scroll {
                        self.scroll_offset = self.max_scroll;
                    }

                    let paragraph = Paragraph::new(content.clone())
                        .style(Style::default().fg(Color::White))
                        .wrap(Wrap { trim: false })
                        .scroll((self.scroll_offset as u16, 0));

                    frame.render_widget(paragraph, inner_area);
                }
                ViewMode::Rendered => {
                    // 使用改进的 Markdown 渲染器
                    let rendered = MarkdownRenderer::render(&blog.content);
                    let line_count = rendered.lines.len();
                    let visible_lines = inner_area.height as usize;
                    self.max_scroll = if line_count > visible_lines {
                        line_count - visible_lines
                    } else {
                        0
                    };

                    if self.scroll_offset > self.max_scroll {
                        self.scroll_offset = self.max_scroll;
                    }

                    // 渲染可见行
                    let scroll = self.scroll_offset.min(line_count);
                    let visible_lines: Vec<_> = rendered
                        .lines
                        .iter()
                        .skip(scroll)
                        .take(visible_lines)
                        .cloned()
                        .collect();

                    let text = ratatui::text::Text::from(visible_lines);
                    frame.render_widget(Paragraph::new(text), inner_area);
                }
            }
        }
    }

    fn calculate_wrapped_lines(&self, content: &str, width: u16) -> usize {
        if width == 0 {
            return content.lines().count();
        }

        let width = width as usize;
        let mut total_lines = 0;

        for line in content.lines() {
            if line.is_empty() {
                total_lines += 1;
                continue;
            }

            let line_width: usize = line.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum();
            total_lines += (line_width + width - 1) / width;
        }

        total_lines.max(1)
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        use crate::ui::HelpPanel;

        let mut help = HelpPanel::new("博客预览页帮助")
            .add_item("↑/k", "向上滚动")
            .add_item("↓/j", "向下滚动")
            .add_item("t", "切换源码/渲染视图");

        if self.is_blog_owner() {
            help = help
                .add_item("e", "编辑当前博客")
                .add_item("F10", "上传博客");
        }

        help = help
            .add_item("q/Esc", "返回")
            .add_item("?", "显示/隐藏帮助");

        help.render(frame, area);
    }
}

#[derive(Debug, Clone)]
pub enum BlogViewAction {
    None,
    Back,
    EditBlog(String),
    UploadBlog(String),
}
