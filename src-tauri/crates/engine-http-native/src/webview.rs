use std::io::Write;
use std::process::{Command, Stdio};

use engine_model::{Error, Result, WebViewBridge, WebViewRequest, WebViewResponse};

#[derive(Debug, Clone)]
pub struct CommandWebViewBridge {
    command: String,
}

impl CommandWebViewBridge {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }

    pub fn command(&self) -> &str {
        &self.command
    }
}

impl WebViewBridge for CommandWebViewBridge {
    fn fetch<'a>(
        &'a self,
        request: WebViewRequest,
    ) -> engine_model::BoxFuture<'a, Result<WebViewResponse>> {
        let command = self.command.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || run_bridge(&command, request))
                .await
                .map_err(|error| Error::Other(format!("WebView bridge task failed: {error}")))?
        })
    }
}

fn run_bridge(command: &str, request: WebViewRequest) -> Result<WebViewResponse> {
    let input = serde_json::to_vec(&request)?;
    let mut child = Command::new(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| Error::Other(format!("start WebView bridge `{command}`: {error}")))?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(&input)
            .map_err(|error| Error::Other(format!("write WebView bridge input: {error}")))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| Error::Other(format!("wait WebView bridge: {error}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Other(format!(
            "WebView bridge exited with {}: {}",
            output.status,
            stderr.trim()
        )));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| Error::Other(format!("parse WebView bridge output: {error}")))
}
