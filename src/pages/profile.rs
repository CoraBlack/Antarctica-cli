use crate::{
    api::{BlogListItem, Visibility},
    config::UserInfo,
    events::Event,
    ui::{FooterBar, MainLayout, TitleBar},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub struct ProfilePage {
    user_info: Option<UserInfo>,
    blogs: Vec<BlogListItem>,
    selected_index: usize,
    scroll_offset: usize,
    show_help: bool,
    is_loading: bool,
}

impl ProfilePage {
    pub fn new() -> Self {
        Self {
            user_info: None,
            blogs: vec![],
            selected_index: 0,
            scroll_offset: 0,
            show_help: false,
            is_loading: true,
        }
    }

    pub fn set_user_info(&mut self, user_info: UserInfo) {
        self.user_info = Some(user_info);
    }

    pub fn set_blogs(&mut self, blogs: Vec<BlogListItem>) {
        // println!("DEBUG: set_blogs called with {} blogs", blogs.len());
        // for (i, blog) in blogs.iter().enumerate() {
        //     println!("DEBUG: Blog {}: id={}, title={}", i, blog.id, blog.title);
        // }
        self.blogs = blogs;
        self.is_loading = false;
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn handle_event(&mut self, event: Event) -> ProfileAction {
        if self.show_help {
            match event {
                Event::Key(key) if key.code == KeyCode::Char('?') || key.code == KeyCode::Esc => {
                    self.show_help = false;
                    return ProfileAction::None;
                }
                _ => return ProfileAction::None,
            }
        }

        match event {
            Event::Key(key) => self.handle_key(key),
            _ => ProfileAction::None,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> ProfileAction {
        let total_items = self.blogs.len() + 2;

        match key.code {
            KeyCode::Char('?') => {
                self.show_help = true;
                ProfileAction::None
            }
            KeyCode::Esc | KeyCode::Char('q') => ProfileAction::BackToHome,
            KeyCode::Char('n') => ProfileAction::NewBlog,
            KeyCode::Down | KeyCode::Char('j') => {
                if total_items > 0 {
                    self.selected_index = (self.selected_index + 1) % total_items;
                    self.update_scroll();
                }
                ProfileAction::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if total_items > 0 {
                    self.selected_index = if self.selected_index == 0 {
                        total_items - 1
                    } else {
                        self.selected_index - 1
                    };
                    self.update_scroll();
                }
                ProfileAction::None
            }
            KeyCode::Enter => {
                if self.selected_index == 0 {
                    return ProfileAction::NewBlog;
                } else if self.selected_index == self.blogs.len() + 1 {
                    return ProfileAction::Logout;
                } else if self.selected_index > 0 {
                    if let Some(blog) = self.blogs.get(self.selected_index - 1) {
                        return ProfileAction::ViewBlog(blog.id.clone());
                    }
                }
                ProfileAction::None
            }
            _ => ProfileAction::None,
        }
    }

    fn update_scroll(&mut self) {
        if self.selected_index >= 3 {
            self.scroll_offset = self.selected_index - 2;
        } else {
            self.scroll_offset = 0;
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let layout = MainLayout::new(frame);

        TitleBar::new("Antarctica-Profile").render(frame, layout.title_area);

        self.render_main(frame, layout.main_area);

        let left_info = if let Some(ref user) = self.user_info {
            vec![format!("用户: {}", user.username)]
        } else {
            vec!["未登录".to_string()]
        };

        let hints = if self.selected_index == 0 {
            vec![
                ("↑/k".to_string(), "上移".to_string()),
                ("↓/j".to_string(), "下移".to_string()),
                ("Enter".to_string(), "新建".to_string()),
                ("q/Esc".to_string(), "返回".to_string()),
                ("?".to_string(), "帮助".to_string()),
            ]
        } else if self.selected_index == self.blogs.len() + 1 {
            vec![
                ("↑/k".to_string(), "上移".to_string()),
                ("↓/j".to_string(), "下移".to_string()),
                ("Enter".to_string(), "退出登录".to_string()),
                ("q/Esc".to_string(), "返回".to_string()),
                ("?".to_string(), "帮助".to_string()),
            ]
        } else {
            vec![
                ("↑/k".to_string(), "上移".to_string()),
                ("↓/j".to_string(), "下移".to_string()),
                ("Enter".to_string(), "查看".to_string()),
                ("n".to_string(), "新建文章".to_string()),
                ("q/Esc".to_string(), "返回".to_string()),
                ("?".to_string(), "帮助".to_string()),
            ]
        };

        FooterBar::new()
            .with_left_info(left_info)
            .with_right_hints(hints)
            .render(frame, layout.footer_left, layout.footer_right);

        if self.show_help {
            self.render_help(frame, area);
        }
    }

    fn render_main(&self, frame: &mut Frame, area: Rect) {
        let user_info_height = (area.height as f32 * 0.35) as u16;
        let list_height = area.height.saturating_sub(user_info_height);

        let user_area = Rect::new(area.x, area.y, area.width, user_info_height);
        let list_area = Rect::new(area.x, area.y + user_info_height, area.width, list_height);

        self.render_user_info(frame, user_area);

        if !self.is_loading {
            self.render_blog_list(frame, list_area);
        } else {
            let loading = Paragraph::new("加载中...")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(loading, list_area);
        }
    }

    fn render_user_info(&self, frame: &mut Frame, area: Rect) {
        if let Some(ref user) = self.user_info {
            let block = Block::default()
                .title("个人信息")
                .borders(Borders::ALL)
                .border_style(Color::Cyan);

            let inner_area = block.inner(area);
            frame.render_widget(block, area);

            let content = vec![
                Line::from(vec![
                    Span::styled(
                        "用户名: ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(&user.username),
                ]),
                Line::from(vec![
                    Span::styled(
                        "昵称: ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(&user.nickname),
                ]),
                Line::from(vec![
                    Span::styled(
                        "邮箱: ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(&user.email),
                ]),
                Line::from(vec![
                    Span::styled(
                        "简介: ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(user.bio.as_deref().unwrap_or("暂无简介")),
                ]),
            ];

            let paragraph = Paragraph::new(content)
                .alignment(Alignment::Left)
                .block(Block::default());

            frame.render_widget(paragraph, inner_area);
        }
    }

    fn render_blog_list(&self, frame: &mut Frame, area: Rect) {
        let mut items: Vec<ListItem> = vec![];

        let new_item = ListItem::new(Text::from(vec![Line::from(vec![
            Span::styled(
                "+ ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("新建文章", Style::default().fg(Color::Green)),
        ])]))
        .style(if self.selected_index == 0 {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        });
        items.push(new_item);

        for (i, blog) in self.blogs.iter().enumerate() {
            let is_selected = i + 1 == self.selected_index;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let visibility_icon = if blog.visibility == Visibility::Public {
                "👁"
            } else {
                "🔒"
            };

            let content = Text::from(vec![
                Line::from(vec![Span::styled(
                    &blog.title,
                    style.add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![Span::styled(
                    format!(
                        "  {} | 作者: {} | {}",
                        visibility_icon, blog.author.username, blog.created_at
                    ),
                    if is_selected {
                        style
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                )]),
            ]);

            items.push(ListItem::new(content));
        }

        let logout_item = ListItem::new(Text::from(vec![Line::from(vec![
            Span::styled(
                "x ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("退出登录", Style::default().fg(Color::Red)),
        ])]))
        .style(if self.selected_index == self.blogs.len() + 1 {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        });
        items.push(logout_item);

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("我的文章 ({})", self.blogs.len())),
        );

        frame.render_widget(list, area);
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        use crate::ui::HelpPanel;

        HelpPanel::new("个人信息页帮助")
            .add_item("↑/k", "选择上一项")
            .add_item("↓/j", "选择下一项")
            .add_item("Enter", "执行选中项操作")
            .add_item("n", "新建文章")
            .add_item("q/Esc", "返回主页")
            .add_item("?", "显示/隐藏帮助")
            .render(frame, area);
    }
}

#[derive(Debug, Clone)]
pub enum ProfileAction {
    None,
    BackToHome,
    NewBlog,
    ViewBlog(String),
    Logout,
}
