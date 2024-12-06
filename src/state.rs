use std::time::Duration;

use tokio::sync::broadcast;

use crate::{cli::Cli, script::Script, types::Message};

#[derive(Debug, Clone)]
pub struct AppState {
    pub broadcast: broadcast::Sender<Message>,
    pub script: Script,
    pub keep_alive: Duration,
    pub keep_alive_text: String,
    pub timeout: Duration,
    pub timeout_retry: Duration,
}

impl AppState {
    pub async fn from_cli(cli: &Cli) -> anyhow::Result<Self> {
        let script = if cli.unsafe_script {
            Script::unsafe_new()
        } else {
            Script::new()
        };

        if let Some(path) = &cli.script {
            script.load_path(path).await?;
        }

        let (broadcast, _) = broadcast::channel(cli.capacity);

        Ok(Self {
            broadcast,
            script,
            keep_alive: cli.keep_alive,
            keep_alive_text: cli.keep_alive_text.clone(),
            timeout: cli.timeout,
            timeout_retry: cli.timeout_retry,
        })
    }
}
