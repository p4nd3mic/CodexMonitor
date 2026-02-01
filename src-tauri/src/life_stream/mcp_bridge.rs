//! Bridge to life-mcp Node.js server for rich data operations.
//!
//! Spawns life-mcp as a subprocess and communicates via JSON-RPC over stdio.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
    data: Option<Value>,
}

#[derive(Default)]
struct BridgeState {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<BufReader<ChildStdout>>,
    request_id: u64,
}

#[derive(Clone, Default)]
pub struct LifeMcpBridge {
    state: Arc<Mutex<BridgeState>>,
    mcp_path: Option<String>,
}

impl LifeMcpBridge {
    pub fn new(mcp_path: Option<String>) -> Self {
        Self {
            state: Arc::new(Mutex::new(BridgeState { request_id: 1, ..Default::default() })),
            mcp_path,
        }
    }

    pub fn from_env() -> Self {
        let mcp_path = env::var("LIFE_MCP_PATH")
            .ok()
            .filter(|value| !value.trim().is_empty());
        Self::new(mcp_path)
    }

    pub fn is_enabled(&self) -> bool {
        self.mcp_path.is_some()
    }

    async fn ensure_started(&self) -> Result<(), String> {
        let mcp_path = self
            .mcp_path
            .as_ref()
            .ok_or_else(|| "life-mcp path not configured".to_string())?;
        let mut state = self.state.lock().await;
        if state.child.is_some() {
            return Ok(());
        }

        let script_path = format!("{}/dist/index.js", mcp_path);
        if !Path::new(&script_path).exists() {
            return Err(format!("life-mcp not found at {}", script_path));
        }

        let mut child = Command::new("node")
            .arg(&script_path)
            .env("MCP_MODE", "stdio")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .map_err(|err| format!("Failed to spawn life-mcp: {err}"))?;

        state.stdin = child.stdin.take();
        state.stdout = child
            .stdout
            .take()
            .map(|stdout| BufReader::new(stdout));
        state.child = Some(child);
        Ok(())
    }

    pub async fn call_tool(&self, tool_name: &str, args: Value) -> Result<Option<Value>, String> {
        if !self.is_enabled() {
            return Ok(None);
        }

        if let Err(err) = self.ensure_started().await {
            return Err(err);
        }

        let mut state = self.state.lock().await;
        let id = state.request_id;
        state.request_id = state.request_id.saturating_add(1);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: json!({
                "name": tool_name,
                "arguments": args,
            }),
            id,
        };

        let request_str = serde_json::to_string(&request)
            .map_err(|err| format!("Failed to serialize request: {err}"))?;

        if let Some(stdin) = &mut state.stdin {
            stdin
                .write_all(request_str.as_bytes())
                .await
                .map_err(|err| format!("Failed to write to life-mcp: {err}"))?;
            stdin
                .write_all(b"\n")
                .await
                .map_err(|err| format!("Failed to write newline: {err}"))?;
            stdin
                .flush()
                .await
                .map_err(|err| format!("Failed to flush stdin: {err}"))?;
        } else {
            return Err("life-mcp stdin unavailable".to_string());
        }

        let stdout = state
            .stdout
            .as_mut()
            .ok_or_else(|| "life-mcp stdout unavailable".to_string())?;
        let mut line = String::new();
        stdout
            .read_line(&mut line)
            .await
            .map_err(|err| format!("Failed to read life-mcp response: {err}"))?;

        if line.trim().is_empty() {
            return Err("life-mcp response empty".to_string());
        }

        let response: JsonRpcResponse =
            serde_json::from_str(&line).map_err(|err| format!("Invalid response: {err}"))?;
        if let Some(error) = response.error {
            return Err(format!("RPC error: {} (code {})", error.message, error.code));
        }

        Ok(response.result)
    }

    pub async fn call_tool_strict(&self, tool_name: &str, args: Value) -> Result<Value, String> {
        match self.call_tool(tool_name, args).await? {
            Some(value) => Ok(value),
            None => Err("life-mcp returned empty result".to_string()),
        }
    }

    pub async fn stop(&self) {
        let mut state = self.state.lock().await;
        if let Some(mut child) = state.child.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        state.stdin = None;
        state.stdout = None;
    }
}

impl Drop for LifeMcpBridge {
    fn drop(&mut self) {
        if Arc::strong_count(&self.state) == 1 {
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                let state = Arc::clone(&self.state);
                handle.spawn(async move {
                    let mut guard = state.lock().await;
                    if let Some(mut child) = guard.child.take() {
                        let _ = child.kill().await;
                        let _ = child.wait().await;
                    }
                    guard.stdin = None;
                    guard.stdout = None;
                });
            }
        }
    }
}
