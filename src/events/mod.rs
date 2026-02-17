use crossterm::event::{Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// 应用程序事件
#[derive(Debug, Clone)]
pub enum Event {
    /// 按键事件
    Key(KeyEvent),
    /// 鼠标事件
    Mouse(MouseEvent),
    /// 终端大小改变
    Resize(u16, u16),
    /// 定时器事件
    Tick,
    /// 退出应用
    Quit,
}

/// 事件处理器
pub struct EventHandler {
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    /// 创建新的事件处理器
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        // 启动事件监听任务
        tokio::spawn(async move {
            let mut last_tick = Instant::now();
            let tick_interval = Duration::from_secs(1);
            
            loop {
                // 检查是否有crossterm事件
                if crossterm::event::poll(Duration::from_millis(50)).unwrap_or(false) {
                    match crossterm::event::read() {
                        Ok(CrosstermEvent::Key(key)) => {
                            // 只处理按键按下事件，忽略释放和重复事件
                            if key.kind != KeyEventKind::Press {
                                continue;
                            }
                            
                            // 检查是否是退出键（Ctrl+C 或 Ctrl+Q）
                            if key.code == crossterm::event::KeyCode::Char('c')
                                && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                            {
                                let _ = sender.send(Event::Quit);
                                break;
                            }
                            if key.code == crossterm::event::KeyCode::Char('q')
                                && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                            {
                                let _ = sender.send(Event::Quit);
                                break;
                            }
                            let _ = sender.send(Event::Key(key));
                        }
                        Ok(CrosstermEvent::Mouse(mouse)) => {
                            let _ = sender.send(Event::Mouse(mouse));
                        }
                        Ok(CrosstermEvent::Resize(width, height)) => {
                            let _ = sender.send(Event::Resize(width, height));
                        }
                        _ => {}
                    }
                }

                // 每秒发送一次定时器事件
                if last_tick.elapsed() >= tick_interval {
                    let _ = sender.send(Event::Tick);
                    last_tick = Instant::now();
                }
            }
        });

        Self { receiver }
    }

    /// 接收下一个事件
    pub async fn next(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }
}
