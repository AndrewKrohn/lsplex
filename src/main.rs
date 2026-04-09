use std::collections::HashMap;

use clap::Parser;
use tokio::sync::mpsc;

use manager::{client::client_handler, lsp};

mod manager;

struct LspDescription {
    lsp_name: String,
    lsp_args: Vec<String>,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Invocation for an LSP server. Can be used multiple times.
    /// Wrap the entire argument in quotes if switches or flags are used
    #[arg(long, num_args = 1..)]
    server: Vec<String>,
}

fn cli() -> Result<Vec<LspDescription>, ()> {
    let invoke = Cli::parse();
    let mut lsps = Vec::new();
    for server in invoke.server {
        let server_instance: Vec<&str> = server.split(' ').collect();
        let lsp_name = match server_instance.get(0) {
            Some(sname) => sname.to_string(),
            None => return Err(()),
        };
        let mut lsp_args: Vec<String> = Vec::new();
        let mut i = 1;
        while let Some(server_arg) = server_instance.get(i) {
            lsp_args.push(server_arg.to_string());
            i += 1;
        }
        lsps.push(LspDescription { lsp_name, lsp_args });
    }
    return Ok(lsps);
}

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time() // enable timeout on diagnostics
        .build()
        .unwrap();
    rt.block_on(async {
        // set up channels and servers
        let mut stdins = HashMap::new();
        let mut lsp_procs = Vec::new();
        let mut default_ls = None;
        if let Ok(lsps_desc) = cli() {
            for ls in lsps_desc.into_iter() {
                if default_ls.is_none() {
                    default_ls = Some(ls.lsp_name.to_string());
                }
                let LspDescription { lsp_name, lsp_args } = ls;
                let (lsp_proc, to_lsp) = lsp::Lsp::create(lsp_name.to_string(), lsp_args);
                stdins.insert(lsp_name, to_lsp);
                lsp_procs.push(lsp_proc);
            }
        }
        let (tx_midware_client, from_client) = mpsc::channel(1);
        let (tx_client, from_midware_client) = mpsc::channel(1);
        let (tx_midware_server, from_servers) = mpsc::channel(1);
        let mut handles = Vec::new();
        for ls in lsp_procs {
            let pass_tx = tx_midware_server.clone();
            handles.push(tokio::task::spawn(async {
                if let Err(x) = ls.run(pass_tx).await {
                    eprintln!("Error starting ls {x:?}");
                }
            }));
        }
        let lsplex = tokio::task::spawn(manager::run_middleware(
            stdins,
            (tx_client, from_client),
            from_servers,
            default_ls.unwrap(),
        ));
        let client = tokio::task::spawn(client_handler(tx_midware_client, from_midware_client));
        if let Err(e) = lsplex.await {
            eprintln!("lsplex failed... {}", e);
        }
        if let Err(e) = client.await {
            eprintln!("lsplex failed... {}", e);
        }
    });
}
