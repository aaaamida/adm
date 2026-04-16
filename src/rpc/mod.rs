pub mod worker;

use std::fs::read;

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tokio::net::TcpStream;
use anyhow::{bail, Result};

#[allow(unused)]
pub struct Rpc {
        socket: WebSocketStream<MaybeTlsStream<TcpStream>>
}

// TODO: match the struct members w aria2's download status struct
#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Download {
        pub gid: String,

        // NOTE: possible values: active, waiting, paused, error, complete, removed
        // TODO: should probably turn it into an enum idk
        pub status: String,
        pub total_length: Option<String>,
        pub completed_length: Option<String>,
        pub download_speed: Option<String>,
        pub upload_speed: Option<String>,
        pub files: Vec<Files>,

        // TODO: impl this feature later maybe
        // torrent only
        // pub info_hash: String,
        // pub seeder: bool,
        // pub bittorrent: BitTorrent,
}

#[allow(unused)]
#[derive(Deserialize)]
#[derive(Debug)]
pub struct Files {
        pub path: String,
        pub length: String,
}

// #[allow(unused)]
// #[derive(Deserialize)]
// pub struct BitTorrent {
//         pub creation_date: u64,
//         pub mode: String,
//         pub comment: String,
// }

impl Default for Download {
        fn default() -> Self {
                Download {
                        gid: "".into(),
                        status: "waiting".into(),
                        total_length: Some("0".into()),
                        completed_length: Some("0".into()),
                        download_speed: Some("0".into()),
                        upload_speed: Some("0".into()),
                        files: vec![],

                        // info_hash: "".into(),
                        // seeder: false,
                        // bittorrent: BitTorrent {
                        //         creation_date: 0u64,
                        //         mode: "single".into(),
                        //         comment: "".into()
                        // }
                }
        }
}

#[allow(unused)]
impl Rpc {
        pub async fn connect(url: &str) -> Result<Self> {
                let (ws, _) = connect_async(url).await?;
                Ok(Rpc { socket: ws })
        }

        async fn send_request(&mut self, method: &str, params: Value) -> Result<Value> {
                let req = json!({
                        "jsonrpc": "2.0",
                        "method": method,
                        "id": "1",
                        "params": params
                });

                self
                        .socket
                        .send(Message::Text(req.to_string().into()))
                        .await?;

                if let Some(Ok(Message::Text(res))) = self.socket.next().await {
                        let parsed: Value = serde_json::from_str(&res)?;
                        Ok(parsed["result"].clone())
                } else {
                        bail!("No response from server")
                }
        }

        pub async fn get_version(&mut self) -> Result<String> {
                let result = self.send_request("aria2.getVersion", json!([])).await?;
                Ok(serde_json::from_value(result)?)
        }

        pub async fn tell_active(&mut self) -> Result<Vec<Download>> {
                let result = self.send_request("aria2.tellActive", json!([[
                        "gid", "status", "totalLength", "completedLength",
                        "downloadSpeed", "uploadSpeed", "files"
                ]])).await?;
                if result.is_null() {
                        return Ok(vec![]);
                }
                Ok(serde_json::from_value(result)?)
        }

        pub async fn tell_waiting(&mut self) -> Result<Vec<Download>> {
                let result = self.send_request("aria2.tellWaiting", json!([0, 100, [
                        "gid", "status", "totalLength", "completedLength",
                        "downloadSpeed", "uploadSpeed", "files"
                ]])).await?;
                if result.is_null() {
                        return Ok(vec![]);
                }
                Ok(serde_json::from_value(result)?)
        }

        pub async fn tell_stopped(&mut self) -> Result<Vec<Download>> {
                let result = self.send_request("aria2.tellStopped", json!([0, 100, [
                        "gid", "status", "totalLength", "completedLength",
                        "downloadSpeed", "uploadSpeed", "files"
                ]])).await?;
                if result.is_null() {
                        return Ok(vec![]);
                }
                Ok(serde_json::from_value(result)?)
        }

        // NOTE: this should return a struct
        pub async fn tell_status(&mut self, gid: String) -> Result<Download> {
        let result = self.send_request("aria2.tellStatus", json!([gid.as_str(), [
                        "gid", "status", "totalLength", "completedLength",
                        "downloadSpeed", "uploadSpeed", "files"
                ]])).await?;
                Ok(serde_json::from_value(result)?)
        }

        pub async fn add_uri(&mut self, uri: String) -> Result<String> {
                let result = self.send_request("aria2.addUri", json!([[uri.as_str()]])).await?;
                Ok(serde_json::from_value(result)?)
        }

        pub async fn add_torrent(&mut self, path: impl AsRef<std::path::Path>) -> Result<String> {
                use base64::engine::general_purpose::STANDARD as b64;

                let bytes = read(path.as_ref())?;
                let torrent = b64.encode(&bytes);

                let result = self.send_request("aria2.addTorrent", json!([torrent.as_str()])).await?;
                Ok(serde_json::from_value(result)?)
        }

        pub async fn pause_download(&mut self, gid: String) -> Result<String> {
                let result = self.send_request("aria2.pause", json!([gid.as_str()])).await?;
                Ok(serde_json::from_value(result)?)
        }

        pub async fn pause_all_downloads(&mut self) -> Result<()> {
                let _ = self.send_request("aria2.pauseAll", json!([])).await?;
                Ok(())
        }

        pub async fn unpause_download(&mut self, gid: String) -> Result<String> {
                let result = self.send_request("aria2.unpause", json!([gid.as_str()])).await?;
                Ok(serde_json::from_value(result)?)
        }

        pub async fn unpause_all_downloads(&mut self) -> Result<()> {
                let _ = self.send_request("aria2.unpauseAll", json!([])).await?;
                Ok(())
        }

        pub async fn remove_download(&mut self, gid: String) -> Result<()> {
                let _ = self.send_request("aria2.remove", json!([gid.as_str()])).await?;
                Ok(())
        }

        pub async fn remove_result(&mut self, gid: String) -> Result<()> {
                let _ = self.send_request("aria2.removeDownloadResult", json!([gid.as_str()])).await?;
                Ok(())
        }

        pub async fn force_remove_download(&mut self, gid: String) -> Result<()> {
                let _ = self.send_request("aria2.forceRemove", json!([gid.as_str()])).await?;
                Ok(())
        }
}
