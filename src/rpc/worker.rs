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
        UnpauseAllDownloads,
        RemoveDownload(String),
        RemoveResult(String),
        ForceRemoveDownload(String),
}

#[allow(unused)]
#[derive(Debug)]
pub enum RpcResponse {
        CurrentVersion(String),
        ActiveDownloads(Vec<Download>),
        WaitingDownloads(Vec<Download>),
        StoppedDownloads(Vec<Download>),
        DownloadStatus(Download),
        Gid(String),
        OK,
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
                                                match rpc.tell_waiting().await {
                                                        Ok(dls) => res_tx.send(RpcResponse::WaitingDownloads(dls)).ok(),
                                                        Err(e) => res_tx.send(RpcResponse::Error(e.to_string())).ok(),
                                                };
                                        },
                                        RpcCommand::TellStopped => {
                                                match rpc.tell_stopped().await {
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
                                                match rpc.add_uri(uri).await {
                                                        Ok(gid) => res_tx.send(RpcResponse::Gid(gid)).ok(),
                                                        Err(e)  => res_tx.send(RpcResponse::Error(e.to_string())).ok(),
                                                };
                                        },
                                        RpcCommand::AddTorrent(path) => {
                                                rpc.add_torrent(path).await.ok();
                                        },
                                        RpcCommand::PauseDownload(gid) => {
                                                match rpc.pause_download(gid).await {
                                                        Ok(gid) => res_tx.send(RpcResponse::Gid(gid)).ok(),
                                                        Err(e) => res_tx.send(RpcResponse::Error(e.to_string())).ok()
                                                };
                                        },
                                        RpcCommand::UnpauseDownload(gid) => {
                                                match rpc.unpause_download(gid).await {
                                                        Ok(gid) => res_tx.send(RpcResponse::Gid(gid)).ok(),
                                                        Err(e) => res_tx.send(RpcResponse::Error(e.to_string())).ok()
                                                };
                                        },
                                        RpcCommand::PauseAllDownloads => {
                                                match rpc.pause_all_downloads().await {
                                                        Ok(_) => res_tx.send(RpcResponse::OK).ok(),
                                                        Err(e) => res_tx.send(RpcResponse::Error(e.to_string())).ok(),
                                                };
                                        },
                                        RpcCommand::UnpauseAllDownloads => {
                                                match rpc.unpause_all_downloads().await {
                                                        Ok(_) => res_tx.send(RpcResponse::OK).ok(),
                                                        Err(e) => res_tx.send(RpcResponse::Error(e.to_string())).ok(),
                                                };
                                        },
                                        RpcCommand::RemoveDownload(gid) => {
                                                match rpc.remove_download(gid).await {
                                                        Ok(_) => res_tx.send(RpcResponse::OK).ok(),
                                                        Err(e) => res_tx.send(RpcResponse::Error(e.to_string())).ok(),
                                                };
                                        },
                                        RpcCommand::RemoveResult(gid) => {
                                                match rpc.remove_result(gid).await {
                                                        Ok(_) => res_tx.send(RpcResponse::OK).ok(),
                                                        Err(e) => res_tx.send(RpcResponse::Error(e.to_string())).ok(),
                                                };
                                        },
                                        RpcCommand::ForceRemoveDownload(gid) => {
                                                match rpc.force_remove_download(gid).await {
                                                        Ok(_) => res_tx.send(RpcResponse::OK).ok(),
                                                        Err(e) => res_tx.send(RpcResponse::Error(e.to_string())).ok(),
                                                };
                                        },
                                };
                        }
                });
        });
}
