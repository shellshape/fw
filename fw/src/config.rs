use std::{collections::HashMap, ops::Deref, path::Path};

use crate::util::{split_cmd_trimmed, transtion_to_string};
use anyhow::Result;
use directories::ProjectDirs;
use figment::{
    providers::{Format, Json, Toml, Yaml},
    Figment,
};
use fwatch::Transition;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub check_interval_ms: Option<u64>,
    pub actions: HashMap<String, Action>,
}

#[derive(Debug, Deserialize)]
pub struct Action {
    pub targets: Vec<Target>,
    pub commands: Vec<Command>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Target {
    Path(String),
    PathDetails(PathDetails),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Command {
    Command(String),
    CommandDetails(CommandDetails),
}

#[derive(Debug, Deserialize, Clone)]
pub struct PathDetails {
    pub path: String,
    pub transitions: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommandDetails {
    pub cmd: String,
    pub cwd: Option<String>,
    #[serde(rename = "async")]
    pub exec_async: Option<bool>,
}

impl Config {
    pub fn init() -> Result<Self> {
        let dirs = ProjectDirs::from("de", "zekro", "fw")
            .ok_or_else(|| anyhow::anyhow!("could not resolve project directories"))?;

        Ok(Figment::new()
            .merge(Toml::file(dirs.config_dir().join("config.toml")))
            .merge(Toml::file("fw.toml"))
            .merge(Yaml::file(dirs.config_dir().join("config.yml")))
            .merge(Yaml::file("fw.yml"))
            .merge(Json::file(dirs.config_dir().join("config.json")))
            .merge(Json::file("fw.json"))
            .extract()?)
    }

    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<Self> {
        let ext = path.as_ref().extension().unwrap_or_default();
        let mut figment = Figment::new();

        figment = match ext.to_string_lossy().deref() {
            "yml" | "yaml" => figment.merge(Yaml::file(path)),
            "toml" => figment.merge(Toml::file(path)),
            "json" => figment.merge(Json::file(path)),
            _ => anyhow::bail!("invalid config file type"),
        };

        Ok(figment.extract()?)
    }
}

impl Target {
    pub fn path(&self) -> &str {
        match self {
            Self::Path(pth) => pth,
            Self::PathDetails(d) => &d.path,
        }
    }

    pub fn matches_transition(&self, t: Transition) -> bool {
        match self {
            Self::Path(_) => true,
            Self::PathDetails(d) => d
                .transitions
                .iter()
                .map(|ts| ts.to_lowercase())
                .any(|ts| ts == transtion_to_string(&t)),
        }
    }
}

impl Command {
    pub fn command(&self) -> &str {
        match self {
            Self::Command(cmd) => cmd,
            Self::CommandDetails(d) => &d.cmd,
        }
    }

    pub fn split_command(&self) -> Vec<&str> {
        match self {
            Self::Command(cmd) => split_cmd_trimmed(cmd),
            Self::CommandDetails(d) => split_cmd_trimmed(&d.cmd),
        }
    }

    pub fn cwd(&self) -> String {
        match self {
            Self::Command(_) => "./".into(),
            Self::CommandDetails(d) => d.cwd.clone().unwrap_or_else(|| "./".into()),
        }
    }

    pub fn is_async(&self) -> bool {
        matches!(self, Command::CommandDetails(d)
            if matches!(d.exec_async, Some(a) if a))
    }
}
