use crate::{
    api::{BlogDetail, Visibility},
    components::{ConfirmDialog, ErrorDialog, LoadingDialog, SuccessDialog},
    events::Event,
    ui::{FooterBar, MainLayout, TitleBar},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct BlogEditPage {
    blog_id: Option<String>,
    title: String,
    content: String,
    original_title: String,
    original_content: String,
    original_is_public: bool,
    is_public: bool,
    mode: EditMode,
    focus_area: FocusArea,
    preview_mode: PreviewMode,
    cursor_char_index: usize,
    scroll_offset: usize,
    title_char_index: usize,
    show_help: bool,
    is_loading: bool,
    is_saving: bool,
    error: Option<String>,
    success_message: Option<String>,
    show_success: bool,
    show_error: bool,
    show_confirm_exit: bool,
    confirm_exit_dialog: ConfirmDialog,
    is_new: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EditMode {
    Normal,
    Insert,
    Preview,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusArea {
    Title,
    Content,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PreviewMode {
    Source,
    Rendered,
}

impl BlogEditPage {
    pub fn new() -> Self {
        Self {
            blog_id: None,
            title: String::new(),
            content: String::new(),
            original_title: String::new(),
            original_content: String::new(),
            original_is_public: false,
            is_public: false,
            mode: EditMode::Normal,
            focus_area: FocusArea::Title,
            preview_mode: PreviewMode::Rendered,
            cursor_char_index: 0,
            scroll_offset: 0,
            title_char_index: 0,
            show_help: false,
            is_loading: false,
            is_saving: false,
            error: None,
            success_message: None,
            show_success: false,
            show_error: false,
            show_confirm_exit: false,
            confirm_exit_dialog: ConfirmDialog::new(
                " 确认退出 ",
                "确定要退出编辑吗？未保存的更改将丢失。",
            ),
            is_new: true,
        }
    }

    pub fn new_for_edit(blog: BlogDetail) -> Self {
        Self {
            blog_id: Some(blog.id.clone()),
            title: blog.title.clone(),
            content: blog.content.clone(),
            original_title: blog.title,
            original_content: blog.content,
            original_is_public: blog.visibility == Visibility::Public,
            is_public: blog.visibility == Visibility::Public,
            mode: EditMode::Normal,
            focus_area: FocusArea::Title,
            preview_mode: PreviewMode::Rendered,
            cursor_char_index: 0,
            scroll_offset: 0,
            title_char_index: 0,
            show_help: false,
            is_loading: false,
            is_saving: false,
            error: None,
            success_message: None,
            show_success: false,
            show_error: false,
            show_confirm_exit: false,
            confirm_exit_dialog: ConfirmDialog::new(
                " 确认退出 ",
                "确定要退出编辑吗？未保存的更改将丢失。",
            ),
            is_new: false,
        }
    }

    pub fn set_saving(&mut self, saving: bool) {
        self.is_saving = saving;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.show_error = true;
        self.is_saving = false;
    }

    pub fn set_success(&mut self, message: String) {
        self.success_message = Some(message);
        self.show_success = true;
        self.is_saving = false;
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.title != self.original_title
            || self.content != self.original_content
            || self.is_public != self.original_is_public
    }

    pub fn handle_event(&mut self, event: Event) -> BlogEditAction {
        if self.show_success {
            match event {
                Event::Key(_) => {
                    self.show_success = false;
                    self.success_message = None;
                    return BlogEditAction::SaveSuccess;
                }
                _ => return BlogEditAction::None,
            }
        }

        if self.show_error {
            match event {
                Event::Key(_) => {
                    self.show_error = false;
                    self.error = None;
                }
                _ => return BlogEditAction::None,
            }
        }

        if self.show_confirm_exit {
            return self.handle_confirm_exit_event(event);
        }

        if self.show_help {
            match event {
                Event::Key(key) if key.code == KeyCode::Char('?') || key.code == KeyCode::Esc => {
                    self.show_help = false;
                    return BlogEditAction::None;
                }
                _ => return BlogEditAction::None,
            }
        }

        if self.is_saving {
            return BlogEditAction::None;
        }

        match event {
            Event::Key(key) => self.handle_key(key),
            _ => BlogEditAction::None,
        }
    }

    fn handle_confirm_exit_event(&mut self, event: Event) -> BlogEditAction {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Left => {
                    self.confirm_exit_dialog.prev_option();
                    BlogEditAction::None
                }
                KeyCode::Right => {
                    self.confirm_exit_dialog.next_option();
                    BlogEditAction::None
                }
                KeyCode::Enter => {
                    if self.confirm_exit_dialog.is_confirmed() {
                        self.show_confirm_exit = false;
                        BlogEditAction::Back
                    } else {
                        self.show_confirm_exit = false;
                        BlogEditAction::None
                    }
                }
                KeyCode::Esc => {
                    self.show_confirm_exit = false;
                    BlogEditAction::None
                }
                _ => BlogEditAction::None,
            },
            _ => BlogEditAction::None,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> BlogEditAction {
        match self.mode {
            EditMode::Normal => self.handle_normal_mode(key),
            EditMode::Insert => self.handle_insert_mode(key),
            EditMode::Preview => self.handle_preview_mode(key),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> BlogEditAction {
        match key.code {
            KeyCode::Char('?') => {
                self.show_help = true;
                BlogEditAction::None
            }
            KeyCode::Esc => {
                if self.has_unsaved_changes() {
                    self.show_confirm_exit = true;
                    BlogEditAction::None
                } else {
                    BlogEditAction::Back
                }
            }
            KeyCode::Tab => {
                self.focus_area = if self.focus_area == FocusArea::Title {
                    FocusArea::Content
                } else {
                    FocusArea::Title
                };
                BlogEditAction::None
            }
            KeyCode::Char('i') => {
                self.mode = EditMode::Insert;
                BlogEditAction::None
            }
            KeyCode::Char('p') => {
                self.is_public = !self.is_public;
                BlogEditAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.focus_area == FocusArea::Content {
                    self.scroll_offset = self.scroll_offset.saturating_add(1);
                }
                BlogEditAction::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.focus_area == FocusArea::Content {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                }
                BlogEditAction::None
            }
            KeyCode::Char('s') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return self.save_blog();
                }
                BlogEditAction::None
            }
            KeyCode::Char('t') => {
                self.mode = EditMode::Preview;
                self.scroll_offset = 0;
                BlogEditAction::None
            }
            KeyCode::F(10) => {
                return self.save_blog();
            }
            _ => BlogEditAction::None,
        }
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) -> BlogEditAction {
        match key.code {
            KeyCode::Esc => {
                self.mode = EditMode::Normal;
                BlogEditAction::None
            }
            KeyCode::Enter => {
                if self.focus_area == FocusArea::Content {
                    self.content.push('\n');
                    self.cursor_char_index += 1;
                }
                BlogEditAction::None
            }
            KeyCode::Backspace => {
                if self.focus_area == FocusArea::Title {
                    if !self.title.is_empty() && self.title_char_index > 0 {
                        let char_count = self.title.chars().count();
                        if self.title_char_index <= char_count {
                            let chars: Vec<char> = self.title.chars().collect();
                            let mut new_title = String::new();
                            for (i, c) in chars.iter().enumerate() {
                                if i != self.title_char_index - 1 {
                                    new_title.push(*c);
                                }
                            }
                            self.title = new_title;
                        }
                        self.title_char_index -= 1;
                    }
                } else if !self.content.is_empty() && self.cursor_char_index > 0 {
                    let char_count = self.content.chars().count();
                    if self.cursor_char_index <= char_count {
                        let chars: Vec<char> = self.content.chars().collect();
                        let mut new_content = String::new();
                        for (i, c) in chars.iter().enumerate() {
                            if i != self.cursor_char_index - 1 {
                                new_content.push(*c);
                            }
                        }
                        self.content = new_content;
                    }
                    self.cursor_char_index -= 1;
                }
                BlogEditAction::None
            }
            KeyCode::Char(c) => {
                if self.focus_area == FocusArea::Title {
                    if !c.is_control() {
                        let char_count = self.title.chars().count();
                        let insert_pos = self.title_char_index.min(char_count);
                        let chars: Vec<char> = self.title.chars().collect();
                        let mut new_title = String::new();
                        for (i, ch) in chars.iter().enumerate() {
                            if i == insert_pos {
                                new_title.push(c);
                            }
                            new_title.push(*ch);
                        }
                        if insert_pos >= char_count {
                            new_title.push(c);
                        }
                        self.title = new_title;
                        self.title_char_index = insert_pos + 1;
                    }
                } else {
                    if !c.is_control() {
                        let char_count = self.content.chars().count();
                        let insert_pos = self.cursor_char_index.min(char_count);
                        let chars: Vec<char> = self.content.chars().collect();
                        let mut new_content = String::new();
                        for (i, ch) in chars.iter().enumerate() {
                            if i == insert_pos {
                                new_content.push(c);
                            }
                            new_content.push(*ch);
                        }
                        if insert_pos >= char_count {
                            new_content.push(c);
                        }
                        self.content = new_content;
                        self.cursor_char_index = insert_pos + 1;
                    }
                }
                BlogEditAction::None
            }
            KeyCode::Left => {
                if self.focus_area == FocusArea::Title {
                    self.title_char_index = self.title_char_index.saturating_sub(1);
                } else {
                    self.cursor_char_index = self.cursor_char_index.saturating_sub(1);
                }
                BlogEditAction::None
            }
            KeyCode::Right => {
                if self.focus_area == FocusArea::Title {
                    self.title_char_index =
                        (self.title_char_index + 1).min(self.title.chars().count());
                } else {
                    self.cursor_char_index =
                        (self.cursor_char_index + 1).min(self.content.chars().count());
                }
                BlogEditAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    if self.focus_area == FocusArea::Content {
                        self.scroll_offset = self.scroll_offset.saturating_add(1);
                    }
                }
                BlogEditAction::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    if self.focus_area == FocusArea::Content {
                        self.scroll_offset = self.scroll_offset.saturating_sub(1);
                    }
                }
                BlogEditAction::None
            }
            _ => BlogEditAction::None,
        }
    }

    fn handle_preview_mode(&mut self, key: KeyEvent) -> BlogEditAction {
        match key.code {
            KeyCode::Esc => {
                self.mode = EditMode::Normal;
                BlogEditAction::None
            }
            KeyCode::Char('?') => {
                self.show_help = true;
                BlogEditAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
                BlogEditAction::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                BlogEditAction::None
            }
            KeyCode::Char('t') => {
                self.preview_mode = if self.preview_mode == PreviewMode::Source {
                    PreviewMode::Rendered
                } else {
                    PreviewMode::Source
                };
                self.scroll_offset = 0;
                BlogEditAction::None
            }
            KeyCode::F(10) => {
                if let Some(ref blog_id) = self.blog_id {
                    return BlogEditAction::Upload(blog_id.clone());
                } else {
                    self.error = Some("请先保存博客后再上传".to_string());
                }
                BlogEditAction::None
            }
            _ => BlogEditAction::None,
        }
    }

    fn save_blog(&mut self) -> BlogEditAction {
        if self.title.trim().is_empty() {
            self.error = Some("标题不能为空".to_string());
            return BlogEditAction::None;
        }
        BlogEditAction::Save {
            blog_id: self.blog_id.clone(),
            title: self.title.clone(),
            content: self.content.clone(),
            is_public: self.is_public,
            is_new: self.is_new,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let layout = MainLayout::new(frame);

        TitleBar::new("Antarctica Blog Editor").render(frame, layout.title_area);

        self.render_main(frame, layout.main_area);

        let left_info = vec![
            format!(
                "模式: {}",
                match self.mode {
                    EditMode::Normal => "浏览",
                    EditMode::Insert => "编辑",
                    EditMode::Preview => "预览",
                }
            ),
            format!(
                "焦点: {}",
                if self.focus_area == FocusArea::Title {
                    "标题"
                } else {
                    "内容"
                }
            ),
            if self.is_new {
                "新建".to_string()
            } else {
                "编辑".to_string()
            },
        ];

        let visibility_str = if self.is_public { "公开" } else { "私有" };

        let hints = if self.mode == EditMode::Preview {
            vec![
                ("↑/k".to_string(), "上滚".to_string()),
                ("↓/j".to_string(), "下滚".to_string()),
                ("t".to_string(), "切换视图".to_string()),
                ("F10".to_string(), "上传".to_string()),
                ("Esc".to_string(), "返回".to_string()),
                ("?".to_string(), "帮助".to_string()),
            ]
        } else {
            let mut hints = vec![
                ("i".to_string(), "编辑".to_string()),
                ("t".to_string(), "预览".to_string()),
                ("Esc".to_string(), "退出".to_string()),
                ("p".to_string(), visibility_str.to_string()),
                ("Tab".to_string(), "切换焦点".to_string()),
                ("Ctrl+S".to_string(), "保存".to_string()),
            ];
            if self.blog_id.is_some() {
                hints.push(("F10".to_string(), "上传".to_string()));
            }
            hints.push(("?".to_string(), "帮助".to_string()));
            hints
        };

        FooterBar::new()
            .with_left_info(left_info)
            .with_right_hints(hints)
            .render(frame, layout.footer_left, layout.footer_right);

        if self.is_saving {
            LoadingDialog::new("正在保存...").render(frame, area);
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

        if self.show_confirm_exit {
            self.confirm_exit_dialog.render(frame, area);
        }
    }

    fn render_main(&self, frame: &mut Frame, area: Rect) {
        if self.mode == EditMode::Preview {
            self.render_preview(frame, area);
            return;
        }

        let title_height = 3u16;
        let content_height = area.height.saturating_sub(title_height);

        let title_area = Rect::new(area.x, area.y, area.width, title_height);
        let content_area = Rect::new(area.x, area.y + title_height, area.width, content_height);

        self.render_title(frame, title_area);
        self.render_content(frame, content_area);
    }

    fn render_preview(&self, frame: &mut Frame, area: Rect) {
        let mode_title = match self.preview_mode {
            PreviewMode::Source => "源码预览",
            PreviewMode::Rendered => "渲染预览",
        };

        let content = match self.preview_mode {
            PreviewMode::Source => &self.content,
            PreviewMode::Rendered => &self.content,
        };

        let block = Block::default()
            .title(mode_title)
            .borders(Borders::ALL)
            .border_style(Color::Cyan);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let paragraph = Paragraph::new(content.clone())
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset as u16, 0));

        frame.render_widget(paragraph, inner_area);
    }

    fn char_display_width(c: char) -> usize {
        if c.is_ascii() {
            1
        } else {
            2
        }
    }

    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus_area == FocusArea::Title;
        let border_style = if is_focused {
            if self.mode == EditMode::Insert {
                Color::Green
            } else {
                Color::Yellow
            }
        } else {
            Color::Cyan
        };

        let mode_indicator = if self.mode == EditMode::Insert {
            " [编辑模式]"
        } else {
            ""
        };

        let block = Block::default()
            .title(format!("标题{}", mode_indicator))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let title_text = if self.title.is_empty() {
            if is_focused && self.mode == EditMode::Normal {
                "按 i 开始编辑".to_string()
            } else if is_focused {
                "".to_string()
            } else {
                "按 Tab 切换到标题".to_string()
            }
        } else {
            self.title.clone()
        };

        let style = if self.title.is_empty() {
            Style::default().fg(Color::Gray)
        } else {
            Style::default().fg(Color::White)
        };

        let paragraph = Paragraph::new(title_text)
            .style(style)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, inner_area);

        if is_focused && self.mode == EditMode::Insert && !self.title.is_empty() {
            let char_count = self.title.chars().count();
            let cursor_idx = self.title_char_index.min(char_count);
            let display_width: usize = self
                .title
                .chars()
                .take(cursor_idx)
                .map(Self::char_display_width)
                .sum();
            let cursor_x = display_width as u16;
            frame.set_cursor_position((inner_area.x + cursor_x, inner_area.y));
        }
    }

    fn render_content(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus_area == FocusArea::Content;
        let border_style = if is_focused {
            if self.mode == EditMode::Insert {
                Color::Green
            } else {
                Color::Yellow
            }
        } else {
            Color::Cyan
        };

        let visibility_indicator = if self.is_public {
            "👁  公开"
        } else {
            "🔒 私有"
        };

        let block = Block::default()
            .title(format!("内容 - {}", visibility_indicator))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let line_count = self.content.lines().count().max(1);
        let visible_lines = inner_area.height as usize;
        let max_scroll = if line_count > visible_lines {
            line_count - visible_lines
        } else {
            0
        };
        let scroll = self.scroll_offset.min(max_scroll);

        let left_width = 4u16;
        let text_area = Rect::new(
            inner_area.x + left_width + 1,
            inner_area.y,
            inner_area.width.saturating_sub(left_width + 1),
            inner_area.height,
        );

        let line_numbers: Vec<Line> = (1..=line_count)
            .map(|i| {
                Line::from(Span::styled(
                    format!("{:>4}", i),
                    Style::default().fg(Color::DarkGray),
                ))
            })
            .collect();

        let line_numbers_widget = Paragraph::new(line_numbers).alignment(Alignment::Right);
        frame.render_widget(
            line_numbers_widget,
            Rect::new(inner_area.x, inner_area.y, left_width, inner_area.height),
        );

        let display_content = if self.content.is_empty() {
            if is_focused && self.mode == EditMode::Normal {
                "按 i 开始编辑".to_string()
            } else if is_focused {
                "".to_string()
            } else {
                "按 Tab 切换到内容".to_string()
            }
        } else {
            self.content.clone()
        };

        let style = if self.content.is_empty() {
            Style::default().fg(Color::Gray)
        } else {
            Style::default().fg(Color::White)
        };

        let paragraph = Paragraph::new(display_content)
            .style(style)
            .wrap(Wrap { trim: false })
            .scroll((scroll as u16, 0));

        frame.render_widget(paragraph, text_area);

        if is_focused && self.mode == EditMode::Insert && !self.content.is_empty() {
            let char_count = self.content.chars().count();
            let cursor_idx = self.cursor_char_index.min(char_count);

            let mut current_row = 0usize;
            let mut current_col = 0usize;
            let mut char_pos = 0usize;

            for c in self.content.chars() {
                if char_pos >= cursor_idx {
                    break;
                }
                if c == '\n' {
                    current_row += 1;
                    current_col = 0;
                } else {
                    current_col += Self::char_display_width(c);
                }
                char_pos += 1;
            }

            let row = current_row.saturating_sub(scroll);
            let col = current_col;

            let cursor_x =
                text_area.x + (col.min(usize::from(text_area.width).saturating_sub(1)) as u16);
            let cursor_y =
                text_area.y + (row.min(usize::from(text_area.height).saturating_sub(1)) as u16);
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        use crate::ui::HelpPanel;

        let help = if self.mode == EditMode::Preview {
            HelpPanel::new("博客预览帮助")
                .add_item("↑/k", "向上滚动")
                .add_item("↓/j", "向下滚动")
                .add_item("t", "切换源码/渲染视图")
                .add_item("F10", "上传博客")
                .add_item("Esc", "返回编辑")
                .add_item("?", "显示/隐藏帮助")
        } else {
            HelpPanel::new("博客编辑器帮助")
                .add_item("i", "进入/退出编辑模式")
                .add_item("t", "进入预览模式")
                .add_item("Esc", "退出编辑模式或返回")
                .add_item("Tab", "切换标题/内容焦点")
                .add_item("p", "切换公开/私有")
                .add_item("↑/k", "上滚内容")
                .add_item("↓/j", "下滚内容")
                .add_item("Ctrl+S", "保存博客")
                .add_item("?", "显示/隐藏帮助")
        };
        help.render(frame, area);
    }
}

#[derive(Debug, Clone)]
pub enum BlogEditAction {
    None,
    Back,
    SaveSuccess,
    Save {
        blog_id: Option<String>,
        title: String,
        content: String,
        is_public: bool,
        is_new: bool,
    },
    Upload(String),
}
