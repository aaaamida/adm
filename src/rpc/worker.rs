use crate::rpc::{Download, Rpc};

#[allow(unused)]
pub enum RpcCommand {
        GetVersion,
        TellActive,
        TellWaiting,
        TellStopped,
        TellStatus(String),
        AddUri(String),
        AddTorrent(std::path::PathBuf),
        PauseDownload(String),
        PauseAllDownloads,
        UnpauseDownload(String),
        UnpauseAllDownloads
}

#[allow(unused)]
pub enum RpcResponse {
        CurrentVersion(String),
        ActiveDownloads(Vec<Download>),
        WaitingDownloads(Vec<Download>),
        StoppedDownloads(Vec<Download>),
        DownloadStatus(Download),
        // Gid(String),
        Error(String)
}

pub fn spawn_worker(
        cmd_rx: std::sync::mpsc::Receiver<RpcCommand>,
        res_tx: std::sync::mpsc::SyncSender<RpcResponse>,
) {
        std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("failed to create runtime");
                rt.block_on(async move {
                        let mut rpc = Rpc::connect("ws://127.0.0.1:6800/jsonrpc").await.expect("failed to connect to RPC");

                        while let Ok(cmd) = cmd_rx.recv() {
                                match cmd {
                                        RpcCommand::GetVersion => {
                                                match rpc.get_version().await {
                                                        Ok(v) => res_tx.send(RpcResponse::CurrentVersion(v)).ok(),
                                                        Err(e) => res_tx.send(RpcResponse::Error(e.to_string())).ok() 
                                                };
                                        },
                                        RpcCommand::TellActive => {
                                                match rpc.tell_active().await {
                                                        Ok(dls) => res_tx.send(RpcResponse::ActiveDownloads(dls)).ok(),
                                                        Err(e) => res_tx.send(RpcResponse::Error(e.to_string())).ok(),
                                                };
                                        },
                                        RpcCommand::TellWaiting => {
                                                match rpc.tell_active().await {
                                                        Ok(dls) => res_tx.send(RpcResponse::WaitingDownloads(dls)).ok(),
                                                        Err(e) => res_tx.send(RpcResponse::Error(e.to_string())).ok(),
                                                };
                                        },
                                        RpcCommand::TellStopped => {
                                                match rpc.tell_active().await {
                                                        Ok(dls) => res_tx.send(RpcResponse::StoppedDownloads(dls)).ok(),
                                                        Err(e) => res_tx.send(RpcResponse::Error(e.to_string())).ok(),
                                                };
                                        },
                                        RpcCommand::TellStatus(gid) => {
                                                match rpc.tell_status(gid).await {
                                                        Ok(status) => res_tx.send(RpcResponse::DownloadStatus(status)).ok(),
                                                        Err(e) => res_tx.send(RpcResponse::Error(e.to_string())).ok(),
                                                };
                                        },
                                        RpcCommand::AddUri(uri) => {
                                                rpc.add_uri(uri).await.ok();
                                        },
                                        RpcCommand::AddTorrent(path) => {
                                                rpc.add_torrent(path).await.ok();
                                        },
                                        RpcCommand::PauseDownload(gid) => {
                                                rpc.pause_download(gid).await.ok();
                                        },
                                        RpcCommand::UnpauseDownload(gid) => {
                                                rpc.unpause_download(gid).await.ok();
                                        },
                                        RpcCommand::PauseAllDownloads => {
                                                rpc.pause_all_downloads().await.ok();
                                        },
                                        RpcCommand::UnpauseAllDownloads => {
                                                rpc.unpause_all_downloads().await.ok();
                                        }
                                };
                        }
                });
        });
}
