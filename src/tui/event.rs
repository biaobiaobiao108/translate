use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use futures::StreamExt;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub enum AppEvent {
    Key(KeyEvent),
    Paste(String),
    Tick,
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut interval = tokio::time::interval(tick_rate);

            // 用于把终端高速拆分发送的单字/回车聚合为单次 Paste 事件
            let mut rapid_chars = Vec::new();
            let mut last_key_time = Instant::now();

            loop {
                let tick_delay = interval.tick();
                let crossterm_event = reader.next();

                tokio::select! {
                    _ = tick_delay => {
                        // 如果之前积攒了快速连击/粘贴流，且超过 30ms 没有新按键，统一作为 Paste 事件发送
                        if !rapid_chars.is_empty() && last_key_time.elapsed() > Duration::from_millis(30) {
                            let pasted_str: String = rapid_chars.drain(..).collect();
                            let _ = tx.send(AppEvent::Paste(pasted_str));
                        }
                        let _ = tx.send(AppEvent::Tick);
                    }
                    Some(Ok(evt)) = crossterm_event => {
                        match evt {
                            CrosstermEvent::Key(key) => {
                                if key.kind == crossterm::event::KeyEventKind::Press {
                                    let now = Instant::now();
                                    let interval_since_last = now.duration_since(last_key_time);
                                    last_key_time = now;

                                    // 检查是否为可打印字符或回车换行
                                    let ch_opt = match key.code {
                                        KeyCode::Char(c) if !key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => Some(c),
                                        KeyCode::Enter => Some('\n'),
                                        _ => None,
                                    };

                                    // 如果按键到达极快 (<12ms) 或者缓冲区已有积攒字符，归为粘贴流缓冲
                                    if let Some(ch) = ch_opt {
                                        if interval_since_last < Duration::from_millis(15) || !rapid_chars.is_empty() {
                                            rapid_chars.push(ch);
                                            continue;
                                        }
                                    }

                                    // 遇到正常按键，先 flush 之前积攒的字符
                                    if !rapid_chars.is_empty() {
                                        let pasted_str: String = rapid_chars.drain(..).collect();
                                        let _ = tx.send(AppEvent::Paste(pasted_str));
                                    }

                                    let _ = tx.send(AppEvent::Key(key));
                                }
                            }
                            CrosstermEvent::Paste(pasted) => {
                                if !rapid_chars.is_empty() {
                                    let prev_str: String = rapid_chars.drain(..).collect();
                                    let _ = tx.send(AppEvent::Paste(prev_str));
                                }
                                let _ = tx.send(AppEvent::Paste(pasted));
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        Self { rx }
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}
