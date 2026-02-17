use thiserror::Error;
use tracing::{error, warn};

/// 错误码定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // 系统级错误 (1000-1999)
    IoError = 1000,
    ConfigError = 1001,

    // 网络错误 (2000-2999)
    NetworkError = 2000,
    ConnectionTimeout = 2001,
    ConnectionRefused = 2002,
    DnsError = 2003,

    // HTTP/API错误 (3000-3999)
    HttpError = 3000,
    BadRequest = 3001,
    Unauthorized = 3002,
    Forbidden = 3003,
    NotFound = 3004,
    ServerError = 3005,
    ServiceUnavailable = 3006,

    // 数据错误 (4000-4999)
    JsonError = 4000,
    ParseError = 4001,
    ValidationError = 4002,

    // 认证错误 (5000-5999)
    AuthError = 5000,
    InvalidCredentials = 5001,
    TokenExpired = 5002,

    // 输入错误 (6000-6999)
    InputError = 6000,
    InvalidInput = 6001,

    // 未知错误 (9000)
    UnknownError = 9000,
}

impl ErrorCode {
    /// 获取错误码数字
    pub fn code(&self) -> u32 {
        *self as u32
    }

    /// 获取用户友好的错误消息
    pub fn user_message(&self) -> &'static str {
        match self {
            ErrorCode::IoError => "文件操作失败，请检查磁盘空间",
            ErrorCode::ConfigError => "配置文件读取失败",

            ErrorCode::NetworkError => "网络连接失败，请检查网络设置",
            ErrorCode::ConnectionTimeout => "连接超时，服务器响应过慢",
            ErrorCode::ConnectionRefused => "无法连接到服务器",
            ErrorCode::DnsError => "域名解析失败",

            ErrorCode::HttpError => "服务器通信异常",
            ErrorCode::BadRequest => "请求参数错误",
            ErrorCode::Unauthorized => "请先登录",
            ErrorCode::Forbidden => "没有权限执行此操作",
            ErrorCode::NotFound => "请求的资源不存在",
            ErrorCode::ServerError => "服务器内部错误",
            ErrorCode::ServiceUnavailable => "服务暂时不可用",

            ErrorCode::JsonError => "数据解析失败",
            ErrorCode::ParseError => "数据格式错误",
            ErrorCode::ValidationError => "数据验证失败",

            ErrorCode::AuthError => "认证失败",
            ErrorCode::InvalidCredentials => "用户名或密码错误",
            ErrorCode::TokenExpired => "登录已过期，请重新登录",

            ErrorCode::InputError => "输入错误",
            ErrorCode::InvalidInput => "输入内容格式不正确",

            ErrorCode::UnknownError => "发生未知错误",
        }
    }
}

/// 应用程序错误
#[derive(Error, Debug, Clone)]
pub struct AppError {
    /// 错误码
    pub code: ErrorCode,
    /// 用户友好的错误消息
    pub user_message: String,
    /// 详细错误信息（用于日志记录）
    pub detail: Option<String>,
}

impl AppError {
    /// 创建新的错误
    pub fn new(code: ErrorCode, user_message: impl Into<String>) -> Self {
        let msg = user_message.into();
        Self {
            code,
            user_message: msg.clone(),
            detail: None,
        }
    }

    /// 添加详细错误信息
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// 获取错误显示信息（包含错误码）
    pub fn display_with_code(&self) -> String {
        format!("[错误码: {}] {}", self.code.code(), self.user_message)
    }

    /// 记录错误日志
    pub fn log(&self) {
        if let Some(ref detail) = self.detail {
            error!(
                error_code = %self.code.code(),
                user_message = %self.user_message,
                detail = %detail,
                "应用错误发生"
            );
        } else {
            error!(
                error_code = %self.code.code(),
                user_message = %self.user_message,
                "应用错误发生"
            );
        }
    }

    /// 记录警告日志
    pub fn log_warn(&self) {
        if let Some(ref detail) = self.detail {
            warn!(
                error_code = %self.code.code(),
                user_message = %self.user_message,
                detail = %detail,
                "应用警告"
            );
        } else {
            warn!(
                error_code = %self.code.code(),
                user_message = %self.user_message,
                "应用警告"
            );
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_with_code())
    }
}

/// 从标准IO错误转换
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        let code = match err.kind() {
            std::io::ErrorKind::NotFound => ErrorCode::NotFound,
            std::io::ErrorKind::PermissionDenied => ErrorCode::Forbidden,
            std::io::ErrorKind::ConnectionRefused => ErrorCode::ConnectionRefused,
            std::io::ErrorKind::ConnectionReset => ErrorCode::NetworkError,
            std::io::ErrorKind::ConnectionAborted => ErrorCode::NetworkError,
            std::io::ErrorKind::NotConnected => ErrorCode::NetworkError,
            std::io::ErrorKind::AddrInUse => ErrorCode::NetworkError,
            std::io::ErrorKind::AddrNotAvailable => ErrorCode::NetworkError,
            std::io::ErrorKind::BrokenPipe => ErrorCode::NetworkError,
            std::io::ErrorKind::AlreadyExists => ErrorCode::ValidationError,
            std::io::ErrorKind::WouldBlock => ErrorCode::NetworkError,
            std::io::ErrorKind::InvalidInput => ErrorCode::InvalidInput,
            std::io::ErrorKind::InvalidData => ErrorCode::ParseError,
            std::io::ErrorKind::TimedOut => ErrorCode::ConnectionTimeout,
            std::io::ErrorKind::WriteZero => ErrorCode::IoError,
            std::io::ErrorKind::Interrupted => ErrorCode::IoError,
            std::io::ErrorKind::Other => ErrorCode::UnknownError,
            std::io::ErrorKind::UnexpectedEof => ErrorCode::ParseError,
            std::io::ErrorKind::OutOfMemory => ErrorCode::IoError,
            _ => ErrorCode::IoError,
        };

        AppError {
            code,
            user_message: code.user_message().to_string(),
            detail: Some(format!("IO错误: {}", err)),
        }
    }
}

/// 从HTTP错误转换
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        let code = if err.is_timeout() {
            ErrorCode::ConnectionTimeout
        } else if err.is_connect() {
            ErrorCode::ConnectionRefused
        } else if err.is_body() {
            ErrorCode::ParseError
        } else if err.is_decode() {
            ErrorCode::JsonError
        } else if err.is_request() {
            ErrorCode::HttpError
        } else if err.is_status() {
            match err.status() {
                Some(status) => match status.as_u16() {
                    400 => ErrorCode::BadRequest,
                    401 => ErrorCode::Unauthorized,
                    403 => ErrorCode::Forbidden,
                    404 => ErrorCode::NotFound,
                    422 => ErrorCode::ValidationError,
                    500 => ErrorCode::ServerError,
                    502 => ErrorCode::ServiceUnavailable,
                    503 => ErrorCode::ServiceUnavailable,
                    504 => ErrorCode::ConnectionTimeout,
                    _ => ErrorCode::HttpError,
                },
                None => ErrorCode::HttpError,
            }
        } else {
            ErrorCode::NetworkError
        };

        // 不暴露URL给普通用户，只在详细日志中记录
        let detail = format!("HTTP请求失败: {}", err);

        AppError {
            code,
            user_message: code.user_message().to_string(),
            detail: Some(detail),
        }
    }
}

/// 从JSON错误转换
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError {
            code: ErrorCode::JsonError,
            user_message: ErrorCode::JsonError.user_message().to_string(),
            detail: Some(format!("JSON解析失败: {}", err)),
        }
    }
}

/// 便捷宏创建错误
#[macro_export]
macro_rules! app_error {
    ($code:expr, $msg:expr) => {
        $crate::utils::AppError::new($code, $msg)
    };
    ($code:expr, $msg:expr, $detail:expr) => {
        $crate::utils::AppError::new($code, $msg).with_detail($detail)
    };
}

pub use app_error;

pub type AppResult<T> = Result<T, AppError>;

/// 错误显示组件，用于在UI中显示错误
pub struct ErrorDisplay;

impl ErrorDisplay {
    /// 获取短格式的错误消息（用于状态栏等）
    pub fn short_message(error: &AppError) -> String {
        format!("错误 [{}]: {}", error.code.code(), error.user_message)
    }

    /// 获取完整错误消息（用于错误弹窗）
    pub fn full_message(error: &AppError) -> String {
        let mut msg = format!("错误码: {}\n", error.code.code());
        msg.push_str(&format!("错误信息: {}\n", error.user_message));

        if let Some(ref detail) = error.detail {
            msg.push_str(&format!("\n详细信息（供开发者参考）:\n{}", detail));
        }

        msg.push_str("\n\n如需帮助，请提供错误码给技术支持。");
        msg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        let err = AppError::new(ErrorCode::NetworkError, "网络错误");
        assert_eq!(err.display_with_code(), "[错误码: 2000] 网络错误");
    }

    #[test]
    fn test_error_with_detail() {
        let err = AppError::new(ErrorCode::HttpError, "服务器错误")
            .with_detail("连接超时: http://example.com/api");

        assert_eq!(err.code.code(), 3000);
        assert_eq!(err.user_message, "服务器错误");
        assert!(err.detail.is_some());
    }
}
