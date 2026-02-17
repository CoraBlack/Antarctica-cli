pub mod blog_edit;
pub mod blog_view;
pub mod home;
pub mod login;
pub mod profile;
pub mod register;

use crate::events::Event;
use ratatui::Frame;

/// 应用页面枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Page {
    /// 主页（最新文章列表）
    Home,
    /// 登录页面
    Login,
    /// 注册页面
    Register,
    /// 博客预览页面
    BlogView,
    /// 博客编辑页面
    BlogEdit,
    /// 个人资料页面
    Profile,
}

/// 页面通用接口
pub trait PageTrait {
    /// 处理事件
    fn handle_event(&mut self, event: Event);
    /// 渲染页面
    fn render(&self, frame: &mut Frame);
    /// 获取页面帮助信息
    fn get_help_text(&self) -> Vec<(String, String)>;
}
