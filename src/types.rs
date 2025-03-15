use serde::Deserialize;
use serde_inline_default::serde_inline_default;
use std::{collections::HashMap, fmt::Display, process::Command};

#[derive(Debug, Clone, Deserialize)]
#[serde_inline_default]
pub struct Service {
    pub name: String,
    #[serde(skip)]
    pub id: String,
    pub exec: Exec,
    #[serde_inline_default(true)]
    pub enabled: bool,
    #[serde(default)]
    pub dependencies: Vec<String>,
    // TODO: Use this.
    #[serde(default)]
    #[allow(dead_code)]
    pub io: Vec<IoOption>,
    #[serde(skip)]
    pub status: Status,
    #[serde(skip)]
    pub pid: Option<i32>,
}

impl Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Debug, Clone)]
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

impl<'de> Deserialize<'de> for Exec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Exec(s))
    }
}

#[derive(Debug, Clone)]
pub enum IoOption {
    Out,
    In,
    Err,
}

impl<'de> Deserialize<'de> for IoOption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "out" => Ok(IoOption::Out),
            "in" => Ok(IoOption::In),
            "err" => Ok(IoOption::Err),
            _ => Err(serde::de::Error::custom("Invalid io option")),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub enum Status {
    #[default]
    Stopped,
    Starting,
    Running,
    Stopping,
}
