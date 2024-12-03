use crate::types::Service;
use logger::{make_fatal, Log};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};
use tl::eval;

// BTreeMap is used instead of a HashMap to preserve the order of service groups
pub type Services = BTreeMap<String, Service>;

#[derive(Debug, Deserialize)]
pub struct PartialConfig {
    pub services: Services,
}

pub fn read() -> Result<PartialConfig, Box<Log>> {
    match eval::<PartialConfig>(
        fs::read_to_string("/system/config.tl")
            .map_err(|_| Box::new(make_fatal!("Failed to read config file", hint: "Check if /system/config.tl exists")))?,
    )? {
        Some(config) => Ok(config),
        None => Err(Box::new(
            make_fatal!("Failed to evaluate config file", hint: "Check if /system/config.tl is valid"),
        )),
    }
}
