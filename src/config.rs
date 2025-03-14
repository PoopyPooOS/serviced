use crate::types::Service;
use logger::{Log, make_fatal};
use serde::Deserialize;
use std::{collections::BTreeMap, path::PathBuf};
use tl::{Source, eval};

// BTreeMap is used instead of a HashMap to preserve the order of service groups
pub type Services = BTreeMap<String, Service>;

#[derive(Debug, Deserialize)]
pub struct PartialConfig {
    pub services: Services,
}

pub fn read() -> Result<PartialConfig, Box<Log>> {
    let source = match Source::from_path(PathBuf::from("/config/system.tl")) {
        Ok(source) => source,
        Err(err) => {
            let err = make_fatal!(
                hint: "Check if '/system/config.tl' exists",
                "Failed to read config file: {err}"
            );

            return Err(Box::new(err));
        }
    };

    match eval::<PartialConfig>(source)? {
        Some(config) => Ok(config),
        None => Err(Box::new(make_fatal!(
            hint: "Check if /system/config.tl is valid",
            "Failed to evaluate config file"
        ))),
    }
}
