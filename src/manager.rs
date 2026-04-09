use std::collections::HashMap;

use serde_json::Value;
use tokio::sync::mpsc::{self, Receiver, Sender};

mod capability;
pub mod client;
mod diagnostic;
mod error;
pub mod lsp;
mod msg;
mod tracker;

use diagnostic::DgnPublisher;
use error::MiddlewareError;
use lsp::ServerMessage;
use msg::LspMessage;
use tracker::Tracker;

/// Forward messages from client to servers based on their capabilities.
/// Also tracks which servers were sent which messages based on their ID, so that the required merges can be handled too.
async fn handle_client_msg(
    maybe_msg: Option<LspMessage>,
    stdins: &HashMap<String, mpsc::Sender<msg::LspMessage>>,
    diagnostic_tracker: &mut DgnPublisher,
    tracker: &mut Tracker,
) -> Result<(), MiddlewareError> {
    match maybe_msg {
        Some(client_msg) => {
            match &client_msg {
                LspMessage::Result(id, _method, _value) => {
                    if let Some(target_server) = tracker.client_responded(*id) {
                        for (ss, stdin) in stdins.iter() {
                            if target_server.as_str() == ss {
                                if stdin.send(client_msg.clone()).await.is_err() {
                                    return Err(MiddlewareError::SendToSvrFail);
                                } else {
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                LspMessage::Request(id, method, _value) => {
                    let capable_servers = tracker.get_capable_servers(method, stdins);
                    for (svr_name, svr_stdin) in capable_servers.into_iter() {
                        if svr_stdin.send(client_msg.clone()).await.is_err() {
                            return Err(MiddlewareError::SendToSvrFail);
                        } else {
                            // store the req id as awaiting response from server
                            tracker.client_issued_request(*id, svr_name.to_string());
                        }
                    }
                }
                LspMessage::Notification(method, _value) => {
                    // notifications go to all
                    if method == "textDocument/didChange" {
                        diagnostic_tracker.reset();
                    }
                    for (_, svr_stdin) in stdins.iter() {
                        if svr_stdin.send(client_msg.clone()).await.is_err() {
                            return Err(MiddlewareError::SendToSvrFail);
                        }
                    }
                }
            }
            Ok(())
        }
        None => Err(MiddlewareError::ClientFail),
    }
}

/// collect capabilities and merge results when requests went to multiple servers.
/// also delay sending diagnostics notifications until a timeout or all servers sent in new ones.
async fn handle_server_msg(
    maybe_msg: Option<ServerMessage>,
    tx_client: &Sender<LspMessage>,
    diagnostic_tracker: &mut DgnPublisher,
    sbrd: &mut Tracker,
) -> Result<(), MiddlewareError> {
    match maybe_msg {
        Some(server_msg) => match server_msg.content.clone() {
            LspMessage::Result(id, _, value) => {
                if let Some(Value::Object(res)) = value.get("result")
                    && id == 1
                {
                    if let Some(Value::Object(caps)) = res.get("capabilities") {
                        for cap in caps.iter() {
                            sbrd.register_capability(cap.0, server_msg.server_name.to_string());
                        }
                    }
                }
                // remove server from response tracker
                sbrd.server_responded(id, &server_msg.server_name, server_msg.content);

                // Check if everything has been responded and sent the message
                if sbrd.all_servers_replied(id) {
                    match sbrd.get_merged_response(id) {
                        Some(response) => {
                            if tx_client.send(response).await.is_err() {
                                return Err(MiddlewareError::SendToClientFail);
                            }
                        }
                        None => return Err(MiddlewareError::SendToClientFail),
                    }
                }
                Ok(())
            }
            LspMessage::Request(id, _, _) => {
                if tx_client.send(server_msg.content).await.is_err() {
                    return Err(MiddlewareError::SendToClientFail);
                } else {
                    sbrd.server_issued_request(server_msg.server_name.to_string(), id);
                }
                Ok(())
            }
            LspMessage::Notification(method, _) => {
                match method.as_str() {
                    "textDocument/publishDiagnostics" => {
                        if let Some((jh, push_diagnostics)) =
                            &mut diagnostic_tracker.submit_diagnostics
                        {
                            if !jh.is_finished() {
                                let _ = push_diagnostics.send(server_msg.content.clone()).await;
                            } else {
                                diagnostic_tracker.spawn_diagnostic_tracker(
                                    tx_client.clone(),
                                    server_msg.content.clone(),
                                );
                            }
                        } else {
                            diagnostic_tracker.spawn_diagnostic_tracker(
                                tx_client.clone(),
                                server_msg.content.clone(),
                            );
                        }
                    }
                    _ => {
                        if tx_client.send(server_msg.content).await.is_err() {
                            return Err(MiddlewareError::SendToClientFail);
                        }
                    }
                }
                Ok(())
            }
        },
        None => Err(MiddlewareError::LspCommFail),
    }
}

pub async fn run_middleware(
    stdins: HashMap<String, mpsc::Sender<msg::LspMessage>>,
    client_channel: (Sender<LspMessage>, Receiver<LspMessage>),
    from_servers: Receiver<ServerMessage>,
    default_lsp: String,
) -> Result<(), MiddlewareError> {
    let mut from_servers = from_servers;
    let (client_tx, mut client_rx) = client_channel;
    let mut dgns = DgnPublisher::default();
    let mut sbrd = Tracker::default();
    sbrd.default_server = default_lsp;
    dgns.num_servers = stdins.len();
    loop {
        tokio::select! {
            client_msg = client_rx.recv() => {
                handle_client_msg(client_msg, &stdins, &mut dgns, &mut sbrd).await?;
            }
            server_msg = from_servers.recv() => {
                handle_server_msg(server_msg, &client_tx, &mut dgns, &mut sbrd).await?;
            }
        };
    }
}
