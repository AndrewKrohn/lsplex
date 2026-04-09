use crate::manager::capability;
use crate::manager::msg::LspMessage;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::Sender;

/// Keep track of messages and capabilities across multiple LSPs
#[derive(Default)]
pub struct Tracker {
    /// Log servers capable of a method
    capabilities: HashMap<String, Vec<String>>,
    /// Track which servers need to reply to a request
    outstanding_server_responses: HashMap<i32, HashSet<String>>,
    /// Where to send client responses to for server requests
    outstanding_client_responses: HashMap<i32, String>,
    /// Store server responses
    server_merged_responses: HashMap<i32, LspMessage>,

    /// Name of the default server
    pub default_server: String,
}

impl Tracker {
    /// Register when a server sends a request so that we know where to send the client result.
    pub fn server_issued_request(&mut self, requesting_server: String, msg_id: i32) {
        self.outstanding_client_responses
            .insert(msg_id, requesting_server);
    }

    /// Register a client request going out to a server
    pub fn client_issued_request(&mut self, msg_id: i32, server_name: String) {
        self.outstanding_server_responses
            .entry(msg_id)
            .and_modify(|servers| {
                servers.insert(server_name.to_string());
            })
            .or_insert({
                let mut servers = HashSet::new();
                servers.insert(server_name);
                servers
            });
    }

    /// Register a server result message for the corresponding request.
    pub fn server_responded(&mut self, msg_id: i32, server_name: &String, msg: LspMessage) {
        self.outstanding_server_responses
            .entry(msg_id)
            .and_modify(|v| {
                v.remove(server_name);
            });
        self.server_merged_responses
            .entry(msg_id)
            .and_modify(|v| v.merge(msg.clone()))
            .or_insert(msg);
    }

    pub fn get_merged_response(&mut self, msg_id: i32) -> Option<LspMessage> {
        self.server_merged_responses.remove(&msg_id)
    }

    /// Get the server to send the client response to
    pub fn client_responded(&mut self, msg_id: i32) -> Option<String> {
        self.outstanding_client_responses.remove(&msg_id)
    }

    /// return true if all servers have replied for a request
    pub fn all_servers_replied(&self, msg_id: i32) -> bool {
        if let Some(awaiting) = self.outstanding_server_responses.get(&msg_id) {
            awaiting.len() == 0
        } else {
            false
        }
    }

    /// Extract the servers from stdins that have the provided method based on the capabilities.
    pub fn get_capable_servers<'a>(
        &self,
        method: &'a str,
        stdins: &'a HashMap<String, Sender<LspMessage>>,
    ) -> HashMap<&'a String, &'a Sender<LspMessage>> {
        let mut filtered = HashMap::new();
        if method == "initialize" || method == "initialized" {
            for (ss, stdin) in stdins.iter() {
                filtered.insert(ss, stdin);
            }
            return filtered;
        }
        let cap_name = capability::CAP_MAP
            .get(method)
            .map(|v| v.to_string())
            .unwrap_or_default();

        // only take one server for other requests
        match self.capabilities.get(cap_name.as_str()) {
            Some(supported_servers) => {
                for ss in supported_servers {
                    if ss == &self.default_server && method != "textDocument/codeAction" {
                        if let Some((stdkey, stdin)) = stdins.get_key_value(ss) {
                            filtered.insert(stdkey, stdin);
                            break;
                        }
                    } else {
                        if let Some((stdkey, stdin)) = stdins.get_key_value(ss) {
                            filtered.insert(stdkey, stdin);
                        }
                    }
                }
                return filtered;
            }
            None => {
                eprintln!(
                    "No servers capable of {} for method {} sending to first",
                    cap_name, method
                );
                for (ss, stdin) in stdins.iter() {
                    filtered.insert(ss, stdin);
                    break;
                }
                return filtered;
            }
        }
    }

    /// register a server for a capability
    pub fn register_capability(&mut self, method: &str, server_name: String) {
        self.capabilities
            .entry(method.to_string())
            .and_modify(|cpble_svrs| cpble_svrs.push(server_name.to_string()))
            .or_insert(vec![server_name]);
    }
}
