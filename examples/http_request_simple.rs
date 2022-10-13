use async_retry::{AsyncRetry, ExponentialRetryStrategy, RetryPolicy};
use reqwest::StatusCode;
use std::time::Duration;
use futures::TryFutureExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let resp = AsyncRetry::new(
        || reqwest::get("http://localhost:8085").map_err(RetryPolicy::Fail),
        ExponentialRetryStrategy::default()
            .max_attempts(5)
            .initial_delay(Duration::from_millis(100)),
    )
    .await?;

    eprintln!("resp = {:#?}", resp);

    Ok(())
}
