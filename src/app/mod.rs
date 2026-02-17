use crate::{
    api::ApiClient,
    config::Config,
    events::{Event, EventHandler},
    pages::{
        blog_edit::{BlogEditAction, BlogEditPage},
        blog_view::{BlogViewAction, BlogViewPage},
        home::{HomeAction, HomePage},
        login::{LoginAction, LoginPage},
        profile::{ProfileAction, ProfilePage},
        register::{RegisterAction, RegisterPage},
    },
};
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

/// 应用状态
pub struct App {
    /// 终端
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    /// 事件处理器
    events: EventHandler,
    /// 当前页面
    current_page: Page,
    /// 博客视图的前一个页面（用于正确返回）
    previous_page_before_blog_view: Page,
    /// 配置
    config: Config,
    /// API客户端
    api_client: ApiClient,
    /// 主页
    home_page: HomePage,
    /// 登录页面
    login_page: LoginPage,
    /// 注册页面
    register_page: RegisterPage,
    /// 个人信息页面
    profile_page: ProfilePage,
    /// 博客预览页面
    blog_view_page: BlogViewPage,
    /// 博客编辑页面
    blog_edit_page: BlogEditPage,
    /// 是否需要退出
    should_quit: bool,
    /// 是否是首次运行
    is_first_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Page {
    Home,
    Login,
    Register,
    Profile,
    BlogView,
    BlogEdit,
}

impl App {
    /// 创建新应用
    pub async fn new() -> Result<Self> {
        // 初始化终端
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // 在加载配置前，先检查是否是首次运行
        let is_first_run = Config::is_first_run().unwrap_or(false);

        // 加载配置（首次运行会自动创建配置文件）
        let config = Config::load()?;
        let api_client = ApiClient::new(&config);

        // 创建事件处理器
        let events = EventHandler::new();

        // 创建页面，传递首次运行状态
        let home_page = HomePage::new_with_first_run(is_first_run);
        let login_page = LoginPage::new();
        let register_page = RegisterPage::new();
        let profile_page = ProfilePage::new();
        let blog_view_page = BlogViewPage::new();
        let blog_edit_page = BlogEditPage::new();

        // 创建应用实例
        let mut app = Self {
            terminal,
            events,
            current_page: Page::Home,
            previous_page_before_blog_view: Page::Home,
            config,
            api_client,
            home_page,
            login_page,
            register_page,
            profile_page,
            blog_view_page,
            blog_edit_page,
            should_quit: false,
            is_first_run,
        };

        // 加载最新博客
        if !is_first_run {
            app.load_latest_blogs().await;
        }

        Ok(app)
    }

    /// 运行应用主循环
    pub async fn run(&mut self) -> Result<()> {
        while !self.should_quit {
            // 渲染界面
            self.draw()?;

            // 等待事件
            if let Some(event) = self.events.next().await {
                self.handle_event(event).await?;
            }
        }

        Ok(())
    }

    /// 渲染界面
    fn draw(&mut self) -> Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.area();

            match self.current_page {
                Page::Home => {
                    self.home_page.render(frame, area, self.config.is_authenticated());
                }
                Page::Login => {
                    self.login_page.render(frame, area);
                }
                Page::Register => {
                    self.register_page.render(frame, area);
                }
                Page::Profile => {
                    self.profile_page.render(frame, area);
                }
                Page::BlogView => {
                    self.blog_view_page.render(frame, area);
                }
                Page::BlogEdit => {
                    self.blog_edit_page.render(frame, area);
                }
            }
        })?;

        Ok(())
    }

    /// 处理事件
    async fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Quit => {
                self.should_quit = true;
            }
            _ => match self.current_page {
                Page::Home => self.handle_home_event(event).await?,
                Page::Login => self.handle_login_event(event).await?,
                Page::Register => self.handle_register_event(event).await?,
                Page::Profile => self.handle_profile_event(event).await?,
                Page::BlogView => self.handle_blog_view_event(event).await?,
                Page::BlogEdit => self.handle_blog_edit_event(event).await?,
            },
        }
        Ok(())
    }

    /// 处理主页事件
    async fn handle_home_event(&mut self, event: Event) -> Result<()> {
        match self.home_page.handle_event(event) {
            HomeAction::None => {}
            HomeAction::ViewBlog(blog_id) => {
                self.goto_blog_view(&blog_id).await;
            }
            HomeAction::GotoLogin => {
                self.current_page = Page::Login;
            }
            HomeAction::GotoProfile => {
                self.goto_profile().await;
            }
            HomeAction::Quit => {
                self.should_quit = true;
            }
            HomeAction::SetServerUrl(server_url) => {
                // 更新服务器地址
                self.config.server_url = server_url;
                self.config.save()?;
                
                // 重新创建 API 客户端
                self.api_client = ApiClient::new(&self.config);
                
                // 标记不再是首次运行
                self.is_first_run = false;
                
                // 加载博客列表
                self.load_latest_blogs().await;
                
                tracing::info!("服务器地址已更新为: {}", self.config.server_url);
            }
        }
        Ok(())
    }

    /// 跳转到博客预览页面
    async fn goto_blog_view(&mut self, blog_id: &str) {
        self.previous_page_before_blog_view = self.current_page;
        self.current_page = Page::BlogView;
        
        if let Some(ref user) = self.config.current_user {
            self.blog_view_page.set_current_username(user.username.clone());
        }
        
        match self.api_client.get_blog(blog_id).await {
            Ok(blog) => {
                self.blog_view_page.set_blog(blog);
            }
            Err(e) => {
                self.blog_view_page.set_error(e.user_message);
            }
        }
    }

    /// 跳转到个人信息页面
    async fn goto_profile(&mut self) {
        if let Some(ref user) = self.config.current_user {
            self.current_page = Page::Profile;
            self.profile_page.set_user_info(user.clone());
            
                            match self.api_client.get_user_blogs(&user.username).await {
                Ok(blogs) => {
                    self.profile_page.set_blogs(blogs);
                }
                Err(_) => {
                    self.profile_page.set_blogs(vec![]);
                }
            }
        }
    }

    /// 处理登录事件
    async fn handle_login_event(&mut self, event: Event) -> Result<()> {
        match self.login_page.handle_event(event) {
            LoginAction::None => {}
            LoginAction::Login { username, password } => {
                self.login_page.set_loading(true);
                self.draw()?;

                match self.api_client.login(username, password).await {
                    Ok(response) => {
                        // 保存认证信息
                        self.config.auth_token = Some(response.token);
                        self.config.current_user = Some(response.user);
                        self.config.save()?;

                        // 更新API客户端
                        self.api_client.set_auth_token(self.config.auth_token.clone().unwrap());

                        // 返回主页
                        self.current_page = Page::Home;
                        self.load_latest_blogs().await;
                    }
                    Err(e) => {
                        // 使用新的错误系统
                        self.login_page.set_error(&e);
                    }
                }
            }
            LoginAction::Back => {
                self.current_page = Page::Home;
                self.load_latest_blogs().await;
            }
            LoginAction::GotoRegister => {
                self.current_page = Page::Register;
            }
        }
        Ok(())
    }

    /// 处理注册事件
    async fn handle_register_event(&mut self, event: Event) -> Result<()> {
        match self.register_page.handle_event(event) {
            RegisterAction::None => {}
            RegisterAction::Register {
                username,
                nickname,
                password,
                email,
            } => {
                self.register_page.set_loading(true);
                self.draw()?;

                match self.api_client.register(username, nickname, password, email).await {
                    Ok(_user) => {
                        self.register_page.set_success("注册成功！请登录。".to_string());
                    }
                    Err(e) => {
                        self.register_page.set_error(&e);
                    }
                }
            }
            RegisterAction::Back => {
                self.current_page = Page::Login;
            }
        }
        Ok(())
    }

    /// 处理个人信息页面事件
    async fn handle_profile_event(&mut self, event: Event) -> Result<()> {
        match self.profile_page.handle_event(event) {
            ProfileAction::None => {}
            ProfileAction::BackToHome => {
                self.current_page = Page::Home;
                self.load_latest_blogs().await;
            }
            ProfileAction::NewBlog => {
                self.current_page = Page::BlogEdit;
            }
            ProfileAction::ViewBlog(blog_id) => {
                self.goto_blog_view(&blog_id).await;
            }
            ProfileAction::Logout => {
                self.config.clear_auth();
                self.config.save()?;
                self.api_client = ApiClient::new(&self.config);
                self.current_page = Page::Home;
                self.load_latest_blogs().await;
            }
        }
        Ok(())
    }

    /// 处理博客预览页面事件
    async fn handle_blog_view_event(&mut self, event: Event) -> Result<()> {
        match self.blog_view_page.handle_event(event) {
            BlogViewAction::None => {}
            BlogViewAction::Back => {
                self.current_page = self.previous_page_before_blog_view;
                if self.current_page == Page::Home {
                    self.load_latest_blogs().await;
                }
            }
            BlogViewAction::EditBlog(blog_id) => {
                self.goto_blog_edit(Some(blog_id)).await;
            }
            BlogViewAction::UploadBlog(blog_id) => {
                self.blog_view_page.set_uploading(true);
                self.draw()?;
                
                match self.api_client.upload_blog(&blog_id, &self.config).await {
                    Ok(_) => {
                        tracing::info!("博客上传成功: {}", blog_id);
                        self.blog_view_page.set_success("博客上传成功！".to_string());
                    }
                    Err(e) => {
                        tracing::error!("博客上传失败: {}", e.user_message);
                        self.blog_view_page.set_upload_error(e.user_message);
                    }
                }
            }
        }
        Ok(())
    }

    /// 处理博客编辑页面事件
    async fn handle_blog_edit_event(&mut self, event: Event) -> Result<()> {
        match self.blog_edit_page.handle_event(event) {
            BlogEditAction::None => {}
            BlogEditAction::Back => {
                self.current_page = Page::Profile;
            }
            BlogEditAction::SaveSuccess => {
                self.current_page = Page::Profile;
                if let Some(ref user) = self.config.current_user {
                    match self.api_client.get_user_blogs(&user.username).await {
                        Ok(blogs) => {
                            self.profile_page.set_blogs(blogs);
                        }
                        Err(_) => {}
                    }
                }
            }
            BlogEditAction::Upload(blog_id) => {
                match self.api_client.upload_blog(&blog_id, &self.config).await {
                    Ok(_) => {
                        tracing::info!("博客上传成功: {}", blog_id);
                    }
                    Err(e) => {
                        tracing::error!("博客上传失败: {}", e.user_message);
                    }
                }
            }
            BlogEditAction::Save { blog_id, title, content, is_public, is_new } => {
                self.blog_edit_page.set_saving(true);
                self.draw()?;
                
                let result = if is_new {
                    self.api_client.create_blog(&self.config, title, content, is_public).await
                } else {
                    self.api_client.update_blog(blog_id.as_ref().unwrap(), &self.config, title, content, is_public).await
                };
                
                match result {
                    Ok(_blog) => {
                        self.blog_edit_page.set_success("博客保存成功！".to_string());
                    }
                    Err(e) => {
                        self.blog_edit_page.set_saving(false);
                        self.blog_edit_page.set_error(e.user_message);
                    }
                }
            }
        }
        Ok(())
    }

    /// 跳转到博客编辑页面
    async fn goto_blog_edit(&mut self, blog_id: Option<String>) {
        self.current_page = Page::BlogEdit;
        
        if let Some(id) = blog_id {
            match self.api_client.get_blog(&id).await {
                Ok(blog) => {
                    self.blog_edit_page = BlogEditPage::new_for_edit(blog);
                }
                Err(e) => {
                    self.blog_edit_page.set_error(e.user_message);
                }
            }
        } else {
            self.blog_edit_page = BlogEditPage::new();
        }
    }

    /// 加载最新博客
    async fn load_latest_blogs(&mut self) {
        match self.api_client.get_latest_blogs(20).await {
            Ok(blogs) => {
                self.home_page.set_blogs(blogs);
            }
            Err(e) => {
                // 使用新的错误系统
                self.home_page.set_error(e);
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // 清理终端
        let _ = terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}
