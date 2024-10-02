use std::{fs, thread, time::Duration};

mod config;
mod service;

fn main() -> ! {
    let userspace_config = config::read();

    fs::create_dir_all("/tmp/ipc/serviced").expect("Failed to create /tmp/ipc/serviced");
    let manager = service::Manager::new(userspace_config.serviced_path);

    manager.load_all();

    infinite_loop()
}

pub fn infinite_loop() -> ! {
    loop {
        thread::sleep(Duration::from_secs(u64::MAX));
    }
}
