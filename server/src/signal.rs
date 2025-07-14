use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::task::JoinHandle;

pub struct ExitSignal {
    broadcast_rx: Receiver<()>,
}

impl ExitSignal {
    pub async fn wait(&mut self) {
        if let Err(e) = self.broadcast_rx.recv().await {
            error!("Failed to receive exit signal: {}", e);
        }
    }

    pub async fn wait_owned(mut self) {
        self.wait().await;
    }
}

#[derive(Clone)]
pub struct ExitSignaler {
    broadcast_tx: Arc<Sender<()>>,
}

impl ExitSignaler {
    pub fn new() -> Self {
        Self {
            broadcast_tx: Arc::new(broadcast::channel(1).0),
        }
    }

    pub fn wait_for_shutdown(self) -> JoinHandle<Result<(), anyhow::Error>> {
        tokio::spawn(async move {
            let mut ret = Ok(());
            if let Err(e) = tokio::signal::ctrl_c().await {
                error!("Failed to wait for ctrl-c: {}", e);
                ret = Err(e.into());
            }

            self.exit();
            ret
        })
    }

    pub fn new_exit_signal(&self) -> ExitSignal {
        ExitSignal {
            broadcast_rx: self.broadcast_tx.subscribe(),
        }
    }

    pub fn exit(&self) {
        if let Err(e) = self.broadcast_tx.send(()) {
            error!("Failed to send exit signal: {}", e);
        }
    }
}
