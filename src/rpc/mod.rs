use std::fs::read;

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tokio::net::TcpStream;

#[allow(unused)]
pub struct Rpc {
        socket: WebSocketStream<MaybeTlsStream<TcpStream>>
}

#[allow(unused)]
#[derive(Deserialize)]
pub struct Download {
        pub gid: String,
        pub status: String,
        pub total_length: String,
        pub completed_length: String,
        pub download_speed: String,
        pub files: Vec<Files>
}

#[allow(unused)]
#[derive(Deserialize)]
pub struct Files {
        pub path: String,
        pub length: String,
}

#[allow(unused)]
impl Rpc {
        pub async fn connect(url: &str) -> anyhow::Result<Self> {
                let (ws, _) = connect_async(url).await?;
                Ok(Rpc { socket: ws })
        }

        async fn send_request(&mut self, method: &str, params: Value) -> anyhow::Result<Value> {
                let req = json!({
                        "jsonrpc": "2.0",
                        "method": method,
                        "id": "1",
                        "params": params
                });

                self.socket.send(Message::Text(req.to_string().into())).await?;

                if let Some(Ok(Message::Text(res))) = self.socket.next().await {
                        let parsed: Value = serde_json::from_str(&res)?;
                        Ok(parsed["result"].clone())
                } else {
                        anyhow::bail!("No response from server")
                }
        }

        pub async fn get_version(&mut self) -> anyhow::Result<String> {
                let result = self.send_request("aria2.getVersion", json!([])).await?;
                Ok(serde_json::from_value(result)?)
        }

        pub async fn tell_active(&mut self) -> anyhow::Result<Vec<Download>> {
                let result = self.send_request("aria2.tellActive", json!([])).await?;
                Ok(serde_json::from_value(result)?)
        }

        pub async fn tell_waiting(&mut self) -> anyhow::Result<Vec<Download>> {
                let result = self.send_request("aria2.tellWaiting", json!([])).await?;
                Ok(serde_json::from_value(result)?)
        }

        pub async fn tell_stopped(&mut self) -> anyhow::Result<Vec<Download>> {
                let result = self.send_request("aria2.tellStopped", json!([])).await?;
                Ok(serde_json::from_value(result)?)
        }

        pub async fn add_uri(&mut self, uri: String) -> anyhow::Result<String> {
                let result = self.send_request("aria2.addUri", json!([[uri.as_str()]])).await?;
                Ok(serde_json::from_value(result)?)
        }

        pub async fn add_torrent(&mut self, path: impl AsRef<std::path::Path>) -> anyhow::Result<String> {
                use base64::engine::general_purpose::STANDARD as b64;

                let bytes = read(path.as_ref())?;
                let torrent = b64.encode(&bytes);

                let result = self.send_request("aria2.addTorrent", json!([torrent.as_str()])).await?;
                Ok(serde_json::from_value(result)?)
        }
}
