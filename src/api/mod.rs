use crate::{
    config::{Config, UserInfo},
    utils::{AppResult, ErrorCode, AppError},
};
use serde::{Deserialize, Serialize};

/// API客户端
#[derive(Clone)]
pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    auth_token: Option<String>,
}

/// 登录请求
#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 注册请求
#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub username: String,
    pub nickname: String,
    pub password: String,
    pub email: String,
}

/// API响应包装
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

/// 分页信息
#[derive(Debug, Deserialize)]
pub struct PaginationInfo {
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// 分页响应
#[derive(Debug, Deserialize)]
pub struct PaginatedResponse<T> {
    pub code: u16,
    pub message: String,
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
    #[serde(default)]
    pub timestamp: Option<String>,
}

/// 登录响应
#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

/// 可见性枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Public
    }
}

/// 博客状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BlogStatus {
    Draft,
    Published,
    Deleted,
}

impl Default for BlogStatus {
    fn default() -> Self {
        BlogStatus::Published
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthorInfo {
    pub id: String,
    pub username: String,
}

/// 博客信息（列表和详情共用）
#[derive(Debug, Clone, Deserialize)]
pub struct Blog {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub html_content: String,
    pub author: AuthorInfo,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub status: BlogStatus,
    pub visibility: Visibility,
}

/// 博客详情（别名，保持兼容）
pub type BlogDetail = Blog;

/// 博客列表项（别名，保持兼容）
pub type BlogListItem = Blog;

impl ApiClient {
    /// 创建新的API客户端
    pub fn new(config: &Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: config.server_url.clone(),
            auth_token: config.auth_token.clone(),
        }
    }
    
    /// 构建带有用户信息的请求
    fn build_request_with_user(&self, request_builder: reqwest::RequestBuilder, config: &Config) -> reqwest::RequestBuilder {
        let mut request = request_builder;
        
        // 添加用户信息头
        if let Some(ref user) = config.current_user {
            request = request
                .header("X-User-Id", &user.id)
                .header("X-Username", &user.username);
        }
        
        // 添加认证token
        if let Some(ref token) = self.auth_token {
            request = request.bearer_auth(token);
        }
        
        request
    }

    /// 更新认证token
    pub fn set_auth_token(&mut self, token: String) {
        self.auth_token = Some(token);
    }

    /// 登录
    pub async fn login(&self, username: String, password: String) -> AppResult<LoginResponse> {
        let url = format!("{}/api/v1/login", self.base_url);
        let req = LoginRequest { username, password };

        let response = self.client.post(&url).json(&req).send().await
            .map_err(|e| {
                let err: AppError = e.into();
                err.log();
                err
            })?;

        if response.status().is_success() {
            let api_response: ApiResponse<LoginResponse> = response.json().await
                .map_err(|e| {
                    let err: AppError = e.into();
                    err.log();
                    err
                })?;
            
            if let Some(data) = api_response.data {
                Ok(data)
            } else {
                let err = AppError::new(ErrorCode::AuthError, "登录失败");
                err.log();
                Err(err)
            }
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            
            let err_code = match status.as_u16() {
                401 => ErrorCode::InvalidCredentials,
                403 => ErrorCode::Forbidden,
                404 => ErrorCode::NotFound,
                422 => ErrorCode::ValidationError,
                500 => ErrorCode::ServerError,
                503 => ErrorCode::ServiceUnavailable,
                _ => ErrorCode::AuthError,
            };
            
            let err = AppError::new(err_code, err_code.user_message())
                .with_detail(format!("登录请求失败，状态码: {}", status));
            err.log();
            Err(err)
        }
    }

    /// 注册
    pub async fn register(
        &self,
        username: String,
        nickname: String,
        password: String,
        email: String,
    ) -> AppResult<UserInfo> {
        let url = format!("{}/api/v1/register", self.base_url);
        let req = RegisterRequest {
            username,
            nickname,
            password,
            email,
        };

        let response = self.client.post(&url).json(&req).send().await
            .map_err(|e| {
                let err: AppError = e.into();
                err.log();
                err
            })?;

        if response.status().is_success() {
            let api_response: ApiResponse<UserInfo> = response.json().await
                .map_err(|e| {
                    let err: AppError = e.into();
                    err.log();
                    err
                })?;
            
            if let Some(data) = api_response.data {
                Ok(data)
            } else {
                let err = AppError::new(ErrorCode::ValidationError, "注册失败");
                err.log();
                Err(err)
            }
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            
            let err_code = match status.as_u16() {
                400 => ErrorCode::BadRequest,
                409 => ErrorCode::ValidationError,
                422 => ErrorCode::ValidationError,
                _ => ErrorCode::HttpError,
            };
            
            let err = AppError::new(err_code, err_code.user_message())
                .with_detail(format!("注册请求失败，状态码: {}", status));
            err.log();
            Err(err)
        }
    }

    /// 获取最新博客列表
    pub async fn get_latest_blogs(&self, limit: u64) -> AppResult<Vec<BlogListItem>> {
        let url = format!("{}/api/v1/blogs/latest?per_page={}", self.base_url, limit);

        let response = self.client.get(&url).send().await
            .map_err(|e| {
                let err: AppError = e.into();
                err.log();
                err
            })?;

        if response.status().is_success() {
            let api_response: ApiResponse<Vec<BlogListItem>> = response.json().await
                .map_err(|e| {
                    let err: AppError = e.into();
                    err.log();
                    err
                })?;
            
            Ok(api_response.data.unwrap_or_default())
        } else {
            let status = response.status();
            let err = AppError::new(ErrorCode::HttpError, ErrorCode::HttpError.user_message())
                .with_detail(format!("获取博客列表失败，状态码: {}", status));
            err.log();
            Err(err)
        }
    }

    /// 获取博客详情
    pub async fn get_blog(&self, blog_id: &str) -> AppResult<BlogDetail> {
        let url = format!("{}/api/v1/blogs/{}", self.base_url, blog_id);

        let response = self.client.get(&url).send().await
            .map_err(|e| {
                let err: AppError = e.into();
                err.log();
                err
            })?;

        if response.status().is_success() {
            let api_response: ApiResponse<BlogDetail> = response.json().await
                .map_err(|e| {
                    let err: AppError = e.into();
                    err.log();
                    err
                })?;
            
            if let Some(data) = api_response.data {
                Ok(data)
            } else {
                let err = AppError::new(ErrorCode::NotFound, "博客不存在");
                err.log();
                Err(err)
            }
        } else {
            let status = response.status();
            let err_code = match status.as_u16() {
                404 => ErrorCode::NotFound,
                _ => ErrorCode::HttpError,
            };
            
            let err = AppError::new(err_code, err_code.user_message())
                .with_detail(format!("获取博客详情失败，状态码: {}", status));
            err.log();
            Err(err)
        }
    }

    /// 获取用户博客列表
    pub async fn get_user_blogs(&self, username: &str) -> AppResult<Vec<BlogListItem>> {
        let url = format!("{}/api/v1/users/{}/blogs", self.base_url, username);

        let request_builder = if let Some(ref token) = self.auth_token {
            self.client.get(&url).bearer_auth(token)
        } else {
            self.client.get(&url)
        };

        let response = request_builder.send().await
            .map_err(|e| {
                let err: AppError = e.into();
                err.log();
                err
            })?;

        if response.status().is_success() {
            let api_response: PaginatedResponse<BlogListItem> = response.json().await
                .map_err(|e| {
                    let err: AppError = e.into();
                    err.log();
                    err
                })?;
            
            Ok(api_response.data)
        } else {
            let status = response.status();
            let err = AppError::new(ErrorCode::HttpError, ErrorCode::HttpError.user_message())
                .with_detail(format!("获取用户博客失败，状态码: {}", status));
            err.log();
            Err(err)
        }
    }

    /// 创建博客
    pub async fn create_blog(&self, config: &crate::config::Config, title: String, content: String, is_public: bool) -> AppResult<BlogDetail> {
        let url = format!("{}/api/v1/blogs", self.base_url);
        
        #[derive(Serialize)]
        struct CreateBlogRequest {
            title: String,
            content: String,
            status: BlogStatus,
            visibility: Visibility,
            author_username: String,
        }
        
        let req = CreateBlogRequest {
            title,
            content,
            status: BlogStatus::Published,
            visibility: if is_public { Visibility::Public } else { Visibility::Private },
            author_username: config.current_user.as_ref().map(|u| u.username.clone()).unwrap_or_default(),
        };

        let response = self.client.post(&url)
            .bearer_auth(self.auth_token.as_ref().unwrap())
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                let err: AppError = e.into();
                err.log();
                err
            })?;

        if response.status().is_success() {
            let api_response: ApiResponse<BlogDetail> = response.json().await
                .map_err(|e| {
                    let err: AppError = e.into();
                    err.log();
                    err
                })?;
            
            if let Some(data) = api_response.data {
                Ok(data)
            } else {
                let err = AppError::new(ErrorCode::ValidationError, "创建博客失败");
                err.log();
                Err(err)
            }
        } else {
            let status = response.status();
            let err = AppError::new(ErrorCode::HttpError, ErrorCode::HttpError.user_message())
                .with_detail(format!("创建博客失败，状态码: {}", status));
            err.log();
            Err(err)
        }
    }

    /// 更新博客
    pub async fn update_blog(&self, blog_id: &str, config: &crate::config::Config, title: String, content: String, is_public: bool) -> AppResult<BlogDetail> {
        let url = format!("{}/api/v1/blogs/{}", self.base_url, blog_id);
        
        #[derive(Serialize)]
        struct UpdateBlogRequest {
            title: String,
            content: String,
            visibility: Visibility,
            author_username: String,
        }
        
        let req = UpdateBlogRequest {
            title,
            content,
            visibility: if is_public { Visibility::Public } else { Visibility::Private },
            author_username: config.current_user.as_ref().map(|u| u.username.clone()).unwrap_or_default(),
        };

        let response = self.client.put(&url)
            .bearer_auth(self.auth_token.as_ref().unwrap())
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                let err: AppError = e.into();
                err.log();
                err
            })?;

        if response.status().is_success() {
            let api_response: ApiResponse<BlogDetail> = response.json().await
                .map_err(|e| {
                    let err: AppError = e.into();
                    err.log();
                    err
                })?;
            
            if let Some(data) = api_response.data {
                Ok(data)
            } else {
                let err = AppError::new(ErrorCode::ValidationError, "更新博客失败");
                err.log();
                Err(err)
            }
        } else {
            let status = response.status();
            let err = AppError::new(ErrorCode::HttpError, ErrorCode::HttpError.user_message())
                .with_detail(format!("更新博客失败，状态码: {}", status));
            err.log();
            Err(err)
        }
    }

    /// 上传博客（发布到公开）
    pub async fn upload_blog(&self, blog_id: &str, config: &crate::config::Config) -> AppResult<BlogDetail> {
        let url = format!("{}/api/v1/blogs/{}/publish", self.base_url, blog_id);
        
        #[derive(Serialize)]
        struct PublishBlogRequest {
            author_username: String,
        }
        
        let req = PublishBlogRequest {
            author_username: config.current_user.as_ref().map(|u| u.username.clone()).unwrap_or_default(),
        };

        let response = self.client.post(&url)
            .bearer_auth(self.auth_token.as_ref().unwrap())
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                let err: AppError = e.into();
                err.log();
                err
            })?;

        if response.status().is_success() {
            let api_response: ApiResponse<BlogDetail> = response.json().await
                .map_err(|e| {
                    let err: AppError = e.into();
                    err.log();
                    err
                })?;
            
            if let Some(data) = api_response.data {
                Ok(data)
            } else {
                let err = AppError::new(ErrorCode::ValidationError, "上传博客失败");
                err.log();
                Err(err)
            }
        } else {
            let status = response.status();
            let err = AppError::new(ErrorCode::HttpError, ErrorCode::HttpError.user_message())
                .with_detail(format!("上传博客失败，状态码: {}", status));
            err.log();
            Err(err)
        }
    }
}
