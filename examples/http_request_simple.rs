use async_retry::{AsyncRetry, ExponentialRetryStrategy, RetryPolicy};
use futures::TryFutureExt;
use reqwest::StatusCode;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let resp = AsyncRetry::new(
        || reqwest::get("http://localhost:8085").map_err(|e| RetryPolicy::Repeat(Some(e))),
        ExponentialRetryStrategy::default()
            .max_attempts(50)
            .initial_delay(Duration::from_millis(100)),
    )
    .await?;

    eprintln!("resp = {:#?}", resp);

    Ok(())
}
