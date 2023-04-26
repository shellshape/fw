mod config;
mod util;

use crate::config::{Command, Config};
use env_logger::Env;
use fwatch::{BasicTarget, Transition, Watcher};
use log::{debug, error, info};
use std::str;
use std::{process::Stdio, time::Duration};
use tokio::{process, time};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cfg = match Config::init() {
        Ok(v) => v,
        Err(err) => {
            error!("config initialization failed: {err}");
            return;
        }
    };

    debug!("Parsed config: {cfg:#?}");

    let sleep_interval = Duration::from_millis(cfg.check_interval_ms.unwrap_or(1000));

    let mut watcher: Watcher<BasicTarget> = Watcher::new();

    cfg.actions
        .iter()
        .flat_map(|action| &action.targets)
        .for_each(|target| watcher.add_target(BasicTarget::new(target.path())));

    info!("Watching targets ...");
    loop {
        'watch_loop: for (index, transition) in watcher
            .watch()
            .into_iter()
            .filter(|transition| !matches!(transition, Transition::None))
            .enumerate()
        {
            let Some(path) = watcher.get_path(index) else {
                error!("could not get path for index {index}");
                continue;
            };

            for action in &cfg.actions {
                for target in &action.targets {
                    if target.path() != path.to_string_lossy()
                        || !target.matches_transition(transition)
                    {
                        debug!("Change not tracked: {:?} -> {:?}", &path, &transition);
                        continue 'watch_loop;
                    }

                    info!("Change detected: {:?} -> {:?}", &path, &transition);
                    execute_commands(&action.commands).await;
                }
            }

            time::sleep(sleep_interval).await;
        }
    }
}

async fn execute_commands(cmds: &[Command]) {
    for cmd in cmds {
        let args = cmd.split_command();
        if args.is_empty() {
            continue;
        }

        let ex = args[0];
        let args = &args[1..];

        let mut exec = process::Command::new(ex);
        exec.args(args).current_dir(cmd.cwd());

        if cmd.is_async() {
            match exec.spawn() {
                Err(err) => {
                    error!("Command execution failed ({}): {}", cmd.command(), err);
                    return;
                }
                Ok(v) => {
                    info!("Command spawend ({}): ID: {:?}", cmd.command(), v.id());
                }
            }
        } else {
            match exec
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .output()
                .await
            {
                Err(err) => {
                    error!("Command execution failed ({}): {}", cmd.command(), err);
                    return;
                }
                Ok(out) => info!(
                    "Command executed ({}): {}",
                    cmd.command(),
                    str::from_utf8(out.stdout.as_slice()).unwrap_or_default()
                ),
            }
        }
    }
}
