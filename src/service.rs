use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::service;

#[derive(Debug, Deserialize)]
pub struct Service {
    pub name: String,
    pub id: String,
    pub exec: Exec,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub io: Vec<IoOption>,
    #[serde(skip)]
    pub status: Status,
    #[serde(skip)]
    pub pid: Option<u32>,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct Exec(pub String);

impl From<&Exec> for Command {
    fn from(val: &Exec) -> Self {
        let parts = val.0.split_whitespace();
        let mut env = HashMap::new();
        let mut program = String::new();
        let mut args = Vec::new();

        for part in parts {
            if part.contains('=') {
                let mut kv = part.splitn(2, '=');
                let key = kv.next().unwrap().to_string();
                let value = kv.next().unwrap().to_string();
                env.insert(key, value);
            } else if program.is_empty() {
                program = part.to_string();
            } else {
                args.push(part.to_string());
            }
        }

        let mut command = Command::new(program);
        command.envs(env);
        command.args(args);

        command
    }
}

#[derive(Debug, Deserialize)]
pub enum IoOption {
    Out,
    In,
    Err,
}

#[derive(Debug, Deserialize, PartialEq, Default)]
pub enum Status {
    Stopping,
    #[default]
    Stopped,
    Starting,
    Running,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub service: Vec<Service>,
}

pub struct Manager {
    pub service_groups: Vec<Vec<Service>>,
}

impl Manager {
    pub fn new(services_path: PathBuf) -> Self {
        assert!(services_path.exists(), "Services config not found");

        let service_groups = fs::read_dir(&services_path)
            .expect("Failed to read service group configs")
            .filter_map(Result::ok)
            .filter(|file| file.path().extension().map_or(false, |ext| ext == "toml"))
            .map(|file| fs::read_to_string(file.path()).expect("Failed to read service file"))
            .map(|raw| toml::from_str::<service::Config>(&raw).expect("Failed to parse service file"))
            .map(|service_config| service_config.service)
            .collect::<Vec<_>>();

        Self { service_groups }
    }

    pub fn load_all(&self) {
        for group in &self.service_groups {
            for service in group {
                let program = PathBuf::from(&service.exec.0);

                println!("Loading {}", program.display());
                println!(
                    "{} {}",
                    program.display(),
                    if program.exists() { "exists" } else { "does not exist" }
                );

                if let Err(err) = Command::from(&service.exec).spawn() {
                    eprintln!("Service \"{}\" failed to start: {:#?}", service.id, err);
                }
            }
        }
    }
}
