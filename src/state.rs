use std::{path::PathBuf, time::Duration};

use tokio::sync::broadcast;

use crate::{cli::Cli, req::PublishRequest, script::Script};

#[derive(Debug, Clone)]
pub struct AppState {
    pub broadcast: broadcast::Sender<PublishRequest>,
    pub script: Script,
    pub keep_alive: Duration,
    pub keep_alive_text: String,
    pub timeout: Duration,
    pub timeout_retry: Duration,
    pub pub_path: String,
    pub sub_path: String,
    pub serve_root_dir: Option<PathBuf>,
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
            pub_path: cli.pub_path.clone(),
            sub_path: cli.sub_path.clone(),
            serve_root_dir: cli.serve_root_dir.clone(),
        })
    }
}
