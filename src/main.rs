#![allow(clippy::unused_self)]

use logger::fatal;
use std::{env, process};
use tokio::fs;

mod config;
mod service;
mod sort;
mod types;
mod util;

#[tokio::main]
async fn main() -> ! {
    logger::set_app_name!();
    unsafe { env::set_var("SERVICED_PID", process::id().to_string()) };

    let config = config::read().unwrap_or_else(|err| {
        err.output();
        process::exit(1);
    });

    fs::create_dir_all("/tmp/ipc/services")
        .await
        .unwrap_or_else(|err| {
            fatal!("Failed to create service IPC directory: {:#?}.", err);
            process::exit(1);
        });

    let manager = service::Manager::new(config.services).unwrap_or_else(|err| {
        err.output();
        process::exit(1);
    });

    manager.start().await;
}
