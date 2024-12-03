use std::time::Duration;

pub async fn until(mut condition: impl FnMut() -> bool, poll_interval: Duration) {
    while !condition() {
        tokio::time::sleep(poll_interval).await;
    }
}
