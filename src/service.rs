use crate::{
    config::Services,
    sort::sort_services,
    types::{Service, Status},
    util::until,
};
use futures::stream::StreamExt;
use logger::{Log, error, info, make_fatal};
use rustix::process::{Pid, Signal, kill_process};
use signal_hook::{consts::signal::SIGUSR1, iterator::exfiltrator::WithOrigin};
use signal_hook_tokio::SignalsInfo;
use std::{
    process::{self, Command},
    sync::Arc,
    thread,
    time::Duration,
};
use tokio::sync::Mutex;

pub struct Manager {
    pub services: Arc<Mutex<Vec<Service>>>,
}

impl Manager {
    pub fn new(services: Services) -> Result<Self, Box<Log>> {
        let mut services = services
            .into_iter()
            .map(|(service_id, mut service)| {
                service.id = service_id;
                service
            })
            .collect::<Vec<Service>>();

        services = match sort_services(&services) {
            Ok(services) => services,
            Err(err) => {
                return Err(Box::new(make_fatal!(
                    "Failed to sort services for services: {err:#?}.",
                )));
            }
        };

        Ok(Self {
            services: Arc::new(Mutex::new(services)),
        })
    }

    pub async fn start(&self) -> ! {
        let mut signals =
            SignalsInfo::<WithOrigin>::new([SIGUSR1]).expect("Failed to create signal handler");

        let services_clone = Arc::clone(&self.services);
        tokio::spawn(async move {
            while let Some(origin) = signals.next().await {
                match origin.signal {
                    SIGUSR1 => {
                        if let Some(process) = origin.process {
                            let mut services = services_clone.lock().await;
                            for service in &mut services.iter_mut() {
                                if service.pid == Some(process.pid)
                                    && service.status == Status::Starting
                                {
                                    info!("{service} is ready.");
                                    service.status = Status::Running;
                                }
                            }
                        }
                    }
                    other => unreachable!("The signal '{other}' is not captured."),
                }
            }
        });

        let mut services = self.services.lock().await.clone();

        for service in &mut services.iter_mut() {
            if !service.enabled || service.status != Status::Stopped {
                continue;
            }

            self.start_service(service).await;
        }

        loop {
            thread::sleep(Duration::from_secs(u64::MAX));
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    async fn start_service(&self, service: &mut Service) {
        info!("Starting {service}.");
        service.status = Status::Starting;

        let services = &self.services.lock().await;

        let mut dependencies = services
            .iter()
            .filter(|s| {
                service.dependencies.contains(&s.id)
                    && (s.status == Status::Running || s.status == Status::Stopped)
            })
            .collect::<Vec<&Service>>();

        for service in &mut dependencies {
            if service.status != Status::Stopped || service.status != Status::Running {
                continue;
            }

            until(
                || service.status == Status::Running,
                Duration::from_millis(25),
            )
            .await;
        }

        let mut command = Command::from(&service.exec);
        let mut child = command.spawn().unwrap_or_else(|err| {
            error!("Failed to start service \"{}\": {err:#?}.", service.id);
            process::exit(1)
        });

        service.pid = Some(child.id() as i32);

        let service = service.clone();
        tokio::spawn(async move {
            let child_exit_status = child.wait().unwrap_or_else(|err| {
                error!("Failed to wait for service \"{}\": {err:#?}.", service.id);
                process::exit(1)
            });

            if !child_exit_status.success() {
                error!(
                    "Service \"{}\" exited with non-zero status: {child_exit_status:#?}.",
                    service.id
                );
                process::exit(1)
            }
        });
    }

    #[allow(dead_code, reason = "Will be used in the future")]
    fn stop_service(&self, service: &mut Service) -> Result<(), Box<Log>> {
        info!("Stopping {}.", service);
        service.status = Status::Stopping;

        if let Some(Some(pid)) = service.pid.map(Pid::from_raw) {
            kill_process(pid, Signal::TERM).map_err(|err| {
                Box::new(make_fatal!(
                    "Failed to send SIGTERM to service \"{}\": {err:#?}.",
                    service.id
                ))
            })
        } else {
            Err(Box::new(make_fatal!(
                "Service \"{}\" is not running.",
                service.id
            )))
        }
    }
}
