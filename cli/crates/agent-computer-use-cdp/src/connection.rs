use crate::protocol::{CdpRequest, CdpResponse, TargetInfo, VersionInfo};
use agent_computer_use_core::Error;
use futures_util::{SinkExt, StreamExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

const HTTP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);
const WS_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

pub struct CdpConnection {
    ws: Mutex<WsStream>,
    next_id: AtomicU64,
    pub port: u16,
}

impl CdpConnection {
    pub async fn connect(port: u16) -> agent_computer_use_core::Result<Self> {
        if let Some(cached_url) = load_cached_ws_url(port) {
            if let Ok(conn) = Self::connect_ws(&cached_url, port).await {
                return Ok(conn);
            }
        }

        let ws_url = Self::discover_ws_url(port).await?;
        save_ws_url_cache(port, &ws_url);
        Self::connect_ws(&ws_url, port).await
    }

    async fn connect_ws(ws_url: &str, port: u16) -> agent_computer_use_core::Result<Self> {
        tracing::debug!("connecting to CDP at {ws_url}");

        let (ws, _) = tokio::time::timeout(WS_TIMEOUT, tokio_tungstenite::connect_async(ws_url))
            .await
            .map_err(|_| Error::PlatformError {
                message: format!("CDP WebSocket connection timed out (port {port})"),
            })?
            .map_err(|e| Error::PlatformError {
                message: format!("CDP WebSocket connection failed: {e}"),
            })?;

        Ok(Self {
            ws: Mutex::new(ws),
            next_id: AtomicU64::new(1),
            port,
        })
    }

    async fn discover_ws_url(port: u16) -> agent_computer_use_core::Result<String> {
        if let Ok(targets) = Self::http_get_json::<Vec<TargetInfo>>(port, "/json").await {
            if let Some(target) = targets
                .iter()
                .find(|t| t.target_type == "page")
                .or_else(|| targets.first())
            {
                if let Some(ref url) = target.web_socket_debugger_url {
                    return Ok(url.clone());
                }
            }
        }

        let version = Self::http_get_json::<VersionInfo>(port, "/json/version").await?;
        version
            .web_socket_debugger_url
            .ok_or_else(|| Error::PlatformError {
                message: format!("CDP on port {port} did not provide a WebSocket URL"),
            })
    }

    async fn http_get_json<T: serde::de::DeserializeOwned>(
        port: u16,
        path: &str,
    ) -> agent_computer_use_core::Result<T> {
        let url = format!("http://127.0.0.1:{port}{path}");

        let stream = tokio::time::timeout(
            HTTP_TIMEOUT,
            tokio::net::TcpStream::connect(format!("127.0.0.1:{port}")),
        )
        .await
        .map_err(|_| Error::PlatformError {
            message: format!("CDP connection timed out on port {port}"),
        })?
        .map_err(|e| Error::PlatformError {
            message: format!("cannot connect to CDP port {port}: {e}"),
        })?;

        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut stream = stream;
        let request =
            format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
        stream
            .write_all(request.as_bytes())
            .await
            .map_err(|e| Error::PlatformError {
                message: format!("CDP HTTP write failed: {e}"),
            })?;

        let mut buf = Vec::with_capacity(4096);
        match tokio::time::timeout(HTTP_TIMEOUT, stream.read_to_end(&mut buf)).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                return Err(Error::PlatformError {
                    message: format!("CDP HTTP read failed: {e}"),
                })
            }
            Err(_) if !buf.is_empty() => {
                tracing::debug!("CDP HTTP read timed out, using {} bytes", buf.len());
            }
            Err(_) => {
                return Err(Error::PlatformError {
                    message: format!("CDP HTTP read timed out on {url}"),
                })
            }
        }

        let response = String::from_utf8_lossy(&buf);
        let body = response
            .split("\r\n\r\n")
            .nth(1)
            .ok_or_else(|| Error::PlatformError {
                message: format!("invalid HTTP response from {url}"),
            })?;

        serde_json::from_str(body).map_err(|e| Error::PlatformError {
            message: format!("invalid JSON from {url}: {e}"),
        })
    }

    pub async fn send(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> agent_computer_use_core::Result<serde_json::Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let request = CdpRequest {
            id,
            method: method.to_string(),
            params,
        };

        let msg = serde_json::to_string(&request).map_err(|e| Error::PlatformError {
            message: format!("CDP serialize error: {e}"),
        })?;

        let mut ws = self.ws.lock().await;
        ws.send(Message::Text(msg))
            .await
            .map_err(|e| Error::PlatformError {
                message: format!("CDP send error: {e}"),
            })?;

        let deadline = tokio::time::Instant::now() + WS_TIMEOUT;

        loop {
            let msg = tokio::time::timeout_at(deadline, ws.next())
                .await
                .map_err(|_| Error::PlatformError {
                    message: format!("CDP response timed out for {method}"),
                })?
                .ok_or_else(|| Error::PlatformError {
                    message: "CDP connection closed".into(),
                })?
                .map_err(|e| Error::PlatformError {
                    message: format!("CDP read error: {e}"),
                })?;

            let text = match msg {
                Message::Text(t) => t.to_string(),
                Message::Binary(b) => String::from_utf8_lossy(&b).to_string(),
                Message::Close(_) => {
                    return Err(Error::PlatformError {
                        message: "CDP connection closed".into(),
                    })
                }
                _ => continue,
            };

            let resp: CdpResponse =
                serde_json::from_str(&text).map_err(|e| Error::PlatformError {
                    message: format!("CDP parse error: {e}"),
                })?;

            if resp.id != Some(id) {
                continue;
            }

            if let Some(err) = resp.error {
                return Err(Error::PlatformError {
                    message: format!("CDP error: {} ({})", err.message, err.code),
                });
            }

            return Ok(resp.result.unwrap_or(serde_json::Value::Null));
        }
    }

    pub async fn evaluate(
        &self,
        expression: &str,
    ) -> agent_computer_use_core::Result<serde_json::Value> {
        let result = self
            .send(
                "Runtime.evaluate",
                Some(serde_json::json!({
                    "expression": expression,
                    "returnByValue": true,
                    "awaitPromise": true,
                })),
            )
            .await?;

        if result.get("exceptionDetails").is_some() {
            let desc = result
                .pointer("/exceptionDetails/exception/description")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown JS error");
            return Err(Error::PlatformError {
                message: format!("JS error: {desc}"),
            });
        }

        Ok(result
            .pointer("/result/value")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }
}

fn cache_path(port: u16) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(format!("{home}/.agent-cu/cdp-{port}.json"))
}

fn load_cached_ws_url(port: u16) -> Option<String> {
    let path = cache_path(port);
    let contents = std::fs::read_to_string(&path).ok()?;
    let cache: serde_json::Value = serde_json::from_str(&contents).ok()?;

    let ts = cache.get("ts")?.as_u64()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if now - ts > 300 {
        let _ = std::fs::remove_file(&path);
        return None;
    }

    cache.get("url")?.as_str().map(String::from)
}

fn save_ws_url_cache(port: u16, url: &str) {
    let path = cache_path(port);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let cache = serde_json::json!({ "url": url, "ts": now });
    let _ = std::fs::write(&path, serde_json::to_string(&cache).unwrap_or_default());
}
