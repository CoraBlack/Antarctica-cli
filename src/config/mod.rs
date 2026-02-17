use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 服务器基础URL
    pub server_url: String,
    /// 当前用户token（登录后保存）
    pub auth_token: Option<String>,
    /// 当前用户信息
    pub current_user: Option<UserInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub nickname: String,
    pub email: String,
    pub bio: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:8080".to_string(),
            auth_token: None,
            current_user: None,
        }
    }
}

impl Config {
    /// 加载配置
    ///
    /// 如果配置文件不存在，会自动创建默认配置文件
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            // 配置文件存在，正常加载
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            info!("配置文件加载成功: {}", config_path.display());
            Ok(config)
        } else {
            // 配置文件不存在，创建默认配置并保存
            warn!("配置文件不存在，将创建默认配置文件");
            let config = Config::default();
            config.save()?;
            info!("默认配置文件已创建: {}", config_path.display());
            Ok(config)
        }
    }

    /// 检查是否是首次运行
    pub fn is_first_run() -> Result<bool> {
        let config_path = Self::config_path()?;
        Ok(!config_path.exists())
    }

    /// 保存配置
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
                info!("配置目录已创建: {}", parent.display());
            }
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        info!("配置文件已保存: {}", config_path.display());

        Ok(())
    }

    /// 获取配置文件路径
    pub fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取主目录"))?;
        Ok(home
            .join(".config")
            .join("antarctica-cli")
            .join("config.json"))
    }

    /// 获取配置目录路径
    pub fn config_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取主目录"))?;
        Ok(home.join(".config").join("antarctica-cli"))
    }

    /// 检查是否已登录
    pub fn is_authenticated(&self) -> bool {
        self.auth_token.is_some() && self.current_user.is_some()
    }

    /// 清除登录状态
    pub fn clear_auth(&mut self) {
        self.auth_token = None;
        self.current_user = None;
    }

    /// 显示当前配置信息
    pub fn display_info(&self) {
        tracing::info!("当前配置信息:");
        tracing::info!("  服务器地址: {}", self.server_url);
        tracing::info!(
            "  登录状态: {}",
            if self.is_authenticated() {
                "已登录"
            } else {
                "未登录"
            }
        );
        if let Some(ref user) = self.current_user {
            tracing::info!("  当前用户: {} ({})", user.username, user.nickname);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.server_url, "http://localhost:8080");
        assert!(config.auth_token.is_none());
        assert!(config.current_user.is_none());
    }

    #[test]
    fn test_config_is_authenticated() {
        let mut config = Config::default();
        assert!(!config.is_authenticated());

        config.auth_token = Some("token".to_string());
        assert!(!config.is_authenticated());

        config.current_user = Some(UserInfo {
            id: "1".to_string(),
            username: "test".to_string(),
            nickname: "Test".to_string(),
            email: "test@example.com".to_string(),
            bio: None,
        });
        assert!(config.is_authenticated());
    }
}
