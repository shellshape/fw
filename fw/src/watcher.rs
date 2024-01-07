use crate::config::{Command, Config};
use crate::util::transtion_to_string;
use fwatch::{BasicTarget, Transition, Watcher};
use log::{debug, error, info};
use std::ffi::OsStr;
use std::str;
use std::{process::Stdio, time::Duration};
use tokio::{process, time};

pub async fn watch(cfg: &Config, filter: Option<Vec<String>>) {
    let sleep_interval = Duration::from_millis(cfg.check_interval_ms.unwrap_or(1000));

    let mut watcher: Watcher<BasicTarget> = Watcher::new();

    let actions: Vec<(_, _)> = cfg
        .actions
        .iter()
        .filter(|(name, _)| filter.is_none() || filter.as_ref().unwrap().contains(name))
        .collect();

    if actions.is_empty() {
        error!("No actions to watch matching the passed action names");
        return;
    }

    actions
        .iter()
        .flat_map(|(_, action)| &action.targets)
        .for_each(|target| watcher.add_target(BasicTarget::new(target.path())));

    let startup_actions = actions.iter().filter(|a| {
        a.1.run_commands_on_startup.is_some_and(|v| v) || a.1.startup_commands.is_some()
    });
    for (name, action) in startup_actions {
        info!("Executing startup commands for {} ...", name);
        if let Some(cmds) = action.startup_commands.as_ref() {
            execute_commands::<Vec<(&str, &str)>, &str, &str>(cmds, None).await;
        } else {
            execute_commands::<Vec<(&str, &str)>, &str, &str>(&action.commands, None).await;
        }
    }

    info!("Watching targets ...");
    loop {
        for (index, transition) in watcher
            .watch()
            .into_iter()
            .enumerate()
            .filter(|(_, transition)| !matches!(transition, Transition::None))
        {
            let Some(path) = watcher.get_path(index) else {
                error!("could not get path for index {index}");
                continue;
            };

            for (_, action) in &actions {
                for target in &action.targets {
                    if target.path() != path.to_string_lossy()
                        || !target.matches_transition(transition)
                    {
                        debug!("Change not tracked: {:?} -> {:?}", &path, &transition);
                        continue;
                    }

                    info!("Change detected: {:?} -> {:?}", &path, &transition);

                    let cmds = action.commands.clone();
                    let envmap = [
                        ("FW_PATH", format!("{}", path.clone().to_string_lossy())),
                        (
                            "FW_TRANSITION",
                            transtion_to_string(&transition).to_string(),
                        ),
                    ];
                    tokio::spawn(async move {
                        execute_commands(&cmds, Some(envmap)).await;
                    });
                }
            }
        }

        time::sleep(sleep_interval).await;
    }
}

async fn execute_commands<E, K, V>(cmds: &[Command], env: Option<E>)
where
    E: IntoIterator<Item = (K, V)> + Clone,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    for cmd in cmds {
        let args = cmd.split_command();
        if args.is_empty() {
            continue;
        }

        let ex = args[0];
        let args = &args[1..];

        let mut exec = process::Command::new(ex);
        exec.args(args).current_dir(cmd.cwd());
        if let Some(env) = env.as_ref() {
            exec.envs(env.clone());
        }

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
