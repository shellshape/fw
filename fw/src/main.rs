mod config;
mod util;
mod watcher;

use crate::config::Config;
use clap::Parser;
use env_logger::Env;
use log::{debug, error};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cmd {
    /// Select actions to be watched. If none are specified,
    /// all actions are watched.
    pub actions: Option<Vec<String>>,

    /// Custom config location.
    #[arg(short, long)]
    pub config: Option<String>,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cmd = Cmd::parse();

    let res = match cmd.config {
        Some(cfg) => Config::from_file(cfg),
        None => Config::init(),
    };

    let cfg = match res {
        Ok(v) => v,
        Err(err) => {
            error!("config initialization failed: {err}");
            return;
        }
    };

    debug!("Parsed config: {cfg:#?}");

    watcher::watch(&cfg, cmd.actions).await;
}
