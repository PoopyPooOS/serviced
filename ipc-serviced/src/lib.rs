#[cfg(not(feature = "panic"))]
use logger::{Log, make_fatal, make_warn};
use rustix::process::{Pid, Signal, kill_process};
use std::env;

#[cfg(not(feature = "panic"))]
pub fn get_pid() -> Result<Pid, Box<Log>> {
    let raw_pid = env::var("SERVICED_PID")
        .map_err(|_| {
            Box::new(make_fatal!(
                "SERVICED_PID environment variable not set, was this launched manually?"
            ))
        })?
        .parse::<i32>()
        .map_err(|_| {
            Box::new(make_fatal!(
                "SERVICED_PID environment variable is not a non-zero integer"
            ))
        })?;

    match Pid::from_raw(raw_pid) {
        Some(pid) => Ok(pid),
        None => Err(Box::new(make_fatal!(
            "SERVICED_PID environment variable is not a non-zero integer"
        ))),
    }
}

#[cfg(feature = "panic")]
pub fn get_pid() -> Pid {
    let raw_pid = env::var("SERVICED_PID")
        .expect("SERVICED_PID environment variable not set, was this launched manually?")
        .parse::<i32>()
        .expect("SERVICED_PID environment variable is not a non-zero integer");

    Pid::from_raw(raw_pid).expect("SERVICED_PID environment variable is not a non-zero integer")
}

#[cfg(not(feature = "panic"))]
pub fn ready(pid: Pid) -> Result<(), Box<Log>> {
    match kill_process(pid, Signal::USR1) {
        Ok(()) => Ok(()),
        Err(err) => Err(Box::new(make_warn!(
            "Failed to send ready signal to serviced: {err:#?}"
        ))),
    }
}
#[cfg(feature = "panic")]
pub fn ready(pid: Pid) {
    use logger::warn;

    match kill_process(pid, Signal::USR1) {
        Ok(()) => (),
        Err(err) => warn!("Failed to send ready signal to serviced: {err:#?}"),
    }
}
