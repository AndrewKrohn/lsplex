use tokio::{
    io::{self, BufReader},
    sync::mpsc,
};

use crate::manager::{error::ClientError, msg::LspMessage};

async fn forward_msg_to_middle(
    msg: Option<LspMessage>,
    to_middle: &mpsc::Sender<LspMessage>,
) -> Result<(), ClientError> {
    match msg {
        Some(fwd_msg) => {
            if to_middle.send(fwd_msg).await.is_err() {
                return Err(ClientError::CommFail);
            }
            Ok(())
        }
        None => Err(ClientError::FailedReadMsg),
    }
}

/// Handle msg from server through middleware
fn handle_message_from_server(lsp_msg: Option<LspMessage>) -> Result<(), ClientError> {
    if let Some(svr_msg) = lsp_msg {
        println!("{}", svr_msg.format_with_header());
    } else {
        return Err(ClientError::FailedRecvMsg);
    }
    return Ok(());
}

/// Client handler reads from stdin and forwards messages to middleware. It also will write to stdout whatever messages it receives from middleware.
pub async fn client_handler(
    to_middle: mpsc::Sender<LspMessage>,
    mut from_middle: mpsc::Receiver<LspMessage>,
) -> Result<(), ClientError> {
    let mut rdr = BufReader::new(io::stdin());

    loop {
        tokio::select! {
            msg = LspMessage::from_reader(&mut rdr) => {
                forward_msg_to_middle(msg.ok(), &to_middle).await?;
            }
            msg = from_middle.recv() => {
                handle_message_from_server(msg)?;
            }
        };
    }
}
