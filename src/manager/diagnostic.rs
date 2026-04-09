use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use crate::manager::msg::LspMessage;

#[derive(Default)]
pub struct DgnPublisher {
    pub num_servers: usize,
    merged_diagnostics: Option<LspMessage>,
    pub submit_diagnostics: Option<(JoinHandle<()>, mpsc::Sender<LspMessage>)>,
}

impl DgnPublisher {
    /// Remove all diagnostics data
    pub fn reset(&mut self) {
        self.merged_diagnostics = None;
        if let Some((merge_diag_task, _)) = &mut self.submit_diagnostics {
            merge_diag_task.abort();
        }
        self.submit_diagnostics = None;
    }

    /// Spawn a separate task to merge diagnostics and send them off to the client when finished or timed out.
    pub fn spawn_diagnostic_tracker(
        &mut self,
        tx_client: mpsc::Sender<LspMessage>,
        pass_msg: LspMessage,
    ) {
        self.reset();
        let (dg_tx, dg_rx) = mpsc::channel(1);
        let num_outstanding_notifs = self.num_servers - 1;
        let jh = tokio::task::spawn(async move {
            publish_merge_diagnostics(dg_rx, tx_client, pass_msg, num_outstanding_notifs).await;
        });
        self.submit_diagnostics = Some((jh, dg_tx));
    }
}
/// publish diagnostics after a timeout.
pub async fn publish_merge_diagnostics(
    new_diags: mpsc::Receiver<LspMessage>,
    publish: mpsc::Sender<LspMessage>,
    original: LspMessage,
    outstanding_msgs: usize,
) {
    let mut new_diags = new_diags;
    let mut original = original;
    let mut outstanding_msgs = outstanding_msgs;
    if outstanding_msgs == 0 {
        let _ = publish.send(original).await;
        return;
    }

    // todo fix hardcoded timeout
    while outstanding_msgs > 0 {
        match timeout(Duration::from_millis(500), new_diags.recv()).await {
            Ok(add) => {
                if let Some(nxt_diag) = add {
                    original.merge(nxt_diag);
                    outstanding_msgs -= 1;
                    if outstanding_msgs == 0 {
                        break;
                    }
                } else {
                    // todo how to handle closed channel
                    break;
                }
            }
            Err(_) => {
                eprintln!("timed out waiting for more diagnostics");
            }
        }
    }
    let _ = publish.send(original).await;
}
