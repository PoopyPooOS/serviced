use linux_ipc::IpcChannel;
pub use types::*;

mod types;

pub struct Init {
    ipc: IpcChannel,
}

impl Init {
    pub fn new(socket_path: &str) -> Self {
        let ipc = IpcChannel::connect(socket_path).unwrap();

        Self { ipc }
    }

    pub fn service_ready(&mut self, pid: u32) {
        self.ipc
            .send::<Command, ()>(Command::ServiceReady(pid))
            .expect("Failed to send service ready command");
    }
}
