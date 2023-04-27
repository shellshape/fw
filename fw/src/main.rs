mod config;
mod util;
mod watcher;

use crate::config::Config;
use clap::Parser;
use env_logger::Env;
use log::{debug, error};
use watcher::watch;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cmd {
    pub actions: Option<Vec<String>>,
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

    watch(&cfg, cmd.actions).await;
}
