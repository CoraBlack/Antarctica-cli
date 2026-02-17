use antarctica_cli::app::App;
use anyhow::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志 - 输出到文件而不是终端，避免污染 TUI
    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("antarctica-cli");
    
    // 确保日志目录存在
    std::fs::create_dir_all(&log_dir)?;
    
    let file_appender = tracing_appender::rolling::daily(log_dir, "antarctica-cli.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(false)
                .with_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into())),
        )
        .init();

    // 创建并运行应用
    let mut app = App::new().await?;
    app.run().await?;

    Ok(())
}
