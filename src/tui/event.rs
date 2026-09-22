use crossterm::event::{Event as CrosstermEvent, KeyEvent, KeyEventKind};
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

pub enum AppEvent {
    Key(KeyEvent),
    Paste(String),
    Tick,
    SearchFinished {
        request_id: u64,
        query: String,
        result: std::result::Result<crate::api::dict::QueryOutput, String>,
    },
    EventError(String),
}

const EVENT_QUEUE_CAPACITY: usize = 256;

pub struct EventHandler {
    rx: mpsc::Receiver<AppEvent>,
    tx: mpsc::Sender<AppEvent>,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel(EVENT_QUEUE_CAPACITY);
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        let reader_tx = tx.clone();

        let task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut interval = tokio::time::interval(tick_rate);

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => break,
                    _ = interval.tick() => {
                        // Tick is only a redraw opportunity. Dropping it when the
                        // queue is busy is preferable to building stale ticks.
                        let _ = reader_tx.try_send(AppEvent::Tick);
                    }
                    event = reader.next() => {
                        match event {
                            Some(Ok(CrosstermEvent::Key(key))) if key.kind == KeyEventKind::Press => {
                                if reader_tx.send(AppEvent::Key(key)).await.is_err() {
                                    break;
                                }
                            }
                            Some(Ok(CrosstermEvent::Paste(pasted))) => {
                                if reader_tx.send(AppEvent::Paste(pasted)).await.is_err() {
                                    break;
                                }
                            }
                            Some(Ok(_)) => {}
                            Some(Err(error)) => {
                                let _ = reader_tx.send(AppEvent::EventError(error.to_string())).await;
                                break;
                            }
                            None => break,
                        }
                    }
                }
            }
        });

        Self {
            rx,
            tx,
            shutdown: Some(shutdown_tx),
            task: Some(task),
        }
    }

    pub fn sender(&self) -> mpsc::Sender<AppEvent> {
        self.tx.clone()
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        let _ = self.shutdown.take();
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}
