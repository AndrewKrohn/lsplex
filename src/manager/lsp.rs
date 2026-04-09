use std::process::Stdio;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    process::Command,
    sync::mpsc,
};

use crate::manager::{error::LspError, msg::LspMessage};

pub struct ServerMessage {
    pub server_name: String,
    pub content: LspMessage,
}
pub struct Lsp {
    name: String,
    proc: Command,
    from_midware: mpsc::Receiver<LspMessage>,
}

impl Lsp {
    pub fn create(prog: String, args: Vec<String>) -> (Self, mpsc::Sender<LspMessage>) {
        let mut cmd = Command::new(prog.to_string());
        cmd.args(args);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        let (midware_com, from_midware) = mpsc::channel(1);
        (
            Self {
                name: prog,
                proc: cmd,
                from_midware,
            },
            midware_com,
        )
    }

    /// Run the lsp server
    pub async fn run(mut self, to_midware: mpsc::Sender<ServerMessage>) -> Result<(), LspError> {
        let mut child = self.proc.spawn().map_err(|_| LspError::StdioError)?;
        let stdout = child.stdout.take().unwrap();
        let mut stdin = child.stdin.take().unwrap();
        let mut buf_reader = BufReader::new(stdout);
        loop {
            tokio::select! {
                msg = LspMessage::from_reader(&mut buf_reader) => {
                    let fwd_msg = msg.map_err(|_| LspError::ReadMsgError)?;
                    if to_midware
                        .send(ServerMessage {
                            server_name: self.name.to_string(),
                            content: fwd_msg,
                        })
                    .await.is_err() {
                        return Err(LspError::FwdMsgError);
                    }
                        }
                msg = self.from_midware.recv() => {
                    if let Some(server_msg) =  msg {
                        if stdin.write(&server_msg.format_with_header().into_bytes()).await.is_err() {
                            return Err(LspError::StdioError);
                        }
                    } else {
                        return Err(LspError::RecvMsgError);
                    }
                }
            }
        }
    }
}
